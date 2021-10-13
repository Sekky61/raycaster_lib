use crate::camera::Camera;
use crate::volume::Volume;

pub struct Renderer<V>
where
    V: Volume,
{
    volume: V,
    camera: Camera,
}
