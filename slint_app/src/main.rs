use nalgebra::{point, vector};
use raycaster_lib::{
    camera::PerspectiveCamera,
    premade::{
        parse::{from_file, skull_parser},
        transfer_functions::skull_tf,
    },
    render::{RenderOptions, Renderer},
    volumetric::LinearVolume,
};
use slint::{Image, Rgb8Pixel, SharedPixelBuffer};

slint::include_modules!();

const RENDER_WIDTH_U: usize = 700;
const RENDER_HEIGHT_U: usize = 700;

const RENDER_WIDTH: u32 = RENDER_WIDTH_U as u32;
const RENDER_HEIGHT: u32 = RENDER_HEIGHT_U as u32;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn main() {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(all(debug_assertions, target_arch = "wasm32"))]
    console_error_panic_hook::set_once();

    let mut pixel_buffer = SharedPixelBuffer::<Rgb8Pixel>::new(RENDER_WIDTH, RENDER_HEIGHT);

    let volume: LinearVolume = from_file("volumes/Skull.vol", skull_parser, skull_tf).unwrap();

    let pos = point![300.0, 300.0, 300.0];
    let dir = vector![-1.0, -1.0, -1.0];
    let camera = PerspectiveCamera::new(pos, dir);

    let mut raycast_renderer = Renderer::<_, _>::new(volume, camera);

    raycast_renderer.set_render_options(RenderOptions {
        resolution: (RENDER_WIDTH_U, RENDER_HEIGHT_U),
        ray_termination: true,
        empty_index: true,
        multi_thread: false,
    });

    raycast_renderer.render_to_buffer(pixel_buffer.make_mut_bytes());

    let image = Image::from_rgb8(pixel_buffer);

    let app = App::new();

    app.set_render_target(image);

    app.run();
}
