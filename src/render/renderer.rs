use super::*;

pub const SINGLE_THREAD: bool = false;
pub const MULTI_THREAD: bool = true;

type Threading = bool;

pub struct Renderer<V, const TH: Threading>
where
    V: Volume,
{
    pub(super) volume: V,
    pub(super) camera: Camera,
}

impl<V> Renderer<V, SINGLE_THREAD>
where
    V: Volume,
{
    pub fn new(volume: V, camera: Camera) -> Renderer<V, SINGLE_THREAD> {
        Renderer { volume, camera }
    }

    pub fn set_camera_pos(&mut self, pos: Vector3<f32>) {
        self.camera.set_pos(pos);
    }

    pub fn change_camera_pos(&mut self, delta: Vector3<f32>) {
        self.camera.change_pos(delta);
    }
}

impl<V: Volume> Renderer<V, MULTI_THREAD> {
    pub fn new(volume: V, camera: Camera) -> Renderer<V, MULTI_THREAD> {
        Renderer { volume, camera }
    }

    pub fn set_camera_pos(&mut self, pos: Vector3<f32>) {
        self.camera.set_pos(pos);
    }

    pub fn change_camera_pos(&mut self, delta: Vector3<f32>) {
        self.camera.change_pos(delta);
    }
}
