use nalgebra::{vector, Vector3, Vector4};

use crate::{color::RGBA, common::Ray, volumetric::Volume, PerspectiveCamera};

use super::RenderOptions;

/// light direction (normalized)
const LIGHT_DIR: Vector3<f32> = vector![-0.74278, -0.55708, -0.37139];

/// Single threaded, synchronous renderer.
pub struct Renderer<V: Volume> {
    volume: V,
    render_options: RenderOptions,
}

impl<V> Renderer<V>
where
    V: Volume,
{
    /// Construct new renderer.
    ///
    /// # Params
    /// * `volume` - volume object implementing [`Volume`] trait.
    /// * `render_options` - Parameters for rendering.
    pub fn new(volume: V, render_options: RenderOptions) -> Renderer<V> {
        Renderer {
            volume,
            render_options,
        }
    }

    /// Public render function.
    ///
    /// # Params
    /// * `camera` - camera to cast rays from.
    /// * `buffer` - reference to target buffer.
    pub fn render(&mut self, camera: &PerspectiveCamera, buffer: &mut [u8]) {
        // Hide quality setting
        self.render_to_buffer(camera, buffer, true)
    }

    /// Public render function.
    ///
    /// # Params
    /// * `camera` - camera to cast rays from.
    /// * `buffer` - reference to target buffer.
    /// * `quality` - Use full (`true`) of fast (`false`) render quality. Specified in [`RenderOptions`].
    pub(crate) fn render_to_buffer(
        &mut self,
        camera: &PerspectiveCamera,
        buffer: &mut [u8],
        quality: bool,
    ) {
        // buffer y=0 is up
        // expects black background

        // Image resolution
        let img_w = self.render_options.resolution.x;
        let img_h = self.render_options.resolution.y;
        let (image_width, image_height) = (img_w as f32, img_h as f32);
        // Gap between pixels on canvas
        let step_x = 1.0 / image_width;
        let step_y = 1.0 / image_height;

        // Clear buffer
        for byte in buffer.iter_mut() {
            *byte = 0;
        }

        // Get rectangle in canvas
        let bbox = self.volume.get_bound_box();
        let rect = camera.project_box(bbox);
        let pixels = rect.get_pixel_range(self.render_options.resolution);

        // Stride
        let width_bytes_skip = (3 * (img_w - pixels.width())) as usize;
        let mut index =
            ((pixels.x.start as usize) + (img_w as usize) * (pixels.y.start as usize)) * 3;

        for y in pixels.y.clone() {
            let y_norm = y as f32 * step_y;
            for x in pixels.x.clone() {
                // Get ray
                let pixel_coord = (x as f32 * step_x, y_norm);
                let ray = camera.get_ray(pixel_coord);

                // Color pixel
                let ray_color = self.collect_light(&ray, camera, quality);

                // Draw boundbox
                // todo delete

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

                let color_bytes = ray_color * ray_color.w;
                buffer[index] = color_bytes.x as u8;
                buffer[index + 1] = color_bytes.y as u8;
                buffer[index + 2] = color_bytes.z as u8;
                index += 3;
            }
            index += width_bytes_skip;
        }
    }

    /// Accumulate color along one ray.
    /// Opacity is corrected for sample step
    fn collect_light(&self, ray: &Ray, camera: &PerspectiveCamera, quality: bool) -> RGBA {
        // Get intersection with volume
        let (obj_ray, t) = match self.volume.transform_ray(ray) {
            Some(e) => e,
            None => return vector![0.0, 0.0, 0.0, 0.0],
        };

        let mut rgb = vector![0.0, 0.0, 0.0];
        let mut opacity = 0.0;

        let view_dir_neg = -camera.get_dir();

        // Setup iterating along ray
        let begin = obj_ray.origin;
        let direction = ray.direction;

        let step_size = if quality {
            self.render_options.ray_step_quality
        } else {
            self.render_options.ray_step_fast
        };

        let step = direction * step_size; // normalized
        let mut pos = begin;

        let tf = self.volume.get_tf();

        // Source:
        // https://developer.nvidia.com/gpugems/gpugems/part-vi-beyond-triangles/chapter-39-volume-rendering-techniques
        // Equation 3
        //
        // reference_step_length / new_step_length
        let step_ratio = step_size;

        // Maximum number of step is known from intersection
        let max_n_of_steps = (t / step_size) as usize; // todo inverted into options
        for _ in 0..max_n_of_steps {
            // todo good ray iterator to save float to int casts

            // Empty space skipping
            if self.render_options.empty_space_skipping && self.volume.is_empty(pos) {
                pos += step;
                continue;
            }

            // Sample with gradient
            let (sample, grad_samples) = self.volume.sample_at_gradient(pos);

            pos += step;

            // Color sample
            let color_b = tf(sample);
            if color_b.w == 0.0 {
                continue;
            }
            let mut sample_rgb = color_b.xyz();

            // Inverted gradient, as low values indicate outside of an object
            let grad = vector![
                sample - grad_samples.x,
                sample - grad_samples.y,
                sample - grad_samples.z
            ];

            let grad_magnitude = grad.magnitude();
            const GRAD_MAG_THRESH: f32 = 10.0; // todo tweak

            // Apply shading to samples on the edge of the object only
            if grad_magnitude > GRAD_MAG_THRESH {
                // todo const albedo: f32 = 0.18 / PI;

                // Phong
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

            // Accumulate color
            rgb += (1.0 - opacity) * opacity_corrected * sample_rgb;
            opacity += (1.0 - opacity) * opacity_corrected;

            // ERT
            // relying on branch predictor to "eliminate" branch
            if self.render_options.early_ray_termination && opacity > 0.99 {
                break;
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
    fn understanding_phong() {
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
