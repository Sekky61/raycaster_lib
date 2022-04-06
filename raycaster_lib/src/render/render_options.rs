use nalgebra::Vector2;

pub struct RenderOptions {
    pub resolution: Vector2<u16>,
    pub ray_termination: bool,
    pub empty_index: bool,
}

impl RenderOptions {
    pub fn new(
        resolution: Vector2<u16>,
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
