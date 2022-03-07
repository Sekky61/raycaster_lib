use std::sync::{Arc, Mutex};

use native_dialog::FileDialog;
use render_thread::{
    RenderThread, RenderThreadMessage, RENDER_HEIGHT, RENDER_HEIGHT_U, RENDER_WIDTH, RENDER_WIDTH_U,
};
use slint::{Image, Rgb8Pixel, SharedPixelBuffer};

slint::include_modules!();

mod render_thread;

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

    // Callback
    // Invoked when new frame is rendered
    let sender = renderer_sender.clone();
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
        sender.send_message(RenderThreadMessage::StartRendering);
        let image = Image::from_rgb8(pixel_buffer);
        app.set_render_target(image);
    });

    // React to mouse move in render area
    let sender = renderer_sender.clone();
    app.on_render_area_move_event(move |mouse_pos| {
        sender.send_message(RenderThreadMessage::MousePos(mouse_pos));
    });

    // React to mouse event (click) in render area
    let sender = renderer_sender.clone();
    app.on_render_area_pointer_event(move |pe| {
        sender.send_message(RenderThreadMessage::MouseClick(pe));
    });

    let sender = renderer_sender.clone();
    app.on_load_file(move || {
        let path = FileDialog::new()
            .set_location(".")
            .show_open_single_file()
            .unwrap();

        let path = match path {
            Some(path) => path,
            None => return,
        };

        sender.send_message(RenderThreadMessage::NewVolume(path));
    });

    let sender = renderer_sender.clone();
    app.on_load_folder(move || {
        let path = FileDialog::new()
            .set_location(".")
            .show_open_single_dir()
            .unwrap();

        let path = match path {
            Some(path) => path,
            None => return,
        };

        sender.send_message(RenderThreadMessage::NewVolume(path));
    });

    app.show();
    slint::run_event_loop();
    app.hide();

    println!("App shutting down");
    renderer_sender.send_message(RenderThreadMessage::ShutDown);

    let join_result = render_thread_handle.join();
    match join_result {
        Ok(_) => (),
        Err(_) => eprintln!("Render thread fialed"),
    }
}
