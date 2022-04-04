use std::{
    cell::UnsafeCell,
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use crossbeam::select;
use nalgebra::Vector3;

use crate::{common::PixelBox, PerspectiveCamera};

use super::{
    communication::CompWorkerComms,
    messages::{RenderTask, SubFrameResult, SubRenderResult, ToMasterMsg, ToWorkerMsg},
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
}

unsafe impl Sync for Canvas {}

impl Canvas {
    // Segment viewport into NxN subframes
    // Calc pixel sizes and offsets
    // todo return only PixelBox
    pub fn new(resolution: (usize, usize), tile_side: usize) -> Canvas {
        let tiles_x = (resolution.0 + tile_side - 1) / tile_side; // ceil
        let tiles_y = (resolution.1 + tile_side - 1) / tile_side; // ceil
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
                let pixel_box =
                    PixelBox::new(low_x..(low_x + tile_side), low_y..(low_y + tile_side));

                let sub_canvas = UnsafeCell::new(SubCanvas::new(pixel_box));
                sub_canvases.push(sub_canvas);
            }
        }
        Canvas {
            sub_canvases,
            remaining_subs: Mutex::new(0),
            size,
        }
    }

    pub fn build_queues(&mut self, camera: &PerspectiveCamera) {
        todo!();
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
                ToWorkerMsg::StopRendering => Run::Continue,
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
        // Initial batch of tasks
        // Safety: Subcanvases are accessed based on worker id, no overlap possible
        unsafe {
            let canvases = &self.canvas.sub_canvases[..];

            let mut tile_id = self.compositor_id as usize; // todo maybe for cache reasons do first 1/n tiles instead
            while tile_id < canvases.len() {
                let subcanvas_ptr = canvases[tile_id].get();
                let subcanvas = subcanvas_ptr.as_mut().unwrap();
                let order = subcanvas.queue.pop_front();
                match order {
                    Some(block_id) => self.send_task(block_id, tile_id, subcanvas_ptr),
                    None => self.tile_finished(subcanvas),
                }
                tile_id += self.compositor_count as usize;
            }
        }

        loop {
            // Wait for render task
            let result = select! {
                recv(self.comms.result_rec) -> msg => msg.unwrap(),
                recv(self.comms.command_rec) -> msg => return msg.unwrap(),
            };

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
            if *remaining_tiles == 0 {
                // Send message to master, render is done
                self.comms.master_sen.send(ToMasterMsg::RenderDone).unwrap();
            }
        }
    }

    fn send_task(&self, block_id: usize, tile_id: usize, subcanvas: *mut SubCanvas) {
        let task = RenderTask::new(block_id, tile_id, subcanvas);
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