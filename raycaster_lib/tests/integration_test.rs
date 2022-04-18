use nalgebra::{point, vector, Point3, Vector2, Vector3};
use raycaster_lib::{
    render::RenderOptions, test_helpers, volumetric::volumes::LinearVolume, PerspectiveCamera,
};

pub const WIDTH: u16 = 700;
pub const HEIGHT: u16 = 700;
pub const RESOLUTION: Vector2<u16> = vector![WIDTH, HEIGHT];

pub const POSITION: Point3<f32> = point![300.0, 300.0, 300.0];
pub const DIRECTION: Vector3<f32> = vector![-1.0, -1.0, -1.0];

#[test]
fn single_thread_api() {
    let volume: LinearVolume = test_helpers::skull_volume(None);
    let camera = PerspectiveCamera::new(POSITION, DIRECTION);

    let render_options = RenderOptions::builder()
        .resolution(RESOLUTION)
        .early_ray_termination(true)
        .empty_space_skipping(true)
        .build_unchecked();

    let mut renderer = raycaster_lib::render::Renderer::new(volume, render_options);

    let mut buffer = vec![0; 3 * (WIDTH as usize) * (HEIGHT as usize)];

    renderer.render(&camera, &mut buffer);

    assert_eq!(4, 2 + 2);
}
