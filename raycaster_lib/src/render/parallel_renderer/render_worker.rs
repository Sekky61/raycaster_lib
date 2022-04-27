use std::cell::UnsafeCell;

use crossbeam::select;
use nalgebra::{vector, Vector3};

use crate::{
    common::Ray,
    render::RenderOptions,
    volumetric::{Blocked, Volume},
    PerspectiveCamera,
};

use super::{
    communication::RenderWorkerComms,
    composition::SubCanvas,
    messages::{SubRenderResult, ToWorkerMsg},
};

/// light direction (normalized).
/// Single static light.
const LIGHT_DIR: Vector3<f32> = vector![-0.74278, -0.55708, -0.37139];

/// Worker state.
enum Run {
    Stop,
    Continue,
    Render,
}

/// Renderer worker.
/// In bachelors thesis, this is refered to as 'RV' (Renderovací Vlákno).
///
/// Overview of lifecycle:
/// * Task is received.
/// * Block is rendered into tile.
/// * Compositor is notified of task being done.
pub struct RenderWorker<'a, BV>
where
    BV: Volume + Blocked,
{
    renderer_id: usize,
    camera: &'a UnsafeCell<PerspectiveCamera>,
    sample_step: f32,
    render_options: RenderOptions,
    comms: RenderWorkerComms,
    volume: &'a BV,
}

