use std::cmp;

use nalgebra::{max, point, vector, Point3, Vector3, Vector4};

use crate::{
    camera::Camera,
    ray::Ray,
    transfer_functions::skull_tf,
    volumetric::{BlockType, EmptyIndex, Volume},
};

#[derive(Default)]
pub struct RenderOptions {
    pub ray_termination: bool,
    pub empty_index: bool,
    pub multi_thread: bool,
}

impl RenderOptions {
    pub fn new(ray_termination: bool, empty_index: bool, multi_thread: bool) -> Self {
        Self {
            ray_termination,
            empty_index,
            multi_thread,
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
    pub empty_index: EmptyIndex<2>,
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
                ray_termination: true,
                empty_index: false,
                multi_thread: false,
            },
        }
    }

    pub fn set_render_options(&mut self, opts: RenderOptions) {
        self.render_options = opts;
    }

    pub fn render_to_buffer(&mut self, buffer: &mut [u8]) {
        self.render(buffer);
    }

    fn render(&mut self, buffer: &mut [u8]) {
        let (img_w, img_h) = self.camera.get_resolution();

        let (image_width, image_height) = (img_w as f32, img_h as f32);

        let origin_4 = self.camera.get_position().to_homogeneous();

        let aspect_ratio = image_width / image_height;

        // cam to world
        let lookat_matrix = self.camera.view_matrix();

        let mut buffer_index = 0;

        for y in 0..img_h {
            for x in 0..img_w {
                let pixel_ndc_x = (x as f32 + 0.5) / image_width;
                let pixel_ndc_y = (y as f32 + 0.5) / image_height;

                let pixel_screen_x = (pixel_ndc_x * 2.0 - 1.0) * aspect_ratio;
                let pixel_screen_y = 1.0 - pixel_ndc_y * 2.0; // v NDC Y roste dolu, obratime

                //todo FOV

                let pix_cam_space = vector![pixel_screen_x, pixel_screen_y, -1.0, 1.0];

                let dir_world = (lookat_matrix * pix_cam_space) - origin_4;
                let dir_world_3 = dir_world.xyz().normalize();

                let ray_world = Ray::from_3(self.camera.get_position(), dir_world_3);

                // performance: branch gets almost optimized away since it is predictable
                let ray_color = if self.render_options.empty_index {
                    self.collect_light_index(&ray_world)
                } else {
                    self.collect_light(&ray_world)
                };

                let opacity = ray_color.w;

                // expects black background
                buffer[buffer_index] = (ray_color.x * opacity) as u8;
                buffer[buffer_index + 1] = (ray_color.y * opacity) as u8;
                buffer[buffer_index + 2] = (ray_color.z * opacity) as u8;

                buffer_index += 3;
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

            let (sample, grad) = self.volume.sample_at_gradient(pos);
            let grad = grad.normalize();

            let light_source = vector![1.0, 1.0, 0.0].normalize();

            let color_b = tf(sample);

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
