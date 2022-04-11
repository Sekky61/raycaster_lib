//! Volume rendering demo app
//!
//! Launch without arguments, for example:
//! `cargo run --release --bin vol_app`

use crate::state::State;
use crossbeam_channel::{select, Receiver, Sender};
use nalgebra::vector;
use native_dialog::FileDialog;
use raycaster_lib::render::RendererMessage;
use slint::{re_exports::EventResult, Image, Rgb8Pixel, SharedPixelBuffer};

// GUI bindings
slint::include_modules!();

mod state;

pub fn main() {
    // GUI App object and handles
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

    // Start thread listening for messages from renderer
    let (shutdown_send, shutdown_recv): (Sender<_>, Receiver<()>) = crossbeam_channel::bounded(2);
    let render_msg_recv_thread = {
        let state_mut = state.borrow_mut();
        let render_recv = state_mut.get_renderer_receiver();
        std::thread::spawn(move || loop {
            select! {
                recv(shutdown_recv) -> _ => return,
                recv(render_recv) -> msg => {
                    match msg {
                        Ok(()) => (),
                        Err(_) => return
                    }
                }
            }

            let a = app_poll.clone();
            slint::invoke_from_event_loop(move || a.unwrap().invoke_new_rendered_frame());
        })
    };

    //
    // Registering callbacks
    //

    // Callback
    // Invoked when new frame is rendered
    let state_clone = state.clone();
    app.on_new_rendered_frame(move || {
        let mut state = state_clone.borrow_mut();
        let app = state.get_app();

        let shared_buffer = state.get_buffer_handle();

        let pixel_buffer = {
            let resolution = state.get_resolution();
            let mut lock = shared_buffer.lock();
            let slice = lock.as_mut_slice();
            SharedPixelBuffer::<Rgb8Pixel>::clone_from_slice(
                slice,
                resolution.x as u32,
                resolution.y as u32,
            )
            // mutex drop
        };

        // Send image to GUI
        let image = Image::from_rgb8(pixel_buffer);
        app.set_render_target(image);

        println!("Frame displayed");

        state.handle_rendering_finished();
    });

    // Callback
    // React to mouse move in render area
    let state_clone = state.clone();
    app.on_render_area_move_event(move |mouse_pos| {
        state_clone
            .borrow_mut()
            .handle_mouse_pos(vector![mouse_pos.x, mouse_pos.y]);
    });

    // Callback
    // React to mouse event (click) in render area
    let state_clone = state.clone();
    app.on_render_area_pointer_event(move |pe| {
        state_clone.borrow_mut().handle_pointer_event(pe);
    });

    // Callback
    // React to keyboard event
    let state_clone = state.clone();
    app.on_key_pressed(move |ke| {
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

        state_clone.borrow_mut().set_file_picked(path);
    });

    // Open button callback
    let state_clone = state.clone();
    app.on_open_file(move || {
        let mut state = state_clone.borrow_mut();

        let app = state.get_app();
        let parser_gui_index = app.get_parser_picked_index();
        state.handle_open_vol(parser_gui_index);
    });

    // MT checkbox changed callback
    let state_clone = state.clone();
    app.on_mt_changed(move || {
        let mut state = state_clone.borrow_mut();
        let app = state.get_app();
        let checked = app.get_mt_checked();
        state.set_mt(checked);
    });

    // New TF selected callback
    let state_clone = state.clone();
    app.on_tf_selected(move |tf_name| {
        let mut state = state_clone.borrow_mut();
        state.handle_tf_changed(&tf_name);
    });

    // Callbacks for position sliders

    let state_clone = state.clone();
    app.on_x_slider_new_value(move |f| state_clone.borrow_mut().slider_event(0, f));

    let state_clone = state.clone();
    app.on_y_slider_new_value(move |f| state_clone.borrow_mut().slider_event(1, f));

    let state_clone = state.clone();
    app.on_z_slider_new_value(move |f| state_clone.borrow_mut().slider_event(2, f));

    // Run app
    app.show();
    slint::run_event_loop();
    app.hide();

    // Shutdown
    println!("App shutting down");
    state.borrow_mut().shutdown_renderer();

    shutdown_send.send(()).unwrap();
    render_msg_recv_thread.join().unwrap();
}
