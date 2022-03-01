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

// pub struct State {
//     pub left_mouse_held: bool,
//     pub right_mouse_held: bool,
// }

// impl State {
//     fn get_user_input(&mut self, cam: &mut PerspectiveCamera, event: &sdl2::event::Event) {
//         match event {
//             Event::MouseMotion { xrel, yrel, .. } => {
//                 // When mouse button is down, drag camera around

//                 match (self.left_mouse_held, self.right_mouse_held) {
//                     (false, false) => (),
//                     (true, false) => {
//                         // move on the plane described by camera position and normal
//                         let drag_diff = (*xrel as f32, *yrel as f32);
//                         cam.change_pos_plane(-drag_diff.0 * 1.2, -drag_diff.1 * 1.2);
//                     }
//                     (false, true) => {
//                         // change camera direction
//                         let drag_diff = (*xrel as f32, *yrel as f32);
//                         cam.look_around(drag_diff.0 * -0.01, drag_diff.1 * -0.01);
//                     }
//                     (true, true) => {
//                         // rotate around origin
//                         let drag_diff = (*xrel as f32, *yrel as f32);
//                         let axisangle = Vector3::y() * (std::f32::consts::FRAC_PI_8 * drag_diff.0);
//                         let rot = nalgebra::Rotation3::new(axisangle);

//                         cam.change_pos_matrix(rot);
//                     }
//                 }
//             }
//             Event::MouseButtonDown { mouse_btn, .. } => match mouse_btn {
//                 sdl2::mouse::MouseButton::Left => self.left_mouse_held = true,
//                 sdl2::mouse::MouseButton::Right => self.right_mouse_held = true,
//                 _ => (),
//             },
//             Event::MouseButtonUp { mouse_btn, .. } => match mouse_btn {
//                 sdl2::mouse::MouseButton::Left => self.left_mouse_held = false,
//                 sdl2::mouse::MouseButton::Right => self.right_mouse_held = false,
//                 _ => (),
//             },
//             Event::MouseWheel { y, .. } => {
//                 // y        ... vertical scroll
//                 // +1 unit  ... 1 step of wheel down (negative -> scroll up)

//                 cam.change_pos_view_dir((*y as f32) * 5.0);
//             }
//             _ => {}
//         }
//     }
// }

fn start_render_thread(
    app_weak: slint::Weak<App>,
    shared_buffer: Arc<Mutex<Vec<u8>>>,
    rx: std::sync::mpsc::Receiver<bool>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
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
                .change_pos(vector![10.0, 10.0, 10.0]); // canary
            {
                let mut lock = shared_buffer.lock().unwrap();
                raycast_renderer.render_to_buffer(lock.as_mut_slice());
                // drop of mutex
            }

            let handle_copy = app_weak.clone();
            slint::invoke_from_event_loop(move || {
                handle_copy.unwrap().invoke_send_rendered_frame()
            });

            let res = rx.recv().unwrap();
            if res {
                continue;
            } else {
                break;
            }
        }
    })
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn main() {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(all(debug_assertions, target_arch = "wasm32"))]
    console_error_panic_hook::set_once();

    // Main App object and handles
    let app = App::new();
    let app_thread = app.as_weak();
    let app_render = app_thread.clone();
    let app_move = app_thread.clone();

    // IPC
    let (tx, rx) = std::sync::mpsc::channel();
    let shared_img = Arc::new(Mutex::new(vec![0u8; 3 * RENDER_WIDTH_U * RENDER_HEIGHT_U]));
    let shared_img_thread = shared_img.clone();

    // Rendering thread
    let render_thread_handle = start_render_thread(app_thread, shared_img_thread, rx);

    // Callback
    // Invoked when new frame is rendered
    app.on_send_rendered_frame(move || {
        let app = app_render.unwrap();

        let mut pixel_buffer = SharedPixelBuffer::<Rgb8Pixel>::new(RENDER_WIDTH, RENDER_HEIGHT);

        {
            let lock = shared_img.lock().unwrap();
            pixel_buffer
                .make_mut_bytes()
                .clone_from_slice(lock.as_slice());
            // mutex drop
        }
        tx.send(true).unwrap();
        let image = Image::from_rgb8(pixel_buffer);
        app.set_render_target(image);
    });

    app.on_render_area_pointer_event(move |pe| {
        println!("Pointer");
        println!("{pe:?}");
    });

    app.on_render_area_moved_event(move || {
        let app = app_move.unwrap();
        println!("Move");
        let x = app.get_render_area_mouse_x();
        println!("-- {:?}", x);
    });

    app.run();
    render_thread_handle.join().unwrap();
}
