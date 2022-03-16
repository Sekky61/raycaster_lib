use std::{
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use nalgebra::{point, vector};
use native_dialog::FileDialog;
use raycaster_lib::{
    camera::PerspectiveCamera,
    premade::{
        parse::{from_file, skull_parser},
        transfer_functions::skull_tf,
    },
    render::{ParalelRenderer, RenderOptions, RenderSingleThread, RendererMessage},
    volumetric::BlockVolume,
};
use slint::{re_exports::EventResult, Image, Rgb8Pixel, SharedPixelBuffer, Timer, TimerMode};
use state::{RENDER_HEIGHT, RENDER_HEIGHT_U, RENDER_WIDTH, RENDER_WIDTH_U};

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
        // Create renderer and tart render thread
        let mut state_mut = state.borrow_mut();
        let renderer = volume_setup_linear();
        state_mut.renderer_front.start_rendering(renderer);
        state_mut.render_thread_send_message(RendererMessage::StartRendering); // Initial command
    }

    let _timer = {
        let state_mut = state.borrow_mut();
        let app_poll = app_poll;
        let render_recv = state_mut.renderer_front.get_receiver();
        let timer = Timer::default();
        timer.start(TimerMode::Repeated, Duration::from_millis(1), move || {
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

        let mut pixel_buffer = SharedPixelBuffer::<Rgb8Pixel>::new(RENDER_WIDTH, RENDER_HEIGHT);

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

        // let mut item = slint::re_exports::StandardListViewItem::default();
        // item.text = SharedString::from("bar");
        // let list = slint::re_exports::ModelRc::new(item);

        // app_file.unwrap().set_parsers_name_list(list);

        state_clone.borrow_mut().handle_new_vol(path);
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

        state_clone.borrow_mut().handle_new_vol(path);
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

fn volume_setup_paralel() -> ParalelRenderer {
    let position = point![300.0, 300.0, 300.0];
    let direction = point![34.0, 128.0, 128.0] - position;
    let volume = from_file("volumes/Skull.vol", skull_parser, skull_tf).unwrap();

    let camera = PerspectiveCamera::new(position, direction);
    let camera = Arc::new(RwLock::new(camera));

    let render_options = RenderOptions::new((RENDER_WIDTH_U, RENDER_HEIGHT_U), true, true);

    ParalelRenderer::new(volume, camera, render_options)
}

fn volume_setup_linear() -> RenderSingleThread<BlockVolume> {
    let position = point![300.0, 300.0, 300.0];
    let direction = point![34.0, 128.0, 128.0] - position; // vector![-0.8053911, -0.357536, -0.47277182]
    let volume: BlockVolume = from_file("volumes/Skull.vol", skull_parser, skull_tf).unwrap();

    let camera = PerspectiveCamera::new(position, direction);
    let camera = Arc::new(RwLock::new(camera));

    let render_options = RenderOptions::new((RENDER_WIDTH_U, RENDER_HEIGHT_U), true, true);

    RenderSingleThread::new(volume, camera, render_options)
}
