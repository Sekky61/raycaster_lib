use crate::{volumetric::BlockType, EmptyIndexes};

use super::*;

#[derive(PartialEq, Eq)]
pub enum BufferStatus {
    Ready,
    NotReady,
}

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

pub struct Renderer<V>
where
    V: Volume,
{
    pub(super) volume: V,
    pub(super) camera: Camera,
    pub(super) buffer: Vec<u8>,
    pub(super) buf_status: BufferStatus,
    pub(super) empty_index: EmptyIndexes,
    render_options: RenderOptions,
}

impl<V> Renderer<V>
where
    V: Volume,
{
    pub fn new(volume: V, camera: Camera) -> Renderer<V> {
        let (w, h) = camera.get_resolution();
        let empty_index = EmptyIndexes::from_volume(&volume);
        Renderer {
            volume,
            camera,
            buffer: vec![0; w * h * 3],
            buf_status: BufferStatus::NotReady,
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

    pub fn set_camera_pos(&mut self, pos: Vector3<f32>) {
        self.camera.set_pos(pos);
    }

    pub fn change_camera_pos(&mut self, delta: Vector3<f32>) {
        self.camera.change_pos(delta);
    }

    pub fn try_get_frame(&mut self) -> Option<&[u8]> {
        if self.buf_status == BufferStatus::NotReady {
            return None;
        }

        self.buf_status = BufferStatus::NotReady;
        Some(self.buffer.as_slice())
    }

    pub fn render_to_buffer(&mut self) {
        self.render();
        self.buf_status = BufferStatus::Ready;
    }

    pub fn get_buffer(self) -> Vec<u8> {
        self.buffer
    }

    pub fn get_data(&self) -> &[u8] {
        self.buffer.as_slice()
    }

    fn render(&mut self) {
        println!("THE RENDER");
        let (image_width, image_height) = (
            self.camera.resolution.0 as f32,
            self.camera.resolution.1 as f32,
        );

        let origin_4 = Vector4::new(
            self.camera.position.x,
            self.camera.position.y,
            self.camera.position.z,
            1.0,
        );

        let aspect_ratio = image_width / image_height;

        // cam to world
        let lookat_matrix = self.camera.get_look_at_matrix();

        for y in 0..self.camera.resolution.1 {
            for x in 0..self.camera.resolution.0 {
                let pixel_ndc_x = (x as f32 + 0.5) / image_width;
                let pixel_ndc_y = (y as f32 + 0.5) / image_height;

                let pixel_screen_x = (pixel_ndc_x * 2.0 - 1.0) * aspect_ratio;
                let pixel_screen_y = 1.0 - pixel_ndc_y * 2.0; // v NDC Y roste dolu, obratime

                //todo FOV

                let pix_cam_space = vector![pixel_screen_x, pixel_screen_y, -1.0, 1.0];

                let dir_world = (lookat_matrix * pix_cam_space) - origin_4;
                let dir_world_3 = dir_world.xyz().normalize();

                //println!("{}", dir_world_3);

                let ray_world = Ray::from_3(self.camera.position, dir_world_3);

                let ray_color = self.collect_light_index(&ray_world);

                let index = (y * self.camera.resolution.0 + x) * 3; // packed structs -/-

                self.buffer[index] = ray_color.0;
                self.buffer[index + 1] = ray_color.1;
                self.buffer[index + 2] = ray_color.2;
            }
        }
    }

    pub fn collect_light(&self, ray: &Ray) -> (u8, u8, u8) {
        let mut accum = (0.0, 0.0, 0.0, 0.0);

        let (t1, t2) = match self.volume.intersect(ray) {
            Some(tup) => tup,
            None => return (0, 0, 0),
        };

        let begin = ray.point_from_t(t1);
        let direction = ray.get_direction();

        let step_size = 1.0;

        let step = direction * step_size; // normalized

        let mut pos = begin;

        loop {
            let sample = self.volume.sample_at(pos);

            let color = transfer_function(sample);

            pos += step;

            if color.3 == 0.0 {
                if !self.volume.is_in(pos) {
                    break;
                }
                continue;
            }

            // pseudocode from https://scholarworks.rit.edu/cgi/viewcontent.cgi?article=6466&context=theses page 55, figure 5.6
            //sum = (1 - sum.alpha) * volume.density * color + sum;

            accum.0 += (1.0 - accum.3) * color.0;
            accum.1 += (1.0 - accum.3) * color.1;
            accum.2 += (1.0 - accum.3) * color.2;
            accum.3 += (1.0 - accum.3) * color.3;

            // relying on branch predictor to "eliminate" branch
            if self.render_options.ray_termination {
                // early ray termination
                if (color.3 - 0.99) > 0.0 {
                    break;
                }
            }

            if !self.volume.is_in(pos) {
                break;
            }
        }

        let accum_i_x = accum.0.min(255.0) as u8;
        let accum_i_y = accum.1.min(255.0) as u8;
        let accum_i_z = accum.2.min(255.0) as u8;

        (accum_i_x, accum_i_y, accum_i_z)
    }

    pub fn collect_light_index(&self, ray: &Ray) -> (u8, u8, u8) {
        let mut accum = (0.0, 0.0, 0.0, 0.0);

        let (t1, t2) = match self.volume.intersect(ray) {
            Some(tup) => tup,
            None => return (0, 0, 0),
        };

        let begin = ray.point_from_t(t1);
        let direction = ray.get_direction();

        let step_size = 1.0;

        let step = direction * step_size; // normalized

        let mut pos = begin;

        let m_max = self.empty_index.len() - 1;
        let mut m = m_max;

        loop {
            let index = self.empty_index.get_index_at(m, pos);

            if index == BlockType::NonEmpty {
                if m > 0 {
                    // go down a level
                    m -= 1;
                    continue;
                } else {
                    // m == 0
                    // sample
                    let sample = self.volume.sample_at(pos);

                    let color = transfer_function(sample);

                    accum.0 += (1.0 - accum.3) * color.0;
                    accum.1 += (1.0 - accum.3) * color.1;
                    accum.2 += (1.0 - accum.3) * color.2;
                    accum.3 += (1.0 - accum.3) * color.3;

                    if self.render_options.ray_termination {
                        // early ray termination
                        if (color.3 - 0.99) > 0.0 {
                            break;
                        }
                    }

                    continue;
                }
            }

            // empty

            // step to next on same level

            let ray_dirs = step.map(|v| if v > 0.0 { 1.0f32 } else { 0.0 });
            let index_edge = EmptyIndexes::get_index_size(m);
            let index_3d_offset = EmptyIndexes::get_block_coords(m, pos);
            let index_low_coords = index_3d_offset * index_edge;
            let index_low_coords = index_low_coords.map(|v| v as f32);

            // remember parent
            let parent_offset = EmptyIndexes::get_block_coords(m + 1, pos);

            let delta_i =
                (ray_dirs * (index_edge as f32) + index_low_coords - pos).component_div(&step);

            let delta_i = delta_i.map(|f| f.ceil() as usize);

            let n_of_steps = delta_i.min().max(1);

            pos += step * (n_of_steps as f32);

            let new_parent_offset = EmptyIndexes::get_block_coords(m + 1, pos);

            while parent_offset != new_parent_offset && m < m_max - 1 {
                // parents changed
                m += 1;
            }

            if !self.volume.is_in(pos) {
                break;
            }
        }

        let accum_i_x = accum.0.min(255.0) as u8;
        let accum_i_y = accum.1.min(255.0) as u8;
        let accum_i_z = accum.2.min(255.0) as u8;

        (accum_i_x, accum_i_y, accum_i_z)
    }
}

// R G B A -- A <0;1>
pub fn transfer_function(sample: f32) -> (f32, f32, f32, f32) {
    if sample > 180.0 {
        (60.0, 230.0, 40.0, 0.3)
    } else if sample > 70.0 {
        (230.0, 10.0, 10.0, 0.3)
    } else if sample > 50.0 {
        (10.0, 20.0, 100.0, 0.1)
    } else if sample > 5.0 {
        (10.0, 10.0, 40.0, 0.05)
    } else {
        (0.0, 0.0, 0.0, 0.0)
    }
}
