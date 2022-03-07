use std::cmp::min;

use nalgebra::{point, vector, Point3, Vector3, Vector4};

use crate::{
    camera::Camera,
    ray::Ray,
    volumetric::{BlockType, EmptyIndex, Volume},
};

#[derive(Default)]
pub struct RenderOptions {
    pub resolution: (usize, usize),
    pub ray_termination: bool,
    pub empty_index: bool,
}

impl RenderOptions {
    pub fn new(
        resolution: (usize, usize),
        ray_termination: bool,
        empty_index: bool,
    ) -> RenderOptions {
        RenderOptions {
            resolution,
            ray_termination,
            empty_index,
        }
    }
}

pub struct Renderer<V, C>
where
    V: Volume,
    C: Camera,
{
    pub volume: V,
    pub camera: C,
    pub empty_index: EmptyIndex<3>,
    render_options: RenderOptions,
}

impl<V, C> Renderer<V, C>
where
    V: Volume,
    C: Camera,
{
    pub fn new(volume: V, camera: C) -> Renderer<V, C> {
        let empty_index = EmptyIndex::from_volume(&volume);
        Renderer {
            volume,
            camera,
            empty_index,
            render_options: RenderOptions {
                resolution: (100, 100), //todo
                ray_termination: true,
                empty_index: true,
            },
        }
    }

    pub fn set_render_options(&mut self, opts: RenderOptions) {
        self.render_options = opts;
    }

    pub fn set_render_resolution(&mut self, res: (usize, usize)) {
        self.render_options.resolution = res;
    }

    pub fn render_to_buffer(&mut self, buffer: &mut [u8]) {
        self.render(buffer);
    }

    // buffer y=0 is up
    fn render(&mut self, buffer: &mut [u8]) {
        let (img_w, img_h) = self.render_options.resolution;

        let (image_width, image_height) = (img_w as f32, img_h as f32);

        let step_x = 1.0 / image_width;
        let step_y = 1.0 / image_height;

        let bbox = self.volume.get_bound_box();
        let tile = self.camera.project_box(bbox);

        let mut tile_pixel_size = tile.size();
        tile_pixel_size.x = f32::ceil(tile_pixel_size.x * image_width);
        tile_pixel_size.y = f32::ceil(tile_pixel_size.y * image_height);

        let mut start_pixel = tile.lower;
        start_pixel.x = f32::floor(start_pixel.x * image_width);
        start_pixel.y = f32::floor(start_pixel.y * image_height);

        let start_x = start_pixel.x as usize;
        let start_y = start_pixel.y as usize;

        let lim_x = tile_pixel_size.x as usize;
        let lim_y = tile_pixel_size.y as usize;

        let end_x = min(start_x + lim_x, img_w);
        let end_y = min(start_y + lim_y, img_h);

        for y in (start_y..end_y).rev() {
            let y_norm = y as f32 * step_y;
            for x in start_x..end_x {
                let pixel_coord = (x as f32 * step_x, y_norm);
                let ray = self.camera.get_ray(pixel_coord);

                // performance: branch gets almost optimized away since it is predictable
                let ray_color = if self.render_options.empty_index {
                    self.collect_light_index(&ray)
                } else {
                    self.collect_light(&ray)
                };

                let opacity = ray_color.w;

                let index = (x + img_w * y) * 3;

                // expects black background
                buffer[index] = (ray_color.x * opacity) as u8;
                buffer[index + 1] = (ray_color.y * opacity) as u8;
                buffer[index + 2] = (ray_color.z * opacity) as u8;
            }
        }
    }

    pub fn collect_light(&self, ray: &Ray) -> Vector4<f32> {
        let mut accum = vector![0.0, 0.0, 0.0, 0.0];

        let (t1, _) = match self.volume.intersect(ray) {
            Some(tup) => tup,
            None => return accum,
        };

        let begin = ray.point_from_t(t1);
        let direction = ray.get_direction();

        let step_size = 1.0;

        let step = direction * step_size; // normalized

        let mut pos = begin;

        let tf = self.volume.get_tf();

        loop {
            let sample = self.volume.sample_at(pos);

            let color = tf(sample);

            pos += step;

            if color.w == 0.0 {
                if !self.volume.is_in(&pos) {
                    break;
                }
                continue;
            }

            // pseudocode from https://scholarworks.rit.edu/cgi/viewcontent.cgi?article=6466&context=theses page 55, figure 5.6
            //sum = (1 - sum.alpha) * volume.density * color + sum;

            accum += (1.0 - accum.w) * color;

            // relying on branch predictor to "eliminate" branch
            if self.render_options.ray_termination {
                // early ray termination
                if (color.w - 0.99) > 0.0 {
                    break;
                }
            }

            if !self.volume.is_in(&pos) {
                break;
            }
        }

        accum
    }

    pub fn collect_light_index(&self, ray: &Ray) -> Vector4<f32> {
        let mut accum = vector![0.0, 0.0, 0.0, 0.0];

        let (t1, _) = match self.volume.intersect(ray) {
            Some(tup) => tup,
            None => return accum,
        };

        let begin = ray.point_from_t(t1);
        let direction = ray.get_direction();

        let step_size = 1.0;

        let step = direction * step_size; // normalized

        let mut pos = begin;

        let tf = self.volume.get_tf();

        loop {
            if self.empty_index.sample(pos) == BlockType::Empty {
                pos += step;

                if !self.volume.is_in(&pos) {
                    break;
                }
                continue;
            }

            //let sample = self.volume.sample_at(pos);

            let (sample, grad_samples) = self.volume.sample_at_gradient(pos);

            let light_source = vector![1.0, 1.0, 0.0].normalize();

            let color_b = tf(sample);

            let grad = vector![
                sample - grad_samples.x,
                sample - grad_samples.y,
                sample - grad_samples.z
            ];

            let grad = grad.normalize();

            let n_dot_l = f32::max(grad.dot(&light_source), 0.0);
            let rgb = color_b.xyz() * n_dot_l;

            // if color_b.w > 0.0 {
            //     println!("color {color_b} -> {rgb} grad {grad}");
            // }

            pos += step;

            if color_b.w == 0.0 {
                if !self.volume.is_in(&pos) {
                    break;
                }
                continue;
            }

            // pseudocode from https://scholarworks.rit.edu/cgi/viewcontent.cgi?article=6466&context=theses page 55, figure 5.6
            //sum = (1 - sum.alpha) * volume.density * color + sum;

            accum += (1.0 - accum.w) * vector![rgb.x, rgb.y, rgb.z, color_b.w];

            // relying on branch predictor to "eliminate" branch
            if self.render_options.ray_termination {
                // early ray termination
                if color_b.w > 0.99 {
                    break;
                }
            }

            if !self.volume.is_in(&pos) {
                break;
            }
        }

        accum
    }
}
