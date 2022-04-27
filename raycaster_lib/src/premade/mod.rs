//! Some prebuilt parsers and transfer functions.
//!
//! Serves as an example to make your own, for example to add support
//! for some format

use nalgebra::Vector2;

pub mod parse;
pub mod transfer_functions;

/// Example of a usecase - single-threaded renderer
/// todo move to examples
pub fn render_frame(resolution: Vector2<u16>) -> Vec<u8> {
    use crate::{
        premade::{
            parse::{from_file, skull_parser},
            transfer_functions::skull_tf,
        },
        render::{RenderOptions, Renderer},
        volumetric::volumes::*,
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
    let render_options = RenderOptions::builder()
        .resolution(resolution)
        .early_ray_termination(true)
        .empty_space_skipping(false)
        .build_unchecked();

    // Instantiate a renderer, framebuffer
    let mut renderer = Renderer::<FloatVolume>::new(volume, render_options);
    let mut buffer = vec![0; 3 * (resolution.x as usize) * (resolution.y as usize)]; // 3 bytes per pixel

    // Run rendering (blocking)
    renderer.render(&camera, buffer.as_mut_slice());

    buffer
}
