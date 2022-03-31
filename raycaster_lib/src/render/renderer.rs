use std::cmp::min;

use nalgebra::{vector, Vector4};

use crate::{
    common::Ray,
    volumetric::{BlockType, EmptyIndex, Volume},
    PerspectiveCamera,
};

use super::RenderOptions;

pub struct Renderer<V>
where
    V: Volume + 'static,
{
    pub volume: V,
    pub empty_index: EmptyIndex<3>,
    render_options: RenderOptions,
}

impl<V> Renderer<V>
where
    V: Volume,
{
    pub fn new(volume: V, render_options: RenderOptions) -> Renderer<V> {
        let empty_index = EmptyIndex::from_volume(&volume);
        Renderer {
            volume,
            empty_index,
            render_options,
        }
    }

    pub fn set_render_options(&mut self, opts: RenderOptions) {
        self.render_options = opts;
    }

    pub fn set_render_resolution(&mut self, res: (usize, usize)) {
        self.render_options.resolution = res;
    }

    pub fn render_to_buffer(&mut self, camera: &PerspectiveCamera, buffer: &mut [u8]) {
        self.render(camera, buffer);
    }

    // buffer y=0 is up
    fn render(&mut self, camera: &PerspectiveCamera, buffer: &mut [u8]) {
        let (img_w, img_h) = self.render_options.resolution;

        let (image_width, image_height) = (img_w as f32, img_h as f32);

        // Clear
        for byte in buffer.iter_mut() {
            *byte = 0;
        }

        let step_x = 1.0 / image_width;
        let step_y = 1.0 / image_height;

        let bbox = self.volume.get_bound_box();
        let tile = camera.project_box(bbox);

        let res_vec = vector![
            // todo change resolution from tuple to vector or pixelbox
            self.render_options.resolution.0,
            self.render_options.resolution.1
        ];
        let pixels = tile.get_pixel_range(res_vec);
        let pix_width = pixels.width();

        let width_bytes_skip = 3 * (img_w - pix_width);
        let mut index = (pixels.x.start + img_w * pixels.y.start) * 3;

        for y in pixels.y.clone() {
            let y_norm = y as f32 * step_y;
            for x in pixels.x.clone() {
                let pixel_coord = (x as f32 * step_x, y_norm);
                let ray = camera.get_ray(pixel_coord);

                // performance: branch gets almost optimized away since it is predictable
                let ray_color = if self.render_options.empty_index {
                    self.collect_light_index(&ray)
                } else {
                    self.collect_light(&ray)
                };

                let opacity = ray_color.w;

                // if x == pixels.x.start
                //     || x == pixels.x.end - 1
                //     || y == pixels.y.end - 1
                //     || y == pixels.y.start
                // {
                //     buffer[index] = 255;
                //     buffer[index + 1] = 255;
                //     buffer[index + 2] = 255;
                //     index += 3;
                //     continue;
                // }

                // expects black background
                buffer[index] = (ray_color.x * opacity) as u8;
                buffer[index + 1] = (ray_color.y * opacity) as u8;
                buffer[index + 2] = (ray_color.z * opacity) as u8;
                index += 3;
            }
            index += width_bytes_skip;
        }
    }

    pub fn collect_light(&self, ray: &Ray) -> Vector4<f32> {
        let mut accum = vector![0.0, 0.0, 0.0, 0.0];

        let (obj_ray, t) = match self.volume.intersect_transform(ray) {
            Some(e) => e,
            None => return accum,
        };

        let begin = obj_ray.origin;
        let direction = ray.get_direction();

        let step_size = 1.0;
        let max_n_of_steps = (t / step_size) as usize;

        let step = direction * step_size; // normalized

        let mut pos = begin;

        let tf = self.volume.get_tf();
        let light_source = vector![1.0, 1.0, 0.0].normalize();

        for _ in 0..max_n_of_steps {
            //let sample = self.volume.sample_at(pos);

            let (sample, grad_samples) = self.volume.sample_at_gradient(pos);

            let color_b = tf(sample);

            let grad = vector![
                sample - grad_samples.x,
                sample - grad_samples.y,
                sample - grad_samples.z
            ];

            let grad = grad.normalize();

            let n_dot_l = f32::max(grad.dot(&light_source), 0.0);
            let rgb = color_b.xyz() * n_dot_l;

            pos += step;

            if color_b.w == 0.0 {
                continue;
            }

            // pseudocode from https://scholarworks.rit.edu/cgi/viewcontent.cgi?article=6466&context=theses page 55, figure 5.6
            //sum = (1 - sum.alpha) * volume.density * color + sum;

            accum += (1.0 - accum.w) * vector![rgb.x, rgb.y, rgb.z, color_b.w]; // todo dont scale W

            // relying on branch predictor to "eliminate" branch
            if self.render_options.ray_termination {
                // early ray termination
                if color_b.w > 0.99 {
                    break;
                }
            }
        }
        accum
    }

    pub fn collect_light_index(&self, ray: &Ray) -> Vector4<f32> {
        let mut accum = vector![0.0, 0.0, 0.0, 0.0];

        let (obj_ray, t) = match self.volume.intersect_transform(ray) {
            Some(e) => e,
            None => return accum,
        };

        let begin = obj_ray.origin;
        let direction = ray.get_direction();

        let step_size = 1.0;
        let max_n_of_steps = (t / step_size) as usize;

        let step = direction * step_size; // normalized

        let mut pos = begin;

        let tf = self.volume.get_tf();
        let light_source = vector![1.0, 1.0, 0.0].normalize();

        for _ in 0..max_n_of_steps {
            // todo try sampling on integer coords
            if self.empty_index.sample(pos) == BlockType::Empty {
                pos += step;
                continue;
            }

            //let sample = self.volume.sample_at(pos);

            let (sample, grad_samples) = self.volume.sample_at_gradient(pos);

            let color_b = tf(sample);

            let grad = vector![
                sample - grad_samples.x,
                sample - grad_samples.y,
                sample - grad_samples.z
            ];

            let grad = grad.normalize();

            let n_dot_l = f32::max(grad.dot(&light_source), 0.0);
            let rgb = color_b.xyz() * n_dot_l;

            pos += step;

            if color_b.w == 0.0 {
                continue;
            }

            // pseudocode from https://scholarworks.rit.edu/cgi/viewcontent.cgi?article=6466&context=theses page 55, figure 5.6
            //sum = (1 - sum.alpha) * volume.density * color + sum;

            accum += (1.0 - accum.w) * vector![rgb.x, rgb.y, rgb.z, color_b.w]; // todo dont scale W

            // relying on branch predictor to "eliminate" branch
            if self.render_options.ray_termination {
                // early ray termination
                if color_b.w > 0.99 {
                    break;
                }
            }
        }
        accum
    }
}
