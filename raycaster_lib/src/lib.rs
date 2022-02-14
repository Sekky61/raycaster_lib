pub mod camera;
mod ray;
pub mod render;
pub mod transfer_functions;
pub mod volumetric;

pub mod color {
    use nalgebra::{vector, Vector4};

    pub type RGBA = Vector4<f32>;

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> RGBA {
        vector![r, g, b, a]
    }

    pub fn zero() -> RGBA {
        vector![0.0, 0.0, 0.0, 0.0]
    }

    pub fn mono(v: f32, opacity: f32) -> RGBA {
        vector![v, v, v, opacity]
    }
}

pub fn render_frame(width: usize, height: usize) -> Vec<u8> {
    use crate::render::{RenderOptions, Renderer};
    use camera::TargetCamera;
    use volumetric::parse::skull_parser;
    use volumetric::LinearVolume;

    let camera = TargetCamera::new(width, height);
    let volume = volumetric::from_file(
        "volumes/Skull.vol",
        skull_parser,
        crate::transfer_functions::skull_tf,
    )
    .unwrap();

    let mut renderer = Renderer::<LinearVolume, TargetCamera>::new(volume, camera);
    renderer.set_render_options(RenderOptions {
        ray_termination: true,
        empty_index: false,
        multi_thread: false,
    });

    let mut buffer = vec![0; 3 * width * height];

    renderer.render_to_buffer(buffer.as_mut_slice());

    buffer
}
