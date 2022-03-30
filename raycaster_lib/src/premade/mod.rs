// Some prebuilt parsers and transfer functions
// for datasets used in development.
// With so many different data formats, a user should
// write their own to satisfy their needs.

pub mod parse;
pub mod transfer_functions;

pub fn render_frame(width: usize, height: usize) -> Vec<u8> {
    use crate::{
        premade::{
            parse::{from_file, skull_parser},
            transfer_functions::skull_tf,
        },
        render::{RenderOptions, Renderer},
        volumetric::LinearVolume,
        PerspectiveCamera,
    };
    use nalgebra::point;

    let position = point![300.0, 300.0, 300.0];
    let direction = position - point![34.0, 128.0, 128.0];
    let camera = PerspectiveCamera::new(position, direction);
    let volume = from_file("volumes/Skull.vol", skull_parser, skull_tf).unwrap();

    let ren_opts = RenderOptions {
        resolution: (width, height),
        ray_termination: true,
        empty_index: false,
    };

    let mut renderer = Renderer::<LinearVolume>::new(volume, ren_opts);

    let mut buffer = vec![0; 3 * width * height];

    renderer.render_to_buffer(&camera, buffer.as_mut_slice());

    buffer
}
