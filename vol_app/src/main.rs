use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
    time::Instant,
};

use nalgebra::{vector, Vector2};
use native_dialog::FileDialog;
use render_thread::{
    RenderThread, RenderThreadMessage, RenderThreadMessageSender, RENDER_HEIGHT, RENDER_HEIGHT_U,
    RENDER_WIDTH, RENDER_WIDTH_U,
};
use slint::{
    re_exports::{PointerEvent, PointerEventButton, PointerEventKind},
    Image, Rgb8Pixel, SharedPixelBuffer,
};

slint::include_modules!();

mod render_thread;

/* chybí mousewheel
// y        ... vertical scroll
                // +1 unit  ... 1 step of wheel down (negative -> scroll up)

                cam.change_pos_view_dir((*y as f32) * 5.0);
*/

pub struct State {
    pub sender: RenderThreadMessageSender,
    pub timer: Instant,
    pub left_mouse_held: bool,
    pub right_mouse_held: bool,
    pub mouse: Option<Vector2<f32>>,
}

impl State {
    fn new(sender: RenderThreadMessageSender) -> State {
        State {
            sender,
            left_mouse_held: false,
            right_mouse_held: false,
            mouse: None,
            timer: Instant::now(),
        }
    }

    fn render_thread_send_message(&self, message: RenderThreadMessage) {
        self.sender.send_message(message);
    }

    fn handle_mouse_pos(&mut self, action: MousePos) {
        // rust-analyzer struggles here because m is of generated type
        // The type is (f32, f32)

        let drag_diff = if let Some(base) = self.mouse {
            (action.x - base.x, action.y - base.y)
        } else {
            self.mouse = Some(vector![action.x, action.y]);
            return;
        };

        self.mouse = Some(vector![action.x, action.y]);

        match (self.left_mouse_held, self.right_mouse_held) {
            (false, false) => (),
            (true, false) => {
                // move on the plane described by camera position and normal
                let delta = vector![drag_diff.0 * 0.2, drag_diff.1 * 0.2];
                self.sender
                    .send_message(RenderThreadMessage::CameraChangePositionPlane(delta));
            }
            (false, true) => {
                // change camera direction
                let delta = vector![drag_diff.0 * -0.001, drag_diff.1 * -0.001];
                self.sender
                    .send_message(RenderThreadMessage::CameraChangeDirection(delta));
            }
            (true, true) => {
                // rotate around origin
                // TODO
                // let axisangle = Vector3::y() * (std::f32::consts::FRAC_PI_8 * drag_diff.0);
                // let rot = nalgebra::Rotation3::new(axisangle);

                // cam.change_pos_matrix(rot);
            }
        }
    }

    fn handle_pointer_event(&mut self, pe: PointerEvent) {
        self.mouse = None;
        match pe {
            PointerEvent {
                button: PointerEventButton::left,
                kind: PointerEventKind::up,
            } => self.left_mouse_held = false,
            PointerEvent {
                button: PointerEventButton::left,
                kind: PointerEventKind::down,
            } => self.left_mouse_held = true,
            PointerEvent {
                button: PointerEventButton::right,
                kind: PointerEventKind::up,
            } => self.right_mouse_held = false,
            PointerEvent {
                button: PointerEventButton::right,
                kind: PointerEventKind::down,
            } => self.right_mouse_held = true,
            _ => (),
        }
    }
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

    // IPC
    let shared_img = Arc::new(Mutex::new(vec![0u8; 3 * RENDER_WIDTH_U * RENDER_HEIGHT_U]));
    let shared_img_thread = shared_img.clone();

    // Rendering thread
    let render_thread = RenderThread::new(app_thread, shared_img_thread);
    let renderer_sender = render_thread.get_sender();
    let render_thread_handle = render_thread.start();

    // State
    // Wrapped for access in closures
    let state = State::new(renderer_sender);
    let state = Rc::new(RefCell::new(state));

    // Callback
    // Invoked when new frame is rendered
    let state_clone = state.clone();
    app.on_send_rendered_frame(move || {
        let app = app_render.unwrap();

        let mut state_ref = state_clone.borrow_mut();

        let mut pixel_buffer = SharedPixelBuffer::<Rgb8Pixel>::new(RENDER_WIDTH, RENDER_HEIGHT);

        {
            let lock = shared_img.lock().unwrap();
            pixel_buffer
                .make_mut_bytes()
                .clone_from_slice(lock.as_slice());
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
        state_clone.borrow_mut().handle_mouse_pos(mouse_pos);
    });

    // React to mouse event (click) in render area
    let state_clone = state.clone();
    app.on_render_area_pointer_event(move |pe| {
        state_clone.borrow_mut().handle_pointer_event(pe);
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
