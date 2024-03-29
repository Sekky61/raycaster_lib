/*
    raycaster_lib
    Author: Michal Majer
    Date: 2022-05-05
*/

use std::{
    cell::UnsafeCell,
    cmp::min,
    collections::VecDeque,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};

use crossbeam::select;
use nalgebra::{vector, Vector2, Vector3};
use parking_lot::Mutex;
use render_options::RenderOptions;

use crate::{common::PixelBox, render::render_options, volumetric::Volume, PerspectiveCamera};

use super::{
    communication::CompWorkerComms,
    messages::{RenderTask, ToMasterMsg, ToWorkerMsg},
};

/// Subcanvas, aka. Tile.
///
/// Subcanvas is the target of rendering.
/// It has its own queue of subvolumes visible in it.
pub struct SubCanvas {
    /// Items of queue are indexes into current volume.
    queue: VecDeque<u32>,
    pub pixels: PixelBox,
    pub colors: Vec<Vector3<f32>>,
    pub opacities: Vec<f32>,
}

impl SubCanvas {
    /// Constructs new `SubCanvas` with placement and dimensions of `pixels`.
    /// Buffers are zeroed out and queue is empty.
    pub fn new(pixels: PixelBox) -> Self {
        let size = pixels.items();
        let queue = VecDeque::new();
        let colors = vec![Vector3::zeros(); size as usize];
        let opacities = vec![0.0; size as usize];
        Self {
            queue,
            pixels,
            colors,
            opacities,
        }
    }
}

/// `Canvas` is a collection of `SubCanvas`es.
///
/// # Safety
///
/// Subcanvases are wrapped in [`UnsafeCell`], making their safe use
/// the responsibility of the user.
/// Generally, only one mutable reference at a time is allowed.
pub struct Canvas {
    /// Resolution of canvas
    size: PixelBox,
    /// Collection of `SubCanvas`es.
    sub_canvases: Vec<UnsafeCell<SubCanvas>>,
    /// Atomic counter for detecting end of rendering.
    remaining_subs: AtomicU32,
    /// Width of tile in pixels
    tile_side: u16,
    /// number of tiles in one row
    tiles_x: u16,
}

/// Struct is allowed to be sent across threads.
unsafe impl Sync for Canvas {}

impl Canvas {
    /// Constructs new `Canvas`.
    /// Segments viewport into WxH subframes.
    pub fn new(resolution: Vector2<u16>, tile_side: u16) -> Canvas {
        let tiles = Canvas::slice_into_tiles(resolution, tile_side);
        let size = PixelBox::new(0..resolution.x, 0..resolution.y);

        let items = (tiles.x as usize) * (tiles.y as usize);

        let mut sub_canvases = Vec::with_capacity(items);
        println!(
            "Canvas split into {}x{} = {} tiles",
            tiles.x, tiles.y, items
        );

        for y in 0..tiles.y {
            let low_y = y * tile_side;
            for x in 0..tiles.x {
                let low_x = x * tile_side;
                let high_x = min(low_x + tile_side, resolution.x);
                let high_y = min(low_y + tile_side, resolution.y);
                let pixel_box = PixelBox::new(low_x..high_x, low_y..high_y);

                let sub_canvas = UnsafeCell::new(SubCanvas::new(pixel_box));
                sub_canvases.push(sub_canvas);
            }
        }

        let remaining_subs = sub_canvases.len() as u32;
        Canvas {
            sub_canvases,
            remaining_subs: AtomicU32::new(remaining_subs),
            size,
            tile_side,
            tiles_x: tiles.x,
        }
    }

    /// Calculates number of tiles in each dimension, given `resolution` and `tile_side`.
    fn slice_into_tiles(resolution: Vector2<u16>, tile_side: u16) -> Vector2<u16> {
        (resolution + vector![tile_side - 1, tile_side - 1]) // Ceil
            .component_div(&vector![tile_side, tile_side])
    }

