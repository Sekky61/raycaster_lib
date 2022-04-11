use std::{cell::RefCell, rc::Rc};

use nalgebra::vector;
use native_dialog::FileDialog;
use slint::{re_exports::EventResult, Image, Rgb8Pixel, SharedPixelBuffer};

use crate::app::State;

use crate::App;

/// Shared counted reference to `State`
///
/// Every callback needs a reference to state.
#[derive(Clone)]
pub struct StateRef(Rc<RefCell<State>>);

impl std::ops::Deref for StateRef {
    type Target = Rc<RefCell<State>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl StateRef {
    /// Wrap reference in Newtype
    pub fn new(inner: Rc<RefCell<State>>) -> StateRef {
        StateRef(inner)
    }

    /// Initialize renderer with default values and start rendering first frame
    pub fn initial_render_call(&mut self) {
        self.borrow_mut().initial_render_call();
    }

    /// Shuts down renderer
    ///
    /// Finishing function.
    /// Blocks until render thread is joined
    pub fn shutdown_renderer(&mut self) {
        self.borrow_mut().shutdown_renderer()
    }

    /// Register all callbacks from GUI
    ///
    /// # Params
    /// * `app` - reference to GUI
    pub fn register_callbacks(&mut self, app: &App) {
        // Callback
        // Invoked when new frame is rendered
        let state_clone = self.clone();
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
        let state_clone = self.clone();
        app.on_render_area_move_event(move |mouse_pos| {
            state_clone
                .borrow_mut()
                .handle_mouse_pos(vector![mouse_pos.x, mouse_pos.y]);
        });

        // Callback
        // React to mouse event (click) in render area
        let state_clone = self.clone();
        app.on_render_area_pointer_event(move |pe| {
            state_clone.borrow_mut().handle_pointer_event(pe);
        });

        // Callback
        // React to keyboard event
        let state_clone = self.clone();
        app.on_key_pressed(move |ke| {
            let ch = match ke.text.as_str().chars().next() {
                Some(ch) => ch,
                None => return EventResult::accept,
            };
            state_clone.borrow_mut().handle_key_press(ch);
            EventResult::accept
        });

        // Pick file button callback
        let state_clone = self.clone();
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
        let state_clone = self.clone();
        app.on_open_file(move || {
            let mut state = state_clone.borrow_mut();

            let app = state.get_app();
            let parser_gui_index = app.get_parser_picked_index();
            state.handle_open_vol(parser_gui_index);
        });

        // MT checkbox changed callback
        let state_clone = self.clone();
        app.on_mt_changed(move || {
            let mut state = state_clone.borrow_mut();
            let app = state.get_app();
            let checked = app.get_mt_checked();
            state.set_mt(checked);
        });

        // New TF selected callback
        let state_clone = self.clone();
        app.on_tf_selected(move |tf_name| {
            let mut state = state_clone.borrow_mut();
            state.handle_tf_changed(&tf_name);
        });

        // Callbacks for position sliders

        let state_clone = self.clone();
        app.on_x_slider_new_value(move |f| state_clone.borrow_mut().slider_event(0, f));

        let state_clone = self.clone();
        app.on_y_slider_new_value(move |f| state_clone.borrow_mut().slider_event(1, f));

        let state_clone = self.clone();
        app.on_z_slider_new_value(move |f| state_clone.borrow_mut().slider_event(2, f));
    }
}
