use std::{cell::RefCell, path::PathBuf, rc::Rc, sync::Arc};

use crossbeam_channel::Receiver;
use nalgebra::{vector, Vector2, Vector3};
use parking_lot::Mutex;
use raycaster_lib::{render::RendererMessage, volumetric::MemoryType};
use slint::{
    re_exports::{PointerEvent, PointerEventButton, PointerEventKind},
    Weak,
};

use super::{
    common::{CameraMovement, PrewrittenParser, PrewrittenTF},
    defaults,
    render_state::{PickedMemoryType, RenderQualitySettings},
    RenderState, StateRef,
};

use crate::App;

/// Application state
/// All fields are directly accessible
pub struct State {
    rendering: RenderState,
    // GUI
    app: Weak<App>,
    slider: Vector3<f32>,
    left_mouse_held: bool,
    right_mouse_held: bool,
    mouse: Option<Vector2<f32>>,
    // Current volume
    current_vol_path: PathBuf,
    current_parser: PrewrittenParser,
    current_memory_type: PickedMemoryType,
    // Vol picker
    file_picked: Option<PathBuf>,
    parser_picked: Option<PrewrittenParser>, // todo current parser to fix switching TF
    current_tf: PrewrittenTF,
}

impl State {
    /// Default values for state
    ///
    /// # Params
    ///
    /// * app - reference to GUI, see `App::as_weak`
    pub fn new(app: Weak<App>) -> State {
        State {
            app,
            left_mouse_held: false,
            right_mouse_held: false,
            mouse: None,
            slider: Default::default(),
            file_picked: None,
            parser_picked: None,
            current_tf: PrewrittenTF::Skull,
            rendering: RenderState::new(),
            current_vol_path: defaults::VOLUME_PATH.into(),
            current_parser: defaults::VOLUME_PARSER,
            current_memory_type: defaults::MEMORY_TYPE,
        }
    }

    pub fn new_shared(app: Weak<App>) -> StateRef {
        let state = State::new(app);
        StateRef::new(Rc::new(RefCell::new(state)))
    }

    /// Get handle to GUI
    ///
    /// Panics if called from a thread different than main thread
    pub fn get_app(&self) -> App {
        self.app.upgrade().unwrap()
    }

    /// Get handle to shared buffer
    ///
    /// Returns reference, to avoid atomic operation.
    pub fn get_buffer_handle(&self) -> &Arc<Mutex<Vec<u8>>> {
        self.rendering
            .renderer_front
            .get_buffer_handle_borrow()
            .unwrap()
    }

    /// Handle data from GUI sliders
    ///
    /// # Params
    /// * `slider_id` - ID of a slider (axis)
    /// * `value` - value of the slider
    pub fn slider_event(&mut self, slider_id: u8, value: f32) {
        let delta = match slider_id {
            0 => {
                let res = vector![value - self.slider.x, 0.0, 0.0];
                self.slider.x = value;
                res
            }
            1 => {
                let res = vector![0.0, value - self.slider.y, 0.0];
                self.slider.y = value;
                res
            }
            2 => {
                let res = vector![0.0, 0.0, value - self.slider.z];
                self.slider.z = value;
                res
            }
            _ => panic!("Bad slider id, todo enum"),
        };
        self.rendering
            .register_movement(CameraMovement::PositionOrtho(delta));
    }

