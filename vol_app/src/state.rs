use std::{cell::RefCell, collections::VecDeque, path::PathBuf, rc::Rc, time::Instant};

use nalgebra::{vector, Vector2, Vector3};
use raycaster_lib::render::{RendererFront, RendererMessage};
use slint::{
    re_exports::{PointerEvent, PointerEventButton, PointerEventKind},
    Weak,
};

use super::App;

pub const RENDER_WIDTH_U: usize = 700;
pub const RENDER_HEIGHT_U: usize = 700;

pub const RENDER_WIDTH: u32 = RENDER_WIDTH_U as u32;
pub const RENDER_HEIGHT: u32 = RENDER_HEIGHT_U as u32;

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

pub struct State {
    pub app: Weak<App>,
    pub renderer_front: RendererFront,
    pub is_rendering: bool,
    pub camera_buffer: CameraBuffer,
    // GUI
    pub timer: Instant,
    pub slider: Vector3<f32>,
    pub left_mouse_held: bool,
    pub right_mouse_held: bool,
    pub mouse: Option<Vector2<f32>>,
}

impl State {
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
        }
    }

    pub fn new_shared(app: Weak<App>) -> Rc<RefCell<State>> {
        let state = State::new(app);
        Rc::new(RefCell::new(state))
    }

    pub fn render_thread_send_message(&self, msg: RendererMessage) {
        self.renderer_front.send_message(msg);
        println!("Sent order!!");
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

    pub fn handle_new_vol(&self, path: PathBuf) {
        todo!()
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

    fn start_render(&mut self) {
        self.is_rendering = true;
        self.render_thread_send_message(RendererMessage::StartRendering);
    }
}
