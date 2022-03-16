use std::{
    sync::{Arc, Mutex},
    thread::JoinHandle,
};

use crossbeam_channel::{Receiver, Sender};

pub enum RendererMessage {
    StartRendering,
    ShutDown,
}

// Interface for renderer
pub trait RenderThread {
    fn get_shared_buffer(&self) -> Arc<Mutex<Vec<u8>>>;

    fn start(self) -> JoinHandle<()>;

    fn set_communication(&mut self, communication: (Sender<()>, Receiver<RendererMessage>));
}

pub struct RendererFront {
    handle: Option<JoinHandle<()>>,
    buffer: Option<Arc<Mutex<Vec<u8>>>>,
    communication_in: (Sender<RendererMessage>, Receiver<RendererMessage>),
    communication_out: (Sender<()>, Receiver<()>),
}

impl RendererFront {
    pub fn new() -> Self {
        let communication_in = crossbeam_channel::unbounded(); // main -> renderer
        let communication_out = crossbeam_channel::unbounded(); // renderer -> main
        Self {
            handle: None,
            buffer: None,
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

    pub fn get_receiver(&self) -> Receiver<()> {
        self.communication_out.1.clone()
    }

    pub fn receive_message(&self) {
        self.communication_out.1.recv().unwrap()
    }

    pub fn get_buffer_handle(&self) -> Option<Arc<Mutex<Vec<u8>>>> {
        self.buffer.as_ref().cloned()
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
        let handle = renderer.start();
        self.buffer = Some(buffer);
        self.handle = Some(handle);
    }
}

impl Default for RendererFront {
    fn default() -> Self {
        Self::new()
    }
}
