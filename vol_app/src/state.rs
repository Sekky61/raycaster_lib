use std::{cell::RefCell, rc::Rc, time::Instant};

use crate::render_thread::{RenderThreadMessage, RenderThreadMessageSender};
use nalgebra::{vector, Vector2, Vector3};
use slint::re_exports::{PointerEvent, PointerEventButton, PointerEventKind};

pub struct State {
    pub sender: RenderThreadMessageSender,
    pub timer: Instant,
    pub slider: Vector3<f32>,
    pub left_mouse_held: bool,
    pub right_mouse_held: bool,
    pub mouse: Option<Vector2<f32>>,
}

impl State {
    pub fn new(sender: RenderThreadMessageSender) -> State {
        State {
            sender,
            left_mouse_held: false,
            right_mouse_held: false,
            mouse: None,
            timer: Instant::now(),
            slider: Default::default(),
        }
    }

    pub fn new_shared(sender: RenderThreadMessageSender) -> Rc<RefCell<State>> {
        let state = State::new(sender);
        Rc::new(RefCell::new(state))
    }

    pub fn render_thread_send_message(&self, message: RenderThreadMessage) {
        self.sender.send_message(message);
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
        self.sender
            .send_message(RenderThreadMessage::CameraChangePositionOrtho(delta));
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
                let delta = vector![drag_diff.x * -0.2, drag_diff.y * -0.2];
                self.sender
                    .send_message(RenderThreadMessage::CameraChangePositionPlane(delta));
            }
            (false, true) => {
                // change camera direction
                let delta = vector![drag_diff.x * 0.01, drag_diff.y * 0.01];
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
            '+' => self
                .sender
                .send_message(RenderThreadMessage::CameraChangePositionInDir(5.0)),
            '-' => self
                .sender
                .send_message(RenderThreadMessage::CameraChangePositionInDir(-5.0)),
            _ => (),
        }
    }
}
