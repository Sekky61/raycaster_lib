use super::*;

pub const SINGLE_THREAD: bool = false;
pub const MULTI_THREAD: bool = true;

type Threading = bool;

#[derive(PartialEq, Eq)]
pub enum BufferStatus {
    Ready,
    NotReady,
}

pub trait Render {
    fn render(&mut self);
}

pub struct Renderer<V, const TH: Threading>
where
    V: Volume,
{
    pub(super) volume: V,
    pub(super) camera: Camera,
    buffer: Vec<u8>,
    buf_status: BufferStatus,
}

impl<V> Renderer<V, SINGLE_THREAD>
where
    V: Volume,
{
    pub fn new(volume: V, camera: Camera) -> Renderer<V, SINGLE_THREAD> {
        let (w, h) = camera.get_resolution();
        Renderer {
            volume,
            camera,
            buffer: vec![0; w * h * 3],
            buf_status: BufferStatus::NotReady,
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

    pub fn render_to_buffer(&mut self)
    where
        Self: Render,
    {
        self.render();
        self.buf_status = BufferStatus::Ready;
    }

    pub fn get_buffer(self) -> Vec<u8> {
        self.buffer
    }

    pub fn get_data(&self) -> &[u8] {
        self.buffer.as_slice()
    }
}

impl<V: Volume> Renderer<V, MULTI_THREAD> {
    pub fn new(volume: V, camera: Camera) -> Renderer<V, MULTI_THREAD> {
        let (w, h) = camera.get_resolution();
        Renderer {
            volume,
            camera,
            buffer: vec![0; w * h * 3],
            buf_status: BufferStatus::NotReady,
        }
    }

    pub fn set_camera_pos(&mut self, pos: Vector3<f32>) {
        self.camera.set_pos(pos);
    }

    pub fn change_camera_pos(&mut self, delta: Vector3<f32>) {
        self.camera.change_pos(delta);
    }
}

impl Render for Renderer<LinearVolume, SINGLE_THREAD> {
    fn render(&mut self) {
        let buffer = self.buffer.as_mut_slice();
        println!("RENDER BUM");
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
                let mut dir_world_3 = dir_world.xyz();
                dir_world_3.normalize_mut();

                //println!("{}", dir_world_3);

                let ray_world = Ray::from_3(self.camera.position, dir_world_3);

                let ray_color = self.volume.collect_light(&ray_world);

                let index = (y * self.camera.resolution.0 + x) * 3; // packed structs -/-

                buffer[index] = ray_color.0;
                buffer[index + 1] = ray_color.1;
                buffer[index + 2] = ray_color.2;
            }
        }
    }
}

impl Renderer<LinearVolume, MULTI_THREAD> {
    pub fn render(&mut self) {
        let mut buffer = self.buffer.as_mut_slice();
        println!("OMG THE 1 RENDERER");
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
                let mut dir_world_3 = dir_world.xyz();
                dir_world_3.normalize_mut();

                //println!("{}", dir_world_3);

                let ray_world = Ray::from_3(self.camera.position, dir_world_3);

                let ray_color = self.volume.collect_light(&ray_world);

                let index = (y * self.camera.resolution.0 + x) * 3; // packed structs -/-

                buffer[index] = ray_color.0;
                buffer[index + 1] = ray_color.1;
                buffer[index + 2] = ray_color.2;
            }
        }
    }
}