    /// Builds queues of all `SubCanvas`es.
    /// This is done in a few steps:
    /// * Blocks of volume are filtered by visibility.
    /// * Their distance from camera is measured.
    /// * Blocks are sorted in ascending order by their distance from camera.
    /// * For each block, tiles through which block can be seen are found.
    /// * The block is added to the queues of 'affected' tiles.
    ///
    /// # Safety
    ///
    /// uses interior mutability, exclusive access to `Canvas` must be provided.
    pub fn build_queues<V>(
        &self,
        camera: &PerspectiveCamera,
        blocks: &[V],
        empty_blocks: &[bool],
        render_options: RenderOptions,
    ) where
        V: Volume,
    {
        let dont_skip_empty = !render_options.empty_space_skipping;
        let mut block_infos = Vec::with_capacity(blocks.len());
        for (i, (block, empty)) in blocks.iter().zip(empty_blocks).enumerate() {
            if !empty || dont_skip_empty {
                let bbox = &block.get_bound_box();
                let distance = camera.box_distance(bbox);
                block_infos.push((i as u32, distance));
            }
        }

        // todo maybe cache this, until cam dir changes octant
        block_infos.sort_unstable_by(|b1, b2| b1.1.partial_cmp(&b2.1).unwrap()); // todo maybe by cached key

        // Info now sorted by distance

        let res = vector![self.size.width(), self.size.height()];

        // Reset done subcanvases counter
        {
            self.remaining_subs.store(
                self.sub_canvases.len() as u32,
                std::sync::atomic::Ordering::Relaxed,
            );
        }

        for (block_id, _) in block_infos {
            let vpbox = camera.project_box(blocks[block_id as usize].get_bound_box());
            let pixel_box = vpbox.get_pixel_range(res);
            // Count which pixelboxes intersect
            // Assume all tiles are the same size

            let (tiles_x_range, tiles_y_range) =
                Canvas::get_affected_tiles(pixel_box, self.tile_side);

            for y in tiles_y_range {
                for x in tiles_x_range.clone() {
                    let tile_id = self.tiles_x * y + x;

                    // Safety: build phase, only master has access
                    let tile =
                        unsafe { self.sub_canvases[tile_id as usize].get().as_mut().unwrap() };
                    tile.queue.push_back(block_id);
                }
            }
        }
    }

    /// Get overlap of area in which block is visible with tile grid.
    /// Returns 2D range of tile indexes.
    fn get_affected_tiles(
        pixel_box: PixelBox,
        tile_side: u16,
    ) -> (std::ops::Range<u16>, std::ops::Range<u16>) {
        let tile_start_x = pixel_box.x.start / tile_side;
        let tile_end_x = (pixel_box.x.end + tile_side - 1) / tile_side;

        let tile_start_y = pixel_box.y.start / tile_side;
        let tile_end_y = (pixel_box.y.end + tile_side - 1) / tile_side;

        (tile_start_x..tile_end_x, tile_start_y..tile_end_y)
    }
}

/// State of Compositor
enum Run {
    /// Exit loop.
    Stop,
    /// Keep waiting.
    Continue,
    /// Go to active state.
    Render,
}

/// Compositor worker.
/// In bachelors thesis, this is refered to as 'KV' (Kompoziční Vlákno).
///
/// Overview of lifecycle:
/// * Subvolumes are sorted.
/// * Initial batch of rendering tasks are sent to queue.
/// * Passive waiting for incoming completed tasks.
/// * Dispatching new task or copying subcanvas to `main_buffer`.
pub struct CompWorker {
    /// ID of worker thread
    compositor_id: u8,
    /// Number of compositors used in rendering.
    compositor_count: u8,
    /// Shared reference to `Canvas`.
    canvas: Arc<Canvas>,
    /// Final buffer (framebuffer). Subcanvases are copied here.
    main_buffer: Arc<Mutex<Vec<u8>>>,
    /// Interthread communication.
    comms: CompWorkerComms,
}

impl CompWorker {
    /// Construct new `CompWorker`.
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

    /// Main loop (blocking).
    /// Worker is meant to live in a separate thread and wait for commands.
    pub fn run(&self) {
        let mut command = None;
        loop {
            let msg = match command.take() {
                Some(cmd) => cmd,
                None => self.comms.command_rec.recv().unwrap(),
            };
            let cont = match msg {
                ToWorkerMsg::GoIdle => Run::Continue,
                ToWorkerMsg::GoLive { .. } => Run::Render, // Ignore quality setting, irrelevant
                ToWorkerMsg::Finish => Run::Stop,
            };
            command = match cont {
                Run::Stop => break,
                Run::Continue => None,
                Run::Render => Some(self.active_state()),
            }
        }
    }

