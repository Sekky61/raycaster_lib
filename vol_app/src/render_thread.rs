use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crossbeam_channel::{Receiver, Sender};
use nalgebra::{point, vector, Vector3};
use raycaster_lib::{
    camera::{Camera, PerspectiveCamera},
    premade::{
        parse::{from_file, skull_parser},
        transfer_functions::skull_tf,
    },
    render::{RenderOptions, Renderer},
    volumetric::{BlockVolume, LinearVolume, Volume},
};
use slint::re_exports::{PointerEvent, PointerEventButton, PointerEventKind};

use crate::{App, MousePos}; // todo

pub const RENDER_WIDTH_U: usize = 700;
pub const RENDER_HEIGHT_U: usize = 700;

pub const RENDER_WIDTH: u32 = RENDER_WIDTH_U as u32;
pub const RENDER_HEIGHT: u32 = RENDER_HEIGHT_U as u32;

pub struct State {
    pub can_start_rendering: bool,
    pub left_mouse_held: bool,
    pub right_mouse_held: bool,
    pub mouse_x: f32,
    pub mouse_y: f32,
}

impl State {
    fn new() -> State {
        State {
            can_start_rendering: true,
            left_mouse_held: false,
            right_mouse_held: false,
            mouse_x: 0.0,
            mouse_y: 0.0,
        }
    }
}

#[derive(PartialEq)]
pub enum RenderThreadMessage {
    StartRendering,
    ChangeResolution((usize, usize)),
    NewVolume(PathBuf),
    MousePos(MousePos),
    MouseClick(PointerEvent),
    ShutDown,
}

#[derive(Clone)]
pub struct RenderThreadMessageSender(Sender<RenderThreadMessage>);

impl RenderThreadMessageSender {
    pub fn new(sender: Sender<RenderThreadMessage>) -> Self {
        RenderThreadMessageSender(sender)
    }

    pub fn send_message(&self, message: RenderThreadMessage) {
        self.0
            .send(message)
            .expect("Cannot send message to render thread");
    }
}

pub struct RenderThread {
    state: State,
    app_weak: slint::Weak<App>,
    shared_buffer: Arc<Mutex<Vec<u8>>>,
    message_sender: Sender<RenderThreadMessage>,
    message_receiver: Receiver<RenderThreadMessage>, // todo handle all inputs, THEN start rendering
}

impl RenderThread {
    pub fn new(app_weak: slint::Weak<App>, shared_buffer: Arc<Mutex<Vec<u8>>>) -> Self {
        let (message_sender, message_receiver) = crossbeam_channel::unbounded();
        Self {
            state: State::new(),
            app_weak,
            shared_buffer,
            message_sender,
            message_receiver,
        }
    }

    pub fn get_sender(&self) -> RenderThreadMessageSender {
        RenderThreadMessageSender(self.message_sender.clone())
    }

    pub fn start(mut self) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || {
            let volume: BlockVolume =
                from_file("volumes/Skull.vol", skull_parser, skull_tf).unwrap();

            let pos = point![700.0, 700.0, 700.0];
            let dir = vector![-1.0, -1.0, -1.0];
            let camera = PerspectiveCamera::new(pos, dir);

            let mut raycast_renderer = Renderer::<_, _>::new(volume, camera);

            raycast_renderer.set_render_options(RenderOptions {
                resolution: (RENDER_WIDTH_U, RENDER_HEIGHT_U),
                ray_termination: true,
                empty_index: true,
            });

            loop {
                {
                    let mut lock = self.shared_buffer.lock().unwrap();
                    raycast_renderer.render_to_buffer(lock.as_mut_slice());
                    // drop of mutex
                }

                self.notify_frame_rendered();

                let res = self.get_user_input(&mut raycast_renderer);
                if res {
                    continue;
                } else {
                    break;
                }
            }
        })
    }

    // todo can be replaced with mpsc
    fn notify_frame_rendered(&mut self) {
        let handle_copy = self.app_weak.clone();
        self.state.can_start_rendering = false;
        slint::invoke_from_event_loop(move || handle_copy.unwrap().invoke_send_rendered_frame());
    }

    // todo dont use bool
    // todo build translation matrix in different thread, just apply it here and continue rendering
    fn get_user_input<V: Volume>(&mut self, ren: &mut Renderer<V, PerspectiveCamera>) -> bool {
        loop {
            let event = self.message_receiver.try_recv();

            let event = match event {
                Ok(e) => e,
                Err(_) => {
                    // no more commands
                    if self.state.can_start_rendering {
                        return true;
                    } else {
                        continue; // spin
                    }
                }
            };

            match event {
                RenderThreadMessage::StartRendering => self.state.can_start_rendering = true,
                RenderThreadMessage::ChangeResolution(res) => ren.set_render_resolution(res),
                RenderThreadMessage::MouseClick(pe) => self.handle_pointer_event(pe),
                RenderThreadMessage::MousePos(m) => self.handle_mouse_pos(&mut ren.camera, m),
                RenderThreadMessage::ShutDown => return false,
                RenderThreadMessage::NewVolume(path) => self.handle_new_volume(ren, path),
            }
        }
    }

    fn handle_mouse_pos(&mut self, cam: &mut PerspectiveCamera, action: MousePos) {
        // rust-analyzer struggles here because m is of generated type
        // The type is (f32, f32)
        let drag_diff = (self.state.mouse_x - action.x, self.state.mouse_y - action.y); // todo reset mouse_xy when click
        self.state.mouse_x = action.x;
        self.state.mouse_y = action.y;
        match (self.state.left_mouse_held, self.state.right_mouse_held) {
            (false, false) => (),
            (true, false) => {
                // move on the plane described by camera position and normal
                cam.change_pos_plane(drag_diff.0 * 0.2, drag_diff.1 * 0.2);
            }
            (false, true) => {
                // change camera direction
                cam.look_around(drag_diff.0 * -0.001, drag_diff.1 * -0.001);
            }
            (true, true) => {
                // rotate around origin
                let axisangle = Vector3::y() * (std::f32::consts::FRAC_PI_8 * drag_diff.0);
                let rot = nalgebra::Rotation3::new(axisangle);

                cam.change_pos_matrix(rot);
            }
        }
    }

    fn handle_pointer_event(&mut self, pe: PointerEvent) {
        match pe {
            PointerEvent {
                button: PointerEventButton::left,
                kind: PointerEventKind::up,
            } => self.state.left_mouse_held = false,
            PointerEvent {
                button: PointerEventButton::left,
                kind: PointerEventKind::down,
            } => self.state.left_mouse_held = true,
            PointerEvent {
                button: PointerEventButton::right,
                kind: PointerEventKind::up,
            } => self.state.right_mouse_held = false,
            PointerEvent {
                button: PointerEventButton::right,
                kind: PointerEventKind::down,
            } => self.state.right_mouse_held = true,
            _ => (),
        }
    }

    fn handle_new_volume<V: Volume>(
        &self,
        ren: &mut Renderer<V, PerspectiveCamera>,
        path: PathBuf,
    ) {
        todo!()
    }
}
