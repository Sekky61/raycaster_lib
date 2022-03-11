use std::{cmp::min, collections::BinaryHeap, ops::Range, thread::JoinHandle};

use crossbeam::thread::{Scope, ScopedJoinHandle};
use crossbeam_channel::{Receiver, Sender};
use nalgebra::{point, vector, Vector3, Vector4};
use rayon::ThreadPool;

use crate::{
    camera::{Camera, PerspectiveCamera},
    common::{Ray, ViewportBox},
    volumetric::{Block, BlockVolume},
};

use super::RenderOptions;

pub struct OpacityRequest {
    order: usize, // distance from the camera
    pixel_range: (Range<usize>, Range<usize>),
}

pub struct SubRenderResult {
    width: usize,
    colors: Vec<Vector3<f32>>,
    opacities: Vec<f32>,
}

pub struct OpacityData {
    start_pixel: usize, // offset of lowest pixel
    width: usize,
    opacities: Vec<f32>,
}

pub enum ToCompositorMsg {
    OpacityRequest(OpacityRequest),
    RenderResult(SubRenderResult),
    Finish,
}

pub struct ToRendererMsg {
    opacity: OpacityData,
}

pub struct ParalelRenderer {
    volume: BlockVolume,
    camera: PerspectiveCamera,
    render_options: RenderOptions,
}

impl ParalelRenderer {
    pub fn new(
        volume: BlockVolume,
        camera: PerspectiveCamera,
        render_options: RenderOptions,
    ) -> ParalelRenderer {
        ParalelRenderer {
            volume,
            camera,
            render_options,
        }
    }

    pub fn start_rendering(mut self) -> JoinHandle<()> {
        std::thread::spawn(move || {
            // Scope assures threads will be joined before exiting the scope
            crossbeam::scope(|s| {
                let volume = &self.volume;

                // inlined function because borrow checker defeated me (scope cannot leave closure)
                let (render_handles, comp_handles) = {
                    // Send to compositor, compositor recieves message
                    let ren_to_comp = [
                        crossbeam_channel::unbounded(),
                        crossbeam_channel::unbounded(),
                        crossbeam_channel::unbounded(),
                        crossbeam_channel::unbounded(),
                    ];
                    let compositor_send: [Sender<ToCompositorMsg>; 4] = [
                        ren_to_comp[0].0.clone(),
                        ren_to_comp[1].0.clone(),
                        ren_to_comp[2].0.clone(),
                        ren_to_comp[3].0.clone(),
                    ];
                    // Send to renderer, renderer recieves message
                    let comp_to_ren = [
                        crossbeam_channel::unbounded(),
                        crossbeam_channel::unbounded(),
                        crossbeam_channel::unbounded(),
                        crossbeam_channel::unbounded(),
                    ];
                    let renderer_send: [Sender<ToRendererMsg>; 4] = [
                        comp_to_ren[0].0.clone(),
                        comp_to_ren[1].0.clone(),
                        comp_to_ren[2].0.clone(),
                        comp_to_ren[3].0.clone(),
                    ];

                    let mut renderers = Vec::with_capacity(4);
                    let mut compositors = Vec::with_capacity(4);

                    for i in 0..4 {
                        // Create render thread
                        let receiver = comp_to_ren[i].1.clone(); // Receiver
                        let all_compositors = compositor_send.clone();
                        let blocks_ref = &volume.data[..];
                        let handle = s.spawn(move |_| {
                            println!("Started renderer {i}");
                            // Force move into closure
                            let renderer_id = i;
                            let all_compositors = all_compositors; // Senders for all compositors
                            let message_receiver = receiver;
                            let blocks_ref = blocks_ref;

                            loop {

                                // Wait for task from master thread or finish call

                                // Get data from compositers

                                // Render task

                                // give data to compositers
                            }
                        });

                        renderers.push(handle);
                    }

                    for i in 0..4 {
                        // Create compositor thread
                        let receiver = ren_to_comp[i].1.clone(); // Receiver
                        let all_renderers = renderer_send.clone();
                        let blocks_ref = &volume.data[..];
                        let handle = s.spawn(move |_| {
                            println!("Started compositor {i}");
                            // Force move into closure
                            let compositor_id = i;
                            let all_renderers = all_renderers; // Senders for all compositors
                            let message_receiver = receiver;
                            let blocks_ref = blocks_ref;

                            // Subcanvas
                            let subcanvas_size = (0, 0);
                            let subcanvas_items = subcanvas_size.0 * subcanvas_size.1;
                            let subcanvas_rgb = vec![Vector3::<f32>::zeros(); subcanvas_items]; // todo RGB
                            let subcanvas_opacity = vec![0.0; subcanvas_items];

                            // Calculate which subvolumes appear in my subcanvas
                            // Also calculate expected order of subvolumes

                            loop {
                                // Receive requests

                                // Send opacity / store subrender / finish

                                // Finally convert to RGB bytes and send to master thread for output

                                // Wait for wakeup call or finish call
                            }
                        });

                        compositors.push(handle);
                    }

                    (renderers, compositors)
                };

                let (width, height) = self.render_options.resolution;

                loop {
                    // Gather input

                    // Render
                    let mut buffer = vec![0u8; 3 * width * height];
                    self.render(&mut buffer[..]);

                    // Send result
                }
            })
            .unwrap();
        })
    }

    pub fn render(&self, buffer: &mut [u8]) {
        let resolution = self.render_options.resolution;

        // get subvolume distances
        let mut block_order = Vec::with_capacity(self.volume.data.len());
        for (id, block) in self.volume.data.iter().enumerate() {
            let distance = self.camera.box_distance(&block.bound_box);
            block_order.push((id, distance));
        }
        block_order.sort_unstable_by(|i1, i2| i1.1.partial_cmp(&i2.1).unwrap());

        // Send rendering tasks
        //
        // Sent in order of camera distance (asc)
        // for Load balancing
        for (block_id, distance) in block_order {
            let block = &self.volume.data[block_id];

            // Find out if block is empty
        }

        // Get subcanvases from compositors and save them to buffer
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
        let mut colors = vec![];
        let mut opacities = vec![];

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
        let width = x_range.end - x_range.start;
        SubRenderResult {
            width,
            colors,
            opacities,
        }
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