    /// Rendering routine.
    /// Worker stays in this method for the duration of one frame.
    ///
    /// Returns command that could have been sent to worker during rendering (mainly `Finish` command).
    pub fn active_state(&self) -> ToWorkerMsg {
        #[cfg(debug_assertions)]
        println!("Comp {}: start main loop", self.compositor_id);

        // Initial batch of tasks
        // Safety: Subcanvases are accessed based on worker id, no overlap possible
        unsafe {
            let canvases = &self.canvas.sub_canvases[..];

            // relies on compositor ids being continuous, todo fix
            let mut tile_id = (self.compositor_id % self.compositor_count) as u32; // todo maybe for cache reasons do first 1/n tiles instead
            while (tile_id as usize) < canvases.len() {
                let subcanvas_ptr = canvases[tile_id as usize].get();
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
                tile_id += self.compositor_count as u32;
            }
        }

        #[cfg(debug_assertions)]
        println!("Comp {}: initial batch done", self.compositor_id);

        // Passive part
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
                let subcanvas_ptr = subcanvases[result.tile_id as usize].get();

                let subcanvas = subcanvas_ptr.as_mut().unwrap();

                match subcanvas.queue.pop_front() {
                    Some(block_id) => self.send_task(block_id, result.tile_id, subcanvas_ptr),
                    None => self.tile_finished(subcanvas),
                }
            }

            // All done?
        }
    }

    /// Mark tile as finished and copy its contents to main buffer.
    unsafe fn tile_finished(&self, subcanvas: &mut SubCanvas) {
        // copy color to main canvas
        {
            let mut buffer_g = self.main_buffer.lock();
            let buffer = &mut (*buffer_g)[..];

            self.copy_subframe(buffer, subcanvas);
        }

        // Increment canvas counter for finished tiles
        {
            // mutex guard
            let mut remaining_tiles = self.canvas.remaining_subs.load(Ordering::Relaxed);
            remaining_tiles -= 1;
            self.canvas
                .remaining_subs
                .store(remaining_tiles, Ordering::Relaxed);

            #[cfg(debug_assertions)]
            println!(
                "Comp {}: tile done, remaining {}",
                self.compositor_id, remaining_tiles
            );
            if remaining_tiles == 0 {
                // Render is done
                // Send message to master, render is done
                #[cfg(debug_assertions)]
                println!("Comp {}: sent render done message", self.compositor_id);

                self.comms.master_sen.send(ToMasterMsg::RenderDone).unwrap();
            }
        }
    }

    /// Create task to render block with id `block_id` into tile with id `tile_id`.
    fn send_task(&self, block_id: u32, tile_id: u32, subcanvas: *mut SubCanvas) {
        let task = RenderTask::new(block_id, tile_id, subcanvas);
        #[cfg(debug_assertions)]
        println!(
            "Comp {}: sent task block {} tile {}",
            self.compositor_id, block_id, tile_id
        );

        self.comms.task_sen.send(task).unwrap();
    }

    /// Copy subframe into main buffer.
    fn copy_subframe(&self, buffer: &mut [u8], tile: &mut SubCanvas) {
        let bytes = convert_to_bytes(&tile.colors[..]);
        let data = &bytes[..];

        let full_pixelbox = &self.canvas.size;
        let comp_pixelbox = &tile.pixels;

        let offset = full_pixelbox.offset_in_unchecked(comp_pixelbox) * 3;
        let width = (full_pixelbox.width() * 3) as usize;
        let subframe_w = (comp_pixelbox.width() * 3) as usize;

        let mut ptr = offset as usize;
        let mut ptr_data = 0_usize;
        while (ptr_data + subframe_w) <= data.len() {
            buffer[ptr..ptr + subframe_w].copy_from_slice(&data[ptr_data..ptr_data + subframe_w]);

            ptr += width;
            ptr_data += subframe_w;
        }
    }
}

/// Subcanvas data (floats) gets converted to 1-byte integer values.
fn convert_to_bytes(subcanvas_rgb: &[Vector3<f32>]) -> Vec<u8> {
    // todo optimisation potential - extra allocation
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
        let resolution = vector![700, 700];

        let tile_side = 100;
        let tiles = Canvas::slice_into_tiles(resolution, tile_side);
        assert_eq!(tiles, vector![7, 7]);

        let tile_side = 16;
        let tiles = Canvas::slice_into_tiles(resolution, tile_side);
        assert_eq!(tiles, vector![44, 44]);
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
