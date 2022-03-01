use std::sync::{Arc, Mutex};

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

    let app = App::new();

    let (tx, rx) = std::sync::mpsc::channel();

    let app_weak = app.as_weak();
    let shared_img = Arc::new(Mutex::new(vec![0u8; 3 * RENDER_WIDTH_U * RENDER_HEIGHT_U]));
    let shared_img_thread = shared_img.clone();
    let thread = std::thread::spawn(move || {
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

        loop {
            raycast_renderer
                .camera
                .change_pos(vector![10.0, 10.0, 10.0]);
            {
                let mut lock = shared_img_thread.lock().unwrap();
                raycast_renderer.render_to_buffer(lock.as_mut_slice());
                // drop of mutex
                println!("thread dropped mutex");
            }

            let handle_copy = app_weak.clone();
            slint::invoke_from_event_loop(move || {
                handle_copy.unwrap().invoke_send_rendered_frame()
            });
            println!("thread sent");

            let res = rx.recv().unwrap();
            if res {
                continue;
            } else {
                break;
            }
        }
    });

    let app_weak = app.as_weak();
    app.on_send_rendered_frame(move || {
        println!("Running on r");
        let app = app_weak.unwrap();

        let mut pixel_buffer = SharedPixelBuffer::<Rgb8Pixel>::new(RENDER_WIDTH, RENDER_HEIGHT);

        {
            println!("main about to lock");
            let lock = shared_img.lock().unwrap();
            println!("main locked");
            pixel_buffer
                .make_mut_bytes()
                .clone_from_slice(lock.as_slice());
            // mutex drop
        }
        tx.send(true);
        let image = Image::from_rgb8(pixel_buffer);
        app.set_render_target(image);
        println!("Done");
    });

    app.run();
}
