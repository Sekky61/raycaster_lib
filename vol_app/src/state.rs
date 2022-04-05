use std::{
    cell::RefCell,
    collections::VecDeque,
    path::{Path, PathBuf},
    rc::Rc,
    sync::{Arc, RwLock},
    time::Instant,
};

use nalgebra::{point, vector, Vector2, Vector3};
use raycaster_lib::{
    premade::{
        parse::{from_file, generator_parser, skull_parser},
        transfer_functions,
    },
    render::{ParalelRenderer, RenderOptions, RendererFront, RendererMessage, SerialRenderer},
    volumetric::LinearVolume,
    PerspectiveCamera,
};
use slint::{
    re_exports::{PointerEvent, PointerEventButton, PointerEventKind},
    Weak,
};

use super::App;

pub const RENDER_WIDTH_U: usize = 700;
pub const RENDER_HEIGHT_U: usize = 700;

pub const DEFAULT_VOLUME_PATH: &str = "volumes/a.vol";
pub const DEFAULT_VOLUME_PARSER: PrewrittenParser = PrewrittenParser::MyVolParser;
const DEFAULT_MULTI_THREAD: bool = true;

pub enum CameraMovement {
    PositionOrtho(Vector3<f32>),
    PositionPlane(Vector2<f32>),
    Direction(Vector2<f32>),
    PositionInDir(f32),
}

pub struct CameraBuffer {
    buffer: VecDeque<CameraMovement>,
}

impl CameraBuffer {
    pub fn new() -> Self {
        let buffer = VecDeque::new();
        Self { buffer }
    }

    pub fn add_movement(&mut self, movement: CameraMovement) {
        self.buffer.push_back(movement);
    }
}

pub enum PrewrittenParser {
    MyVolParser,
    SkullParser,
}

#[derive(Clone, Copy)]
pub enum PrewrittenTF {
    Green,
    Gray,
}

pub struct State {
    pub app: Weak<App>,
    pub renderer_front: RendererFront,
    pub is_rendering: bool,
    pub camera_buffer: CameraBuffer,
    pub multi_thread: bool,
    pub render_resolution: Vector2<usize>,
    // GUI
    pub timer: Instant,
    pub slider: Vector3<f32>,
    pub left_mouse_held: bool,
    pub right_mouse_held: bool,
    pub mouse: Option<Vector2<f32>>,
    // Vol picker
    pub file_picked: Option<PathBuf>,
    pub parser_picked: Option<PrewrittenParser>,
    pub current_tf: PrewrittenTF,
}

impl State {
    /// Default values for state
    ///
    /// # Params
    ///
    /// * app - reference to GUI, see `App::as_weak`
    pub fn new(app: Weak<App>) -> State {
        let renderer_front = RendererFront::new();

        State {
            app,
            renderer_front,
            is_rendering: false,
            camera_buffer: CameraBuffer::new(),
            left_mouse_held: false,
            right_mouse_held: false,
            mouse: None,
            timer: Instant::now(),
            slider: Default::default(),
            file_picked: None,
            parser_picked: None,
            multi_thread: DEFAULT_MULTI_THREAD,
            current_tf: PrewrittenTF::Green,
            render_resolution: vector![RENDER_WIDTH_U, RENDER_HEIGHT_U],
        }
    }

    pub fn new_shared(app: Weak<App>) -> Rc<RefCell<State>> {
        let state = State::new(app);
        Rc::new(RefCell::new(state))
    }

    pub fn render_thread_send_message(&self, msg: RendererMessage) {
        self.renderer_front.send_message(msg);
    }

    fn new_camera_movement(&mut self, movement: CameraMovement) {
        self.camera_buffer.add_movement(movement);
        if !self.is_rendering {
            self.apply_cam_change();
            self.start_render();
        }
    }

    pub fn slider_event(&mut self, slider_id: u8, slider: f32) {
        let delta = match slider_id {
            0 => {
                let res = vector![slider - self.slider.x, 0.0, 0.0];
                self.slider.x = slider;
                res
            }
            1 => {
                let res = vector![0.0, slider - self.slider.y, 0.0];
                self.slider.y = slider;
                res
            }
            2 => {
                let res = vector![0.0, 0.0, slider - self.slider.z];
                self.slider.z = slider;
                res
            }
            _ => panic!("Bad slider id, todo enum"),
        };
        self.new_camera_movement(CameraMovement::PositionOrtho(delta));
    }