impl<'a, BV> RenderWorker<'a, BV>
where
    BV: Volume + Blocked,
{
    /// Construct new `RenderWorker`.
    #[must_use]
    pub fn new(
        renderer_id: usize,
        camera: &'a UnsafeCell<PerspectiveCamera>,
        render_options: RenderOptions,
        comms: RenderWorkerComms,
        volume: &'a BV,
    ) -> Self {
        Self {
            renderer_id,
            camera,
            sample_step: 0.2, // Default, gets overridden
            render_options,
            comms,
            volume,
        }
    }

    /// Main loop.
    pub fn run(&mut self) {
        let mut command = None;
        loop {
            let msg = match command.take() {
                Some(cmd) => cmd,
                None => self.comms.command_rec.recv().unwrap(),
            };
            let cont = match msg {
                ToWorkerMsg::GoIdle => Run::Continue,
                ToWorkerMsg::GoLive { sample_step } => {
                    self.sample_step = sample_step;
                    Run::Render
                }
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
    fn active_state(&self) -> ToWorkerMsg {
        #[cfg(debug_assertions)]
        println!("Render {}: entering main loop", self.renderer_id);

        let blocks = self.volume.get_blocks();

        let cam_ref = unsafe { self.camera.get().as_ref().unwrap() };

        loop {
            // Wait for task from master thread or finish call
            let task = select! {
                recv(self.comms.task_rec) -> msg => msg.unwrap(),
                recv(self.comms.command_rec) -> msg => return msg.unwrap(),
            };

            let block_id = task.block_id;

            #[cfg(debug_assertions)]
            println!("Render {}: got task block id {block_id}", self.renderer_id);

            // Safety: ref is unique
            let subcanvas = unsafe { task.subcanvas.as_mut().unwrap() };

            let block = &blocks[block_id as usize];

            // Render task
            self.render_block(cam_ref, subcanvas, block);
            // Opacities have been mutated

            #[cfg(debug_assertions)]
            println!("Render {}: rendered {block_id}", self.renderer_id);
            let subrender_res = SubRenderResult::new(task.tile_id);
            self.comms.result_sen.send(subrender_res).unwrap();

            #[cfg(debug_assertions)]
            println!("Render {}: sent back block {block_id}", self.renderer_id);
        }
    }

    /// Render block into `subcanvas`.
    fn render_block(
        &self,
        camera: &PerspectiveCamera,
        subcanvas: &mut SubCanvas,
        block: &<BV as Blocked>::BlockType,
    ) {
        // todo use renderoptions properly
        // Image size, todo move to property
        let res_f = self.render_options.resolution.map(|v| v as f32); // todo cast everywhere
        let step_f = res_f.map(|v| 1.0 / v);

        // todo waiting for opacities can be done here, render and send back immediately
        // flatten skips Nones
        let opacities = &mut subcanvas.opacities[..];

        let x_range = subcanvas.pixels.x.clone();
        let y_range = subcanvas.pixels.y.clone();

        let color_buf = &mut subcanvas.colors[..];
        let mut ptr = 0;

        for y in y_range {
            let y_norm = y as f32 * step_f.y;
            for x in x_range.clone() {
                let pixel_coord = (x as f32 * step_f.x, y_norm);
                let ray = camera.get_ray(pixel_coord);

                // Early opacity check
                if self.render_options.early_ray_termination && opacities[ptr] > 0.99 {
                    ptr += 1;
                    continue;
                }

                // Adds to opacity buffer
                let color = self.sample_color(block, &ray, camera, &mut opacities[ptr]);

                // TODO multiply color with opacity ??
                // TODO results seem ok

                color_buf[ptr] += color;

                ptr += 1;
            }
        }
    }

    /// Accumulation of color along `ray`.
    fn sample_color(
        &self,
        block: &<BV as Blocked>::BlockType,
        ray: &Ray,
        camera: &PerspectiveCamera,
        opacity: &mut f32,
    ) -> Vector3<f32> {
        let mut accum = vector![0.0, 0.0, 0.0];

        let obj_ray = block.transform_ray(ray);

        let view_dir_neg = -camera.get_dir();

        let (obj_ray, t) = match obj_ray {
            Some(r) => r,
            None => return accum,
        };

        let max_n_of_steps = (t / self.sample_step) as usize;

        let step = obj_ray.direction * self.sample_step; // normalized

        let mut pos = obj_ray.origin;

        let tf = self.volume.get_tf();

        // Source:
        // https://developer.nvidia.com/gpugems/gpugems/part-vi-beyond-triangles/chapter-39-volume-rendering-techniques
        // Equation 3
        //
        // reference_step_length / new_step_length
        let step_ratio = self.sample_step;

        for _ in 0..max_n_of_steps {
            //let sample = self.volume.sample_at(pos);
            if self.render_options.early_ray_termination && *opacity > 0.99 {
                break;
            }

            let (sample, grad_samples) = block.sample_at_gradient(pos);

            pos += step;

            let color_b = tf(sample);
            if color_b.w == 0.0 {
                continue;
            }

            // Inverted, as low values indicate outside
            let grad = vector![
                sample - grad_samples.x,
                sample - grad_samples.y,
                sample - grad_samples.z
            ];

            let grad_magnitude = grad.magnitude();
            const GRAD_MAG_THRESH: f32 = 10.0; // todo tweak

            let mut sample_rgb = color_b.xyz();

            if grad_magnitude > GRAD_MAG_THRESH {
                let grad_norm = grad / grad_magnitude;
                let diffuse = f32::max(grad_norm.dot(&-LIGHT_DIR), 0.00); // ambient light 0.09

                let reflect = LIGHT_DIR - 2.0 * (grad_norm.dot(&LIGHT_DIR)) * grad_norm;
                let r_dot_view = reflect.dot(&view_dir_neg);
                let light_intensity = 120.0;
                let specular = f32::max(0.0, r_dot_view).powf(128.0) * light_intensity;

                sample_rgb = sample_rgb * (diffuse + 0.16) + vector![specular, specular, specular];
            }

            // pseudocode from https://scholarworks.rit.edu/cgi/viewcontent.cgi?article=6466&context=theses page 55, figure 5.6
            //sum = (1 - sum.alpha) * volume.density * color + sum;

            let opacity_corrected = 1.0 - (1.0 - color_b.w).powf(step_ratio);

            accum += (1.0 - *opacity) * opacity_corrected * sample_rgb;

            *opacity += (1.0 - *opacity) * opacity_corrected;
        }

        accum
    }
}
