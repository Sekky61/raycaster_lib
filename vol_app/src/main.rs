use std::time::Duration;

use nalgebra::vector;
use native_dialog::FileDialog;
use raycaster_lib::render::RendererMessage;
use slint::{re_exports::EventResult, Image, Rgb8Pixel, SharedPixelBuffer, Timer, TimerMode};

use crate::state::State;

slint::include_modules!();

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
    let app_weak = app.as_weak();
    let app_poll = app_weak.clone();

    // State
    // Wrapped for access in closures
    let state = State::new_shared(app_weak);
    {
        // Start render thread
        let mut state_mut = state.borrow_mut();
        state_mut.initial_render_call();
    }

    let _timer = {
        let state_mut = state.borrow_mut();
        let app_poll = app_poll;
        let render_recv = state_mut.renderer_front.get_receiver();
        let timer = Timer::default();
        timer.start(TimerMode::Repeated, Duration::from_millis(1), move || {
            // todo recv instead of try_recv, passive wait
            if render_recv.try_recv().is_ok() {
                // New Frame
                let a = app_poll.clone();
                slint::invoke_from_event_loop(move || a.unwrap().invoke_new_rendered_frame());
            }
        });
        timer
    };

    // Callback
    // Invoked when new frame is rendered
    let state_clone = state.clone();
    app.on_new_rendered_frame(move || {
        let mut state_ref = state_clone.borrow_mut();
        let app = state_ref.app.unwrap();

        let mut pixel_buffer = SharedPixelBuffer::<Rgb8Pixel>::new(
            state_ref.render_resolution.x as u32,
            state_ref.render_resolution.y as u32,
        );

        let shared_buffer = state_ref.renderer_front.get_buffer_handle().unwrap();

        {
            let mut lock = shared_buffer.lock().unwrap();
            let slice = lock.as_mut_slice();
            pixel_buffer.make_mut_bytes().clone_from_slice(slice);
            // mutex drop
        }
        state_ref.is_rendering = false;
        let image = Image::from_rgb8(pixel_buffer);
        app.set_render_target(image);

        println!("Frame displayed");

        // Frame time
        let elapsed = state_ref.timer.elapsed();
        app.set_frame_time(elapsed.as_millis().try_into().unwrap());
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

    // Pick file button callback
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

        state_clone.borrow_mut().file_picked = Some(path);
    });

    // Open button callback
    let state_clone = state.clone();
    app.on_open_file(move || {
        let mut state = state_clone.borrow_mut();

        let app = state.app.unwrap();
        let parser_gui_index = app.get_parser_picked_index();
        state.handle_open_vol(parser_gui_index);
    });

    // MT checkbox changed callback
    let state_clone = state.clone();
    app.on_mt_changed(move || {
        let mut state = state_clone.borrow_mut();
        let app = state.app.unwrap();
        let checked = app.get_mt_checked();
        state.multi_thread = checked;
    });

    // New TF selected callback
    let state_clone = state.clone();
    app.on_tf_selected(move |tf_name| {
        let mut state = state_clone.borrow_mut();
        state.handle_tf_changed(&tf_name);
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
        .renderer_front
        .send_message(RendererMessage::ShutDown);

    state.borrow_mut().renderer_front.finish();
}
