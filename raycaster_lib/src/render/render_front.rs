use std::{
    sync::{Arc, Mutex, RwLock},
    thread::JoinHandle,
};

use crossbeam::channel::{Receiver, Sender};

use crate::PerspectiveCamera;

pub enum RendererMessage {
    StartRendering,
    ShutDown,
}

// Interface for renderers running in different thread
pub trait RenderThread {
    fn get_shared_buffer(&self) -> Arc<Mutex<Vec<u8>>>;

    fn get_camera(&self) -> Arc<RwLock<PerspectiveCamera>>;

    fn start(self) -> JoinHandle<()>;

    fn set_communication(&mut self, communication: (Sender<()>, Receiver<RendererMessage>));
}

pub struct RendererFront {
    handle: Option<JoinHandle<()>>,
    buffer: Option<Arc<Mutex<Vec<u8>>>>,
    camera: Option<Arc<RwLock<PerspectiveCamera>>>,
    communication_in: (Sender<RendererMessage>, Receiver<RendererMessage>),
    communication_out: (Sender<()>, Receiver<()>), // todo passive wait to read buffer instead?
}

impl RendererFront {
    pub fn new() -> Self {
        let communication_in = crossbeam::channel::unbounded(); // main -> renderer
        let communication_out = crossbeam::channel::unbounded(); // renderer -> main
        Self {
            handle: None,
            buffer: None,
            camera: None,
            communication_in,
            communication_out,
        }
    }

    pub fn get_sender(&self) -> Sender<RendererMessage> {
        self.communication_in.0.clone()
    }

    pub fn send_message(&self, msg: RendererMessage) {
        self.communication_in.0.send(msg).unwrap()
    }

    pub fn get_communication_render_side(&self) -> (Sender<()>, Receiver<RendererMessage>) {
        (
            self.communication_out.0.clone(),
            self.communication_in.1.clone(),
        )
    }

    pub fn get_receiver(&self) -> Receiver<()> {
        self.communication_out.1.clone()
    }

    pub fn receive_message(&self) {
        self.communication_out.1.recv().unwrap()
    }

    pub fn get_buffer_handle(&self) -> Option<Arc<Mutex<Vec<u8>>>> {
        self.buffer.as_ref().cloned()
    }

    pub fn get_camera_handle(&self) -> Option<Arc<RwLock<PerspectiveCamera>>> {
        self.camera.as_ref().cloned()
    }

    pub fn start_rendering<R: RenderThread>(&mut self, mut renderer: R) {
        // Shutdown if needed
        if let Some(handle) = self.handle.take() {
            println!("Shutting down current renderer");
            self.communication_in
                .0
                .send(RendererMessage::ShutDown)
                .unwrap();
            handle.join().unwrap();
            self.buffer = None;
        }

        let communication = (
            self.communication_out.0.clone(),
            self.communication_in.1.clone(),
        );
        renderer.set_communication(communication);
        let buffer = renderer.get_shared_buffer();
        let camera = renderer.get_camera();
        let handle = renderer.start();
        self.buffer = Some(buffer);
        self.handle = Some(handle);
        self.camera = Some(camera);
    }

    pub fn finish(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap();
            self.buffer = None;
            self.handle = None;
            self.camera = None;
        }
    }
}

impl Default for RendererFront {
    fn default() -> Self {
        Self::new()
    }
}
