pub struct RenderOptions {
    pub resolution: (usize, usize),
    pub ray_termination: bool,
    pub empty_index: bool,
}

impl RenderOptions {
    pub fn new(
        resolution: (usize, usize),
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