    pub fn handle_mouse_pos(&mut self, action: Vector2<f32>) {
        // rust-analyzer struggles here because m is of generated type
        // The type is (f32, f32)

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
                self.new_camera_movement(CameraMovement::PositionPlane(delta))
            }
            (false, true) => {
                // change camera direction
                let delta = vector![drag_diff.x * 0.01, drag_diff.y * -0.01];
                self.new_camera_movement(CameraMovement::Direction(delta))
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

    // todo pointer style move
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

    pub fn handle_key_press(&mut self, ch: char) {
        match ch {
            '+' => self.new_camera_movement(CameraMovement::PositionInDir(5.0)),
            '-' => self.new_camera_movement(CameraMovement::PositionInDir(-5.0)),
            _ => (),
        }
    }

    fn apply_cam_change(&mut self) {
        let camera = self.renderer_front.get_camera_handle().unwrap();
        {
            let mut camera = camera.write().unwrap();
            while let Some(movement) = self.camera_buffer.buffer.pop_front() {
                match movement {
                    CameraMovement::PositionOrtho(d) => camera.change_pos(d),
                    CameraMovement::PositionPlane(d) => camera.change_pos_plane(d),
                    CameraMovement::Direction(d) => camera.look_around(d),
                    CameraMovement::PositionInDir(d) => camera.change_pos_view_dir(d),
                }
            }
            // Drop Write camera guard
        }
    }

    // Instruct renderer to start rendering next frame
    fn start_render(&mut self) {
        // todo rename
        self.is_rendering = true;
        self.render_thread_send_message(RendererMessage::StartRendering);
        self.timer = Instant::now();
    }

    pub fn initial_render_call(&mut self) {
        self.start_renderer(DEFAULT_VOLUME_PATH.into(), DEFAULT_VOLUME_PARSER);
    }

    pub fn handle_open_vol(&mut self, parser_index: i32) {
        // Is file and parser picked?
        let path_picked = self.file_picked.is_some();
        let parser_picked = parser_index != -1;

        // Is parser selected?
        if !path_picked || !parser_picked {
            return;
        }

        // Take both
        let path = match self.file_picked.take() {
            Some(path) => path,
            None => return, // todo error
        };
        self.parser_picked = None;

        // Display new
        let parser = match parser_index {
            0 => PrewrittenParser::MyVolParser,
            1 => PrewrittenParser::SkullParser,
            _ => panic!("Unexpected parser"),
        };

        self.start_renderer(path, parser);
    }

    fn start_renderer(&mut self, path: PathBuf, parser: PrewrittenParser) {
        if self.multi_thread {
            let renderer = volume_setup_paralel(&path, parser, self.current_tf);
            self.renderer_front.start_rendering(renderer);
        } else {
            let renderer = volume_setup_linear(&path, parser, self.current_tf);
            self.renderer_front.start_rendering(renderer);
        }
        self.renderer_front
            .send_message(RendererMessage::StartRendering);
        println!(
            "Started renderer: {} | {path:#?}",
            if self.multi_thread { "MT" } else { "ST" }
        );
    }

    pub fn handle_tf_changed(&mut self, tf_name: &str) {
        let tf = match tf_name {
            "Green" => PrewrittenTF::Green,
            "Gray" => PrewrittenTF::Gray,
            _ => panic!("Unknown transfer function '{tf_name}'"),
        };
        self.current_tf = tf;
        self.start_renderer(DEFAULT_VOLUME_PATH.into(), PrewrittenParser::SkullParser);
    }
}

fn volume_setup_paralel(
    path: &Path,
    parser: PrewrittenParser,
    tf: PrewrittenTF,
) -> ParalelRenderer {
    let position = point![300.0, 300.0, 300.0];
    let direction = point![34.0, 128.0, 128.0] - position;

    let parser_fn = match parser {
        PrewrittenParser::MyVolParser => generator_parser,
        PrewrittenParser::SkullParser => skull_parser,
    };

    let tf_fn = match tf {
        PrewrittenTF::Green => transfer_functions::skull_tf,
        PrewrittenTF::Gray => transfer_functions::anything_tf,
    };

    let volume = from_file(path, parser_fn, tf_fn).unwrap();

    let camera = PerspectiveCamera::new(position, direction);
    let camera = Arc::new(RwLock::new(camera));

    let render_options = RenderOptions::new((RENDER_WIDTH_U, RENDER_HEIGHT_U), true, true);

    ParalelRenderer::new(volume, camera, render_options)
}

fn volume_setup_linear(
    path: &Path,
    parser: PrewrittenParser,
    tf: PrewrittenTF,
) -> SerialRenderer<LinearVolume> {
    let position = point![300.0, 300.0, 300.0];
    let direction = point![34.0, 128.0, 128.0] - position; // vector![-0.8053911, -0.357536, -0.47277182]

    let parser_fn = match parser {
        PrewrittenParser::MyVolParser => generator_parser,
        PrewrittenParser::SkullParser => skull_parser,
    };

    let tf_fn = match tf {
        PrewrittenTF::Green => transfer_functions::skull_tf,
        PrewrittenTF::Gray => transfer_functions::anything_tf,
    };

    let volume = from_file(path, parser_fn, tf_fn).unwrap();
    //let volume = from_file("volumes/a.vol", generator_parser, anything_tf).unwrap();

    let camera = PerspectiveCamera::new(position, direction);
    let camera = Arc::new(RwLock::new(camera));

    let render_options = RenderOptions::new((RENDER_WIDTH_U, RENDER_HEIGHT_U), true, false);

    SerialRenderer::new(volume, camera, render_options)
}
