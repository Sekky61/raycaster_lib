use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use nalgebra::vector;
use native_dialog::FileDialog;
use render_thread::{
    RenderThread, RenderThreadMessage, RENDER_HEIGHT, RENDER_HEIGHT_U, RENDER_WIDTH, RENDER_WIDTH_U,
};
use slint::{re_exports::EventResult, Image, Rgb8Pixel, SharedPixelBuffer};

use crate::state::State;

slint::include_modules!();

mod render_thread;
mod state;

/* chybí mousewheel
// y        ... vertical scroll
                // +1 unit  ... 1 step of wheel down (negative -> scroll up)

                cam.change_pos_view_dir((*y as f32) * 5.0);
*/

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

    // IPC
    let shared_img = Arc::new(Mutex::new(vec![0u8; 3 * RENDER_WIDTH_U * RENDER_HEIGHT_U]));
    let shared_img_thread = shared_img.clone();

    // Rendering thread
    let render_thread = RenderThread::new(app_thread, shared_img_thread);
    let renderer_sender = render_thread.get_sender();
    let render_thread_handle = render_thread.start();

    // State
    // Wrapped for access in closures
    let state = State::new_shared(renderer_sender);

    // Callback
    // Invoked when new frame is rendered
    let state_clone = state.clone();
    app.on_send_rendered_frame(move || {
        let app = app_render.unwrap();

        let mut state_ref = state_clone.borrow_mut();

        let mut pixel_buffer = SharedPixelBuffer::<Rgb8Pixel>::new(RENDER_WIDTH, RENDER_HEIGHT);

        {
            let mut lock = shared_img.lock().unwrap();
            let slice = lock.as_mut_slice();
            pixel_buffer.make_mut_bytes().clone_from_slice(slice);
            // TODO measure performance
            for v in slice {
                *v = 0;
            }
            // mutex drop
        }
        state_ref.render_thread_send_message(RenderThreadMessage::StartRendering);
        let image = Image::from_rgb8(pixel_buffer);
        app.set_render_target(image);

        // Frame time
        let elapsed = state_ref.timer.elapsed();
        app.set_frame_time(elapsed.as_millis().try_into().unwrap());
        state_ref.timer = Instant::now();
    });

    // React to mouse move in render area
    let state_clone = state.clone();
    app.on_render_area_move_event(move |mouse_pos| {
        state_clone
            .borrow_mut()
            .handle_mouse_pos(vector![mouse_pos.x, mouse_pos.y]);
    });

    // React to mouse event (click) in render area
    let state_clone = state.clone();
    app.on_render_area_pointer_event(move |pe| {
        state_clone.borrow_mut().handle_pointer_event(pe);
    });

    let state_clone = state.clone();
    app.on_render_key_pressed(move |ke| {
        let ch = match ke.text.as_str().chars().next() {
            Some(ch) => ch,
            None => return EventResult::accept,
        };
        state_clone.borrow_mut().handle_key_press(ch);
        EventResult::accept
    });

    let state_clone = state.clone();
    app.on_load_file(move || {
        let path = FileDialog::new()
            .set_location(".")
            .show_open_single_file()
            .unwrap();

        let path = match path {
            Some(path) => path,
            None => return,
        };

        state_clone
            .borrow_mut()
            .render_thread_send_message(RenderThreadMessage::NewVolume(path));
    });

    let state_clone = state.clone();
    app.on_load_folder(move || {
        let path = FileDialog::new()
            .set_location(".")
            .show_open_single_dir()
            .unwrap();

        let path = match path {
            Some(path) => path,
            None => return,
        };

        state_clone
            .borrow_mut()
            .render_thread_send_message(RenderThreadMessage::NewVolume(path));
    });

    let state_clone = state.clone();
    app.on_x_slider_new_value(move |f| state_clone.borrow_mut().slider_event(0, f));

    let state_clone = state.clone();
    app.on_y_slider_new_value(move |f| state_clone.borrow_mut().slider_event(1, f));

    let state_clone = state.clone();
    app.on_z_slider_new_value(move |f| state_clone.borrow_mut().slider_event(2, f));

    app.show();
    slint::run_event_loop();
    app.hide();

    println!("App shutting down");
    state
        .borrow_mut()
        .render_thread_send_message(RenderThreadMessage::ShutDown);

    let join_result = render_thread_handle.join();
    match join_result {
        Ok(_) => (),
        Err(_) => eprintln!("Render thread fialed"),
    }
}