    /// Queue up camera movements based on mouse actions
    pub fn handle_mouse_pos(&mut self, action: Vector2<f32>) {
        let drag_diff = if let Some(base) = self.mouse {
            action - base
        } else {
            self.mouse = Some(vector![action.x, action.y]);
            return;
        };

        self.mouse = Some(action);

        match (self.left_mouse_held, self.right_mouse_held) {
            (false, false) => (),
            (true, false) => {
                // move on the plane described by camera position and normal
                let delta = vector![drag_diff.x * -0.2, drag_diff.y * 0.2];
                self.rendering
                    .register_movement(CameraMovement::PositionPlane(delta));
            }
            (false, true) => {
                // change camera direction
                let delta = vector![drag_diff.x * 0.01, drag_diff.y * -0.01];
                self.rendering
                    .register_movement(CameraMovement::Direction(delta));
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

    /// Handle change in pressed mouse buttons
    pub fn handle_pointer_event(&mut self, pe: PointerEvent) {
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

    /// handle keyboard presses
    ///
    /// At the moment, only `+` and `-` have an effect
    pub fn handle_key_press(&mut self, ch: char) {
        match ch {
            '+' => self
                .rendering
                .register_movement(CameraMovement::PositionInDir(5.0)),
            '-' => self
                .rendering
                .register_movement(CameraMovement::PositionInDir(-5.0)),
            _ => (),
        }
    }

    /// Start renderer with default values
    pub fn initial_render_call(&mut self) {
        self.rendering.start_renderer(
            &self.current_vol_path,
            self.current_parser,
            self.current_memory_type,
        );
    }

    /// Setter
    pub fn set_file_picked(&mut self, file: PathBuf) {
        self.file_picked = Some(file);

        let picked_vol_path = match self.file_picked {
            Some(ref p) => p.clone().into_os_string().into_string().unwrap(),
            None => "Nothing picked".into(),
        };
        self.app.unwrap().set_path_text(picked_vol_path.into());
    }

    /// Start new renderer based on user input
    pub fn handle_open_vol(&mut self, parser_index: i32, memory_index: i32) {
        // Is file and parser picked?
        let path_picked = self.file_picked.is_some();
        let parser_picked = parser_index != -1;
        let memory_picked = memory_index != -1;

        // Is parser selected?
        if !path_picked || !parser_picked || !memory_picked {
            return;
        }

        // Take both
        let path = match self.file_picked.take() {
            Some(path) => path,
            None => return, // todo error
        };
        self.parser_picked = None;
        self.app.unwrap().set_path_text("Nothing picked".into());

        // Display new
        let parser = match parser_index {
            0 => PrewrittenParser::MyVolParser,
            1 => PrewrittenParser::SkullParser,
            _ => panic!("Unexpected parser"),
        };

        let memory_type = match memory_index {
            0 => PickedMemoryType::Stream,
            1 => PickedMemoryType::Ram,
            2 => PickedMemoryType::RamFloat,
            _ => panic!("Unexpected memory type picked"),
        };

        self.current_vol_path = path;
        self.current_parser = parser;
        self.current_memory_type = memory_type;

        self.rendering.start_renderer(
            &self.current_vol_path,
            self.current_parser,
            self.current_memory_type,
        );
    }

    /// Restart renderer with new transfer function
    pub fn handle_tf_changed(&mut self, tf_name: &str) {
        // todo check if works
        let tf = match tf_name {
            "Skull" => PrewrittenTF::Skull,
            "Gray" => PrewrittenTF::Gray,
            "White" => PrewrittenTF::White,
            "Shapes" => PrewrittenTF::Shapes,
            _ => panic!("Unknown transfer function '{tf_name}'"),
        };
        self.current_tf = tf;
        self.rendering.current_tf = tf;
        self.rendering.start_renderer(
            &self.current_vol_path,
            self.current_parser,
            self.current_memory_type,
        );
    }

    /// New quality setting picked by user
    pub fn handle_quality_changed(&mut self) {
        let app = self.app.unwrap();
        let q_int = app.get_render_quality_mode();
        self.rendering.render_quality_preference = RenderQualitySettings::from_gui_int(q_int);
    }

    pub fn get_renderer_receiver(&self) -> Receiver<()> {
        self.rendering.renderer_front.get_receiver()
    }

    pub fn get_resolution(&self) -> Vector2<u16> {
        self.rendering.render_options.resolution
    }

    /// Displays frame time and checks if camera has moved since last render
    /// Called after receiving a rendered frame
    pub fn handle_rendering_finished(&mut self) {
        self.rendering.is_rendering = false;

        // Frame time counter
        let elapsed = self.rendering.render_time.elapsed();
        self.app.upgrade_in_event_loop(move |app| {
            app.set_frame_time(elapsed.as_millis().try_into().unwrap())
        });

        // Check missed inputs
        self.rendering.check_render_conditions();
    }

    pub fn set_mt(&mut self, multi_thread: bool) {
        self.rendering.multi_thread = multi_thread;
    }

    pub fn set_ert(&mut self, ert: bool) {
        self.rendering.render_options.early_ray_termination = ert;
    }

    pub fn set_ei(&mut self, ei: bool) {
        self.rendering.render_options.empty_space_skipping = ei;
    }

    pub fn sync_state_with_gui(&self) {
        let app = self.app.upgrade().unwrap();
        let mt = self.rendering.multi_thread;
        let ert = self.rendering.render_options.early_ray_termination;
        let ei = self.rendering.render_options.empty_space_skipping;
        let render_quality = self.rendering.render_quality_preference.to_gui_int();
        let picked_vol_path = match self.file_picked {
            Some(ref p) => p.clone().into_os_string().into_string().unwrap(),
            None => "Nothing picked".into(),
        };

        let tf = self.rendering.current_tf.get_name();
        let parser_index = self.current_parser.get_gui_index();
        let memory_index = self.current_memory_type.get_gui_index();

        app.set_mt_checked(mt);
        app.set_ert_checked(ert);
        app.set_ei_checked(ei);
        app.set_tf_current_value(tf.into());

        app.set_path_text(picked_vol_path.into());
        app.set_parser_picked_index(parser_index);
        app.set_memory_picked_index(memory_index);

        app.set_render_quality_mode(render_quality);
    }

    /// Shutdown renderer
    /// Blocks until thread is joined
    pub fn shutdown_renderer(&mut self) {
        self.rendering
            .renderer_front
            .send_message(RendererMessage::ShutDown);

        self.rendering.renderer_front.finish();
    }
}
