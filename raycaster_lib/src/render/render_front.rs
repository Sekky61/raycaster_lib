use std::{sync::Arc, thread::JoinHandle};

use crossbeam::channel::{Receiver, Sender};
use parking_lot::{Mutex, RwLock};

use crate::PerspectiveCamera;

/// Messages to renderer
///
/// Messages queue up and one is read after frame is done
pub enum RendererMessage {
    /// Start rendering
    StartRendering,
    /// Start rendering a lower quality image
    StartRenderingFast,
    /// Shut down, thread will get ready to be joined
    ShutDown,
}

/// Interface for renderers running in different thread
///
/// Must be implemented by renderers that wish to communicate using
/// [`RendererFront`].
pub trait RenderThread {
    /// Get reference to shared framebuffer
    fn get_shared_buffer(&self) -> Arc<Mutex<Vec<u8>>>;

    /// Get reference to camera
    ///
    /// If you obtain write lock, you can change camera position
    fn get_camera(&self) -> Arc<RwLock<PerspectiveCamera>>;

    /// Spawn thread(s) with renderer
    ///
    /// Renderer waits for messages, does _not_ start rendering.
    /// Returns handle which can be used to sync with parent thread.
    fn start(self) -> JoinHandle<()>;

    /// Communication setter
    fn set_communication(&mut self, communication: (Sender<()>, Receiver<RendererMessage>));
}

/// Communicating with renderer
///
/// Can be active or inactive.
pub struct RendererFront {
    handle: Option<JoinHandle<()>>,
    buffer: Option<Arc<Mutex<Vec<u8>>>>,
    camera: Option<Arc<RwLock<PerspectiveCamera>>>,
    communication_in: (Sender<RendererMessage>, Receiver<RendererMessage>),
    communication_out: (Sender<()>, Receiver<()>), // todo passive wait to read buffer instead?
}

impl RendererFront {
    /// Create inactive front
    pub fn new() -> Self {
        let communication_in = crossbeam::channel::bounded(100); // main -> renderer
        let communication_out = crossbeam::channel::bounded(100); // renderer -> main
        Self {
            handle: None,
            buffer: None,
            camera: None,
            communication_in,
            communication_out,
        }
    }

    /// Getter for sender
    /// Returned struct can be used to send commands to renderer
    ///
    /// see `send()` method
    pub fn get_sender(&self) -> Sender<RendererMessage> {
        self.communication_in.0.clone()
    }

    /// Send message to renderer
    ///
    /// Equivalent to:
    /// ```
    /// let sender = front.get_sender();
    /// sender.send(msg);
    /// ```
    pub fn send_message(&self, msg: RendererMessage) {
        self.communication_in.0.send(msg).unwrap()
    }

    /// Getter for message receiver
    ///
    /// Receive messages from renderer.
    /// At the moment, the only message means new frame is ready and shared buffer can be obtained.
    pub fn get_receiver(&self) -> Receiver<()> {
        self.communication_out.1.clone()
    }

    /// Receive message from renderer
    ///
    /// Blocking call
    ///
    /// Equivalent to:
    /// ```
    /// let rec = front.get_receiver();
    /// rec.recv().unwrap(); // returns unit type
    /// ```
    pub fn receive_message(&self) {
        self.communication_out.1.recv().unwrap()
    }

    /// Getter for shared framebuffer
    /// If front is inactive, return `None`
    pub fn get_buffer_handle(&self) -> Option<Arc<Mutex<Vec<u8>>>> {
        self.buffer.as_ref().cloned()
    }

    /// Borrow buffer handle
    /// Avoids incrementing atomic reference counter
    /// Otherwise equivalent to `get_buffer_handle`
    pub fn get_buffer_handle_borrow(&self) -> Option<&Arc<Mutex<Vec<u8>>>> {
        self.buffer.as_ref()
    }

    /// Getter for camera handle
    /// /// If front is inactive, return `None`
    pub fn get_camera_handle(&self) -> Option<Arc<RwLock<PerspectiveCamera>>> {
        self.camera.as_ref().cloned()
    }

    /// Borrow camera handle
    /// Avoids incrementing atomic reference counter
    /// Otherwise equivalent to `get_camera_handle`
    pub fn get_camera_handle_borrow(&self) -> Option<&Arc<RwLock<PerspectiveCamera>>> {
        self.camera.as_ref()
    }

    /// Start `renderer`
    ///
    /// Front goes into active state.
    /// If front was already active, previous renderer gets shutdown first.
    ///
    /// Parameter `renderer` must implement `RenderThread`
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
        let handle = renderer.start(); // start thread but wait for startrendering message
        self.buffer = Some(buffer);
        self.handle = Some(handle);
        self.camera = Some(camera);
    }

    /// Sync thread with parent
    ///
    /// `ShutDown` message must be sent first separately.
    /// Call is blocking until thread is joined.
    /// Front goes into inactive state.
    pub fn finish(&mut self) {
        if let Some(handle) = self.handle.take() {
            // todo should it send shutdown on its own?
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
