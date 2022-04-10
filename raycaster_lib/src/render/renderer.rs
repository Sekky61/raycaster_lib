use nalgebra::{vector, Vector2, Vector4};

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
    pub empty_index: EmptyIndex<4>,
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

    pub fn set_render_resolution(&mut self, res: Vector2<u16>) {
        self.render_options.resolution = res;
    }

    // buffer y=0 is up
    pub fn render_to_buffer(
        &mut self,
        camera: &PerspectiveCamera,
        buffer: &mut [u8],
        quality: bool,
    ) {
        let img_w = self.render_options.resolution.x;
        let img_h = self.render_options.resolution.y;

        let (image_width, image_height) = (img_w as f32, img_h as f32);

        // Clear
        for byte in buffer.iter_mut() {
            *byte = 0;
        }

        let step_x = 1.0 / image_width;
        let step_y = 1.0 / image_height;

        let bbox = self.volume.get_bound_box();
        let tile = camera.project_box(bbox);

        let pixels = tile.get_pixel_range(self.render_options.resolution);
        let pix_width = pixels.width();

        let width_bytes_skip = (3 * (img_w - pix_width)) as usize;
        let mut index =
            ((pixels.x.start as usize) + (img_w as usize) * (pixels.y.start as usize)) * 3;

        for y in pixels.y.clone() {
            let y_norm = y as f32 * step_y;
            for x in pixels.x.clone() {
                let pixel_coord = (x as f32 * step_x, y_norm);
                let ray = camera.get_ray(pixel_coord);

                let ray_color = self.collect_light(&ray, camera, quality);

                let opacity = ray_color.w;

                // Draw boundbox

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

    pub fn collect_light(
        &self,
        ray: &Ray,
        camera: &PerspectiveCamera,
        quality: bool,
    ) -> Vector4<f32> {
        let mut rgb = vector![0.0, 0.0, 0.0];
        let mut opacity = 0.0;

        let (obj_ray, t) = match self.volume.intersect_transform(ray) {
            Some(e) => e,
            None => return vector![0.0, 0.0, 0.0, 0.0],
        };

        let view_dir_neg = -camera.get_dir();

        let begin = obj_ray.origin;
        let direction = ray.get_direction();

        let step_size = if quality {
            self.render_options.ray_step_quality
        } else {
            self.render_options.ray_step_fast
        };

        let max_n_of_steps = (t / step_size) as usize;

        let step = direction * step_size; // normalized

        let mut pos = begin;

        let tf = self.volume.get_tf();
        let light_dir = vector![-1.0, -1.0, -1.0].normalize(); // light direction

        for _ in 0..max_n_of_steps {
            //let sample = self.volume.sample_at(pos);

            // todo try sampling on integer coords
            if self.render_options.empty_space_skipping
                && self.empty_index.sample(pos) == BlockType::Empty
            {
                pos += step;
                continue;
            }

            let (sample, grad_samples) = self.volume.sample_at_gradient(pos);

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
                // todo const albedo: f32 = 0.18 / PI;
                let grad_norm = grad / grad_magnitude;
                let diffuse = f32::max(grad_norm.dot(&-light_dir), 0.00); // ambient light 0.09

                let reflect = light_dir - 2.0 * (grad_norm.dot(&light_dir)) * grad_norm;
                let r_dot_view = reflect.dot(&view_dir_neg);
                let light_intensity = 200.0;
                let specular = f32::max(0.0, r_dot_view).powf(128.0) * light_intensity;

                sample_rgb = sample_rgb * (diffuse + 0.09) + vector![specular, specular, specular];
            }

            // pseudocode from https://scholarworks.rit.edu/cgi/viewcontent.cgi?article=6466&context=theses page 55, figure 5.6
            //sum = (1 - sum.alpha) * volume.density * color + sum;

            rgb += (1.0 - opacity) * color_b.w * sample_rgb;
            opacity += (1.0 - opacity) * color_b.w;

            // relying on branch predictor to "eliminate" branch
            if self.render_options.early_ray_termination {
                // early ray termination
                if opacity > 0.99 {
                    break;
                }
            }
        }
        vector![rgb.x, rgb.y, rgb.z, opacity]
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use nalgebra::Vector3;

    #[test]
    fn phong() {
        // view    light
        //    \ | /
        //     \|/
        // ------------ 0
        //
        // x is right, y is up, z=0

        let mut sample_rgb = vector![255.0, 0.0, 0.0];

        let view_dir = vector![0.4, -1.0, 0.0];
        let view_dir_neg = -view_dir;

        let light_dir = vector![-1.0, -1.0, 0.0].normalize();

        let grad: Vector3<f32> = vector![0.0, 1.0, 0.0];
        let grad_magnitude = grad.magnitude();

        let grad_norm = grad / grad_magnitude;
        let diffuse = f32::max(grad_norm.dot(&-light_dir), 0.00); // ambient light 0.09

        let reflect = light_dir - 2.0 * (grad_norm.dot(&light_dir)) * grad_norm;
        let r_dot_view = reflect.dot(&view_dir_neg);
        let light_intensity = 15.0;
        let specular = f32::max(0.0, r_dot_view).powf(128.0) * light_intensity;

        sample_rgb = sample_rgb * (diffuse + 0.09) + vector![specular, specular, specular];

        println!("rgb: {sample_rgb:?}");
    }
}
