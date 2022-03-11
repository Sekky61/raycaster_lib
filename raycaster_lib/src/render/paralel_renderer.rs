use std::{cmp::min, collections::BinaryHeap, ops::Range, thread::JoinHandle};

use crossbeam_channel::{Receiver, Sender};
use nalgebra::{Vector3, Vector4};
use rayon::ThreadPool;

use crate::{
    camera::{Camera, PerspectiveCamera},
    common::{Ray, ViewportBox},
    volumetric::{Block, BlockVolume},
};

use super::RenderOptions;

pub struct OpacityDataRequest {
    order: usize, // distance from the camera
    pixel_range: (Range<usize>, Range<usize>),
}

pub enum RenderWorkerMessage {
    Finish,
}

pub struct SubRenderResult {
    width: usize,
    colors: Vec<Vector3<f32>>,
    opacities: Vec<f32>,
}

pub struct RenderThreadHandle {
    pub handle: JoinHandle<()>,
    pub sender: Sender<RenderWorkerMessage>,
    pub receiver: Receiver<RenderWorkerMessage>,
}

impl RenderThreadHandle {
    pub fn new(
        handle: JoinHandle<()>,
        sender: Sender<RenderWorkerMessage>,
        receiver: Receiver<RenderWorkerMessage>,
    ) -> Self {
        Self {
            handle,
            sender,
            receiver,
        }
    }

    pub fn join(self) {
        self.handle.join().unwrap()
    }
}

pub struct ParalelRenderer {
    volume: BlockVolume,
    camera: PerspectiveCamera,
    render_options: RenderOptions,
    compositors: [JoinHandle<()>; 4],
    renderers: [RenderThreadHandle; 4],
}

impl ParalelRenderer {
    pub fn new(
        volume: BlockVolume,
        camera: PerspectiveCamera,
        render_options: RenderOptions,
    ) -> ParalelRenderer {
        // Send to compositor, compositor recieves message
        let ren_to_comp = [
            crossbeam_channel::unbounded(),
            crossbeam_channel::unbounded(),
            crossbeam_channel::unbounded(),
            crossbeam_channel::unbounded(),
        ];
        let compositor_send = [
            ren_to_comp[0].0,
            ren_to_comp[1].0,
            ren_to_comp[2].0,
            ren_to_comp[3].0,
        ];
        // Send to renderer, renderer recieves message
        let comp_to_ren = [
            crossbeam_channel::unbounded(),
            crossbeam_channel::unbounded(),
            crossbeam_channel::unbounded(),
            crossbeam_channel::unbounded(),
        ];

        for i in 0..4 {
            // Create render thread
            // Move these to thread
            let ren_to_comp_send = ren_to_comp[i].0;
            let comp_to_ren_recv = comp_to_ren[i].1;

            // Channel associated with this renderer
            let (sender, receiver) = comp_to_ren[i];
            let x = RenderThreadHandle::new(
                std::thread::spawn(move || {
                    println!("deez {i}");
                    let all_compositors = ren_to_comp;
                }),
                sender,
                receiver,
            );
        }

        ParalelRenderer {
            volume,
            camera,
            render_options,
            compositors,
            renderers,
            channel,
        }
    }

    pub fn render(&mut self, buffer: &mut [u8]) {
        let resolution = self.render_options.resolution;

        // get subvolume distances
        let mut block_order = Vec::with_capacity(self.volume.data.len());
        for (id, block) in self.volume.data.iter().enumerate() {
            let distance = self.camera.box_distance(&block.bound_box);
            block_order.push((id, distance));
        }
        block_order.sort_unstable_by(|i1, i2| i1.1.partial_cmp(&i2.1).unwrap());

        // New canvas
        let num_of_pixels = resolution.0 * resolution.1;
        let color_canvas = vec![Vector3::<f32>::zeros(); num_of_pixels];
        let opacity_canvas = vec![0.0f32; num_of_pixels];

        // Divide canvas
        let compositers_count = self.compositors.current_num_threads();
        assert_eq!(compositers_count, 4);
        let halfpoint = (resolution.0 / 2, resolution.1 / 2);

        // Send rendering tasks
        for (block_id, distance) in block_order {
            let block = &self.volume.data[block_id];
            self.renderers.install(move || {
                let block_ref = block;
            })
        }
    }

    fn render_block(&mut self, block: &Block) -> SubRenderResult {
        // get viewport box
        let vpb = self.camera.project_box(block.bound_box);

        // Image size, todo move to property
        let (img_w, img_h) = self.render_options.resolution;
        let (image_width, image_height) = (img_w as f32, img_h as f32);
        let step_x = 1.0 / image_width;
        let step_y = 1.0 / image_height;

        let (x_range, y_range) = self.get_pixel_range(vpb);

        // Request opacity data

        for y in y_range {
            let y_norm = y as f32 * step_y;
            for x in x_range.clone() {
                // todo clone here -- maybe use own impl
                let pixel_coord = (x as f32 * step_x, y_norm);
                let ray = self.camera.get_ray(pixel_coord);

                let (color, opacity) = self.sample_color(block, ray);

                colors.push(color);
                opacities.push(opacity);

                // Add to opacity buffer
            }
        }

        SubRenderResult { colors, opacities }
    }

    fn get_pixel_range(&self, tile: ViewportBox) -> (Range<usize>, Range<usize>) {
        let (width, height) = self.render_options.resolution;
        let width_f = width as f32;
        let height_f = height as f32;

        let mut tile_pixel_size = tile.size();
        tile_pixel_size.x = f32::ceil(tile_pixel_size.x * width_f);
        tile_pixel_size.y = f32::ceil(tile_pixel_size.y * height_f);

        let mut start_pixel = tile.lower;
        start_pixel.x = f32::floor(start_pixel.x * width_f);
        start_pixel.y = f32::floor(start_pixel.y * height_f);

        let start_x = start_pixel.x as usize;
        let start_y = start_pixel.y as usize;

        let lim_x = tile_pixel_size.x as usize;
        let lim_y = tile_pixel_size.y as usize;

        let end_x = min(start_x + lim_x, width);
        let end_y = min(start_y + lim_y, height);

        (start_x..end_x, start_y..end_y)
    }

    fn sample_color(&self, block: &Block, ray: Ray) -> (Vector3<f32>, f32) {
        todo!()
    }
}
