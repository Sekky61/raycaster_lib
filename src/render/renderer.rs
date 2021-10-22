use crate::EmptyIndexes;

use super::*;

#[derive(PartialEq, Eq)]
pub enum BufferStatus {
    Ready,
    NotReady,
}

#[derive(Default)]
pub struct RendererOptions {
    pub ray_termination: bool,
    pub empty_index: bool,
    pub multi_thread: bool,
}

pub struct Renderer<V>
where
    V: Volume,
{
    pub(super) volume: V,
    pub(super) camera: Camera,
    pub(super) buffer: Vec<u8>,
    pub(super) buf_status: BufferStatus,
    pub(super) render: fn(&mut Self),
    pub(super) empty_index: EmptyIndexes,
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
            render: Self::default_render,
            empty_index,
        }
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
        (self.render)(self);
        self.buf_status = BufferStatus::Ready;
    }

    pub fn get_buffer(self) -> Vec<u8> {
        self.buffer
    }

    pub fn get_data(&self) -> &[u8] {
        self.buffer.as_slice()
    }

    fn default_render(&mut self) {
        panic!("Render not specified");
    }

    pub fn render(&mut self) {
        (self.render)(self);
        self.buf_status = BufferStatus::Ready;
    }

    pub fn render_settings(&mut self, options: RendererOptions) {
        self.render = match options {
            RendererOptions {
                ray_termination: true,
                empty_index: false,
                multi_thread: false,
            } => Self::render_rt_st,
            // RendererOptions {
            //     ray_termination: false,
            //     empty_index: false,
            //     multi_thread: false,
            // } => Self::render_st,
            // RendererOptions {
            //     ray_termination: true,
            //     empty_index: true,
            //     multi_thread: false,
            // } => Self::render_rt_ei,
            _ => panic!("Not implemented"),
        };
    }

    fn render_rt_st(&mut self) {
        //let buffer = self.buffer.as_mut_slice();
        println!("RENDER BUM st term");
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

                let ray_color = self.collect_light_term(&ray_world);

                let index = (y * self.camera.resolution.0 + x) * 3; // packed structs -/-

                self.buffer[index] = ray_color.0;
                self.buffer[index + 1] = ray_color.1;
                self.buffer[index + 2] = ray_color.2;
            }
        }
    }

    pub fn collect_light_term(&self, ray: &Ray) -> (u8, u8, u8) {
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

        //let mut steps_count = 0;

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

            // early ray termination
            if (color.3 - 0.99) > 0.0 {
                break;
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
