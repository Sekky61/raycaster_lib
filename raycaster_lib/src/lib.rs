use crate::premade::{parse::skull_parser, transfer_functions::skull_tf};

pub mod camera;
pub mod premade;
mod ray;
pub mod render;
pub mod test_helpers;
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
    use camera::PerspectiveCamera;
    use nalgebra::point;
    use volumetric::LinearVolume;

    let position = point![300.0, 300.0, 300.0];
    let direction = position - point![34.0, 128.0, 128.0];
    let camera = PerspectiveCamera::new(position, direction);
    let volume = volumetric::from_file("volumes/Skull.vol", skull_parser, skull_tf).unwrap();

    let mut renderer = Renderer::<LinearVolume, PerspectiveCamera>::new(volume, camera);
    renderer.set_render_options(RenderOptions {
        resolution: (width, height),
        ray_termination: true,
        empty_index: false,
        multi_thread: false,
    });

    let mut buffer = vec![0; 3 * width * height];

    renderer.render_to_buffer(buffer.as_mut_slice());

    buffer
}
