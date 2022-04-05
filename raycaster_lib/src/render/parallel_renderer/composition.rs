use std::{
    cell::UnsafeCell,
    cmp::min,
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use crossbeam::select;
use nalgebra::{vector, Vector3};

use crate::{common::PixelBox, volumetric::Block, PerspectiveCamera};

use super::{
    communication::CompWorkerComms,
    master_thread::PAR_SIDE,
    messages::{RenderTask, ToMasterMsg, ToWorkerMsg},
};

pub struct SubCanvas {
    queue: VecDeque<usize>, // indexes into blocks
    pub pixels: PixelBox,
    pub colors: Vec<Vector3<f32>>,
    pub opacities: Vec<f32>,
}

impl SubCanvas {
    pub fn new(pixels: PixelBox) -> Self {
        let size = pixels.items();
        let queue = VecDeque::new();
        let colors = vec![Vector3::zeros(); size];
        let opacities = vec![0.0; size];
        Self {
            queue,
            pixels,
            colors,
            opacities,
        }
    }
}

pub struct Canvas {
    size: PixelBox,
    sub_canvases: Vec<UnsafeCell<SubCanvas>>,
    remaining_subs: Mutex<u32>,
    tile_side: usize,
    tiles_x: usize, // number of tiles in one row
}

unsafe impl Sync for Canvas {}

impl Canvas {
    // Segment viewport into NxN subframes
    // Calc pixel sizes and offsets
    // todo return only PixelBox
    pub fn new(resolution: (usize, usize), tile_side: usize) -> Canvas {
        let (tiles_x, tiles_y) = Canvas::slice_into_tiles(resolution, tile_side);
        let size = PixelBox::new(0..resolution.0, 0..resolution.1);

        let mut sub_canvases = Vec::with_capacity(tiles_x * tiles_y); //todo
        println!(
            "Canvas split into {tiles_x}x{tiles_y} = {} tiles",
            tiles_x * tiles_y
        );

        for y in 0..tiles_y {
            let low_y = y * tile_side;
            for x in 0..tiles_x {
                let low_x = x * tile_side;
                let high_x = min(low_x + tile_side, resolution.0);
                let high_y = min(low_y + tile_side, resolution.1);
                let pixel_box = PixelBox::new(low_x..high_x, low_y..high_y);

                let sub_canvas = UnsafeCell::new(SubCanvas::new(pixel_box));
                sub_canvases.push(sub_canvas);
            }
        }

        let remaining_subs = sub_canvases.len() as u32;
        Canvas {
            sub_canvases,
            remaining_subs: Mutex::new(remaining_subs),
            size,
            tile_side,
            tiles_x,
        }
    }

    fn slice_into_tiles(resolution: (usize, usize), tile_side: usize) -> (usize, usize) {
        let tiles_x = (resolution.0 + tile_side - 1) / tile_side; // ceil
        let tiles_y = (resolution.1 + tile_side - 1) / tile_side; // ceil
        (tiles_x, tiles_y)
    }

    // Interior mutability, needs exclusive access
    pub fn build_queues(&self, camera: &PerspectiveCamera, blocks: &[Block<PAR_SIDE>]) {
        let mut block_infos = Vec::with_capacity(blocks.len());
        for (i, block) in blocks.iter().enumerate() {
            let distance = camera.box_distance(&block.bound_box);
            block_infos.push((i, distance));
        }

        // todo maybe cache this, until cam dir changes octant
        block_infos.sort_unstable_by(|b1, b2| b1.1.partial_cmp(&b2.1).unwrap()); // todo maybe by cached key

        // Info now sorted by distance

        let res = vector![self.size.width(), self.size.height()];

        // Reset done subcanvases counter
        {
            let mut remaining = self.remaining_subs.lock().unwrap();
            *remaining = self.sub_canvases.len() as u32;
        }

        for (block_id, _) in block_infos {
            let vpbox = camera.project_box(blocks[block_id].bound_box);
            let pixel_box = vpbox.get_pixel_range(res);
            // Count which pixelboxes intersect
            // Assume all tiles are the same size

            let (tiles_x_range, tiles_y_range) =
                Canvas::get_affected_tiles(pixel_box, self.tile_side);

            for y in tiles_y_range {
                for x in tiles_x_range.clone() {
                    let tile_id = self.tiles_x * y + x;

                    // Safety: build phase, only master has access
                    let tile = unsafe { self.sub_canvases[tile_id].get().as_mut().unwrap() };
                    tile.queue.push_back(block_id);
                }
            }
        }
    }

    fn get_affected_tiles(
        pixel_box: PixelBox,
        tile_side: usize,
    ) -> (std::ops::Range<usize>, std::ops::Range<usize>) {
        let tile_start_x = pixel_box.x.start / tile_side;
        let tile_end_x = (pixel_box.x.end + tile_side - 1) / tile_side;

        let tile_start_y = pixel_box.y.start / tile_side;
        let tile_end_y = (pixel_box.y.end + tile_side - 1) / tile_side;

        (tile_start_x..tile_end_x, tile_start_y..tile_end_y)
    }
}

enum Run {
    Stop,
    Continue,
    Render,
}

pub struct CompWorker {
    compositor_id: u8,
    compositor_count: u8,
    canvas: Arc<Canvas>,
    main_buffer: Arc<Mutex<Vec<u8>>>,
    comms: CompWorkerComms,
}

impl CompWorker {
    pub fn new(
        compositor_id: u8,
        compositor_count: u8,
        canvas: Arc<Canvas>,
        main_buffer: Arc<Mutex<Vec<u8>>>,
        comms: CompWorkerComms,
    ) -> Self {
        Self {
            compositor_id,
            compositor_count,
            canvas,
            main_buffer,
            comms,
        }
    }

    pub fn run(&self) {
        let mut command = None;
        loop {
            let msg = match command.take() {
                Some(cmd) => cmd,
                None => self.comms.command_rec.recv().unwrap(),
            };
            let cont = match msg {
                ToWorkerMsg::GoIdle => Run::Continue,
                ToWorkerMsg::GoLive => Run::Render,
                ToWorkerMsg::Finish => Run::Stop,
            };
            command = match cont {
                Run::Stop => break,
                Run::Continue => None,
                Run::Render => Some(self.main_loop()),
            }
        }
    }

    pub fn main_loop(&self) -> ToWorkerMsg {
        #[cfg(debug_assertions)]
        println!("Comp {}: start main loop", self.compositor_id);

        // Initial batch of tasks
        // Safety: Subcanvases are accessed based on worker id, no overlap possible
        unsafe {
            let canvases = &self.canvas.sub_canvases[..];

            // relies on compositor ids being continuous, todo fix
            let mut tile_id = (self.compositor_id % self.compositor_count) as usize; // todo maybe for cache reasons do first 1/n tiles instead
            while tile_id < canvases.len() {
                let subcanvas_ptr = canvases[tile_id].get();
                let subcanvas = subcanvas_ptr.as_mut().unwrap();

                // Zero out color and opacity
                subcanvas
                    .colors
                    .iter_mut()
                    .for_each(|v| *v = vector![0.0, 0.0, 0.0]);
                subcanvas.opacities.iter_mut().for_each(|v| *v = 0.0);

                let order = subcanvas.queue.pop_front();
                match order {
                    Some(block_id) => self.send_task(block_id, tile_id, subcanvas_ptr),
                    None => self.tile_finished(subcanvas),
                }
                tile_id += self.compositor_count as usize;
            }
        }

        #[cfg(debug_assertions)]
        println!("Comp {}: initial batch done", self.compositor_id);

        loop {
            // Wait for render task
            let result = select! {
                recv(self.comms.result_rec) -> msg => msg.unwrap(),
                recv(self.comms.command_rec) -> msg => return msg.unwrap(),
            };

            #[cfg(debug_assertions)]
            println!(
                "Comp {}: got result tile {}",
                self.compositor_id, result.tile_id
            );

            // subcanvas is already mutated, colors added

            // Safety: subcanvas queue can be mutated, done tiles counter is under mutex
            unsafe {
                // Can another one be dispatched?

                let subcanvases = &self.canvas.sub_canvases[..];
                let subcanvas_ptr = subcanvases[result.tile_id].get();

                let subcanvas = subcanvas_ptr.as_mut().unwrap();

                match subcanvas.queue.pop_front() {
                    Some(block_id) => self.send_task(block_id, result.tile_id, subcanvas_ptr),
                    None => self.tile_finished(subcanvas),
                }
            }

            // All done?
        }
    }

    // Mark tile and copy it to main buffer
    unsafe fn tile_finished(&self, subcanvas: &mut SubCanvas) {
        // copy color to main canvas
        {
            let mut buffer_g = self.main_buffer.lock().unwrap();
            let buffer = &mut (*buffer_g)[..];

            self.copy_subframe(buffer, subcanvas);
        }

        // Increment canvas counter for finished tiles
        {
            // mutex guard
            let mut remaining_tiles = self.canvas.remaining_subs.lock().unwrap();
            *remaining_tiles -= 1;
            #[cfg(debug_assertions)]
            println!(
                "Comp {}: tile done, remaining {}",
                self.compositor_id, *remaining_tiles
            );
            if *remaining_tiles == 0 {
                // Send message to master, render is done
                #[cfg(debug_assertions)]
                println!("Comp {}: sent render done message", self.compositor_id);

                self.comms.master_sen.send(ToMasterMsg::RenderDone).unwrap();
            }
        }
    }

    fn send_task(&self, block_id: usize, tile_id: usize, subcanvas: *mut SubCanvas) {
        let task = RenderTask::new(block_id, tile_id, subcanvas);
        #[cfg(debug_assertions)]
        println!(
            "Comp {}: sent task block {} tile {}",
            self.compositor_id, block_id, tile_id
        );

        self.comms.task_sen.send(task).unwrap();
    }

    // Copy into main buffer
    fn copy_subframe(&self, buffer: &mut [u8], tile: &mut SubCanvas) {
        let bytes = convert_to_bytes(&tile.colors[..]);
        let data = &bytes[..];

        let full_pixelbox = &self.canvas.size;
        let comp_pixelbox = &tile.pixels;

        let offset = full_pixelbox.offset_in_unchecked(comp_pixelbox) * 3;
        let width = full_pixelbox.width() * 3;
        let subframe_w = comp_pixelbox.width() * 3;

        let mut ptr = offset;
        let mut ptr_data = 0;
        while (ptr_data + subframe_w) <= data.len() {
            buffer[ptr..ptr + subframe_w].copy_from_slice(&data[ptr_data..ptr_data + subframe_w]);

            ptr += width;
            ptr_data += subframe_w;
        }
    }
}

fn convert_to_bytes(subcanvas_rgb: &[Vector3<f32>]) -> Vec<u8> {
    let mut v = Vec::with_capacity(3 * subcanvas_rgb.len());
    subcanvas_rgb.iter().for_each(|rgb| {
        rgb.iter().for_each(|&val| v.push(val as u8));
    });
    v
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn slicing_tiles() {
        let resolution = (700, 700);

        let tile_side = 100;
        let tiles = Canvas::slice_into_tiles(resolution, tile_side);
        assert_eq!(tiles, (7, 7));

        let tile_side = 16;
        let tiles = Canvas::slice_into_tiles(resolution, tile_side);
        assert_eq!(tiles, (44, 44));
    }

    #[test]
    fn affected_tiles() {
        let tile_side = 100;
        let pixel_box = PixelBox::new(0..150, 0..180);
        let tiles = Canvas::get_affected_tiles(pixel_box, tile_side);
        assert_eq!(tiles.0, 0..2);
        assert_eq!(tiles.1, 0..2);

        let tile_side = 16;
        let pixel_box = PixelBox::new(16..33, 15..32);
        let tiles = Canvas::get_affected_tiles(pixel_box, tile_side);
        assert_eq!(tiles.0, 1..3);
        assert_eq!(tiles.1, 0..2);

        let tile_side = 350;
        let pixel_box = PixelBox::new(16..33, 15..32);
        let tiles = Canvas::get_affected_tiles(pixel_box, tile_side);
        assert_eq!(tiles.0, 0..1);
        assert_eq!(tiles.1, 0..1);

        let tile_side = 350;
        let pixel_box = PixelBox::new(300..500, 340..600);
        let tiles = Canvas::get_affected_tiles(pixel_box, tile_side);
        assert_eq!(tiles.0, 0..2);
        assert_eq!(tiles.1, 0..2);
    }
}
