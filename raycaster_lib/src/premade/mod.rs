//! Some prebuilt parsers and transfer functions.
//!
//! Serves as an example to make your own, for example to add support
//! for some format

pub mod parse;
pub mod transfer_functions;

/// Example of a usecase - single-threaded renderer
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

    // Camera setup
    let position = point![300.0, 300.0, 300.0];
    let direction = position - point![34.0, 128.0, 128.0];
    let camera = PerspectiveCamera::new(position, direction);

    // Load volume
    //
    // Choose file, parser and transfer function
    // note that the type of volume is inferred
    let volume = from_file("volumes/Skull.vol", skull_parser, skull_tf).unwrap();

    // Render options - set resolution and optimisations
    let ren_opts = RenderOptions {
        resolution: (width, height),
        ray_termination: true,
        empty_index: false,
    };

    // Instantiate a renderer, framebuffer
    let mut renderer = Renderer::<LinearVolume>::new(volume, ren_opts);
    let mut buffer = vec![0; 3 * width * height]; // 3 bytes per pixel

    // Run rendering (blocking)
    renderer.render_to_buffer(&camera, buffer.as_mut_slice());

    buffer
}
