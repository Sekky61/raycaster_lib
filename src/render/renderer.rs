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
}

impl<V> Renderer<V>
where
    V: Volume,
{
    pub fn new(volume: V, camera: Camera) -> Renderer<V> {
        let (w, h) = camera.get_resolution();
        Renderer {
            volume,
            camera,
            buffer: vec![0; w * h * 3],
            buf_status: BufferStatus::NotReady,
            render: Self::default_render,
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
}
