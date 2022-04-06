use std::{
    sync::{Arc, Mutex, RwLock},
    thread::JoinHandle,
};

use crossbeam::channel::{Receiver, Sender};

use crate::{volumetric::Volume, PerspectiveCamera};

use super::{render_front::RenderThread, RenderOptions, Renderer, RendererMessage};

pub struct SerialRenderer<V>
where
    V: Volume + 'static,
{
    volume: V,
    shared_buffer: Arc<Mutex<Vec<u8>>>,
    camera: Arc<RwLock<PerspectiveCamera>>,
    render_options: RenderOptions,
    communication: (Sender<()>, Receiver<RendererMessage>),
}

impl<V> RenderThread for SerialRenderer<V>
where
    V: Volume + 'static,
{
    fn get_shared_buffer(&self) -> Arc<Mutex<Vec<u8>>> {
        self.shared_buffer.clone()
    }

    fn get_camera(&self) -> Arc<RwLock<PerspectiveCamera>> {
        self.camera.clone()
    }

    fn start(self) -> JoinHandle<()> {
        self.start_rendering()
    }

    fn set_communication(&mut self, communication: (Sender<()>, Receiver<RendererMessage>)) {
        self.communication = communication;
    }
}

impl<V> SerialRenderer<V>
where
    V: Volume,
{
    pub fn new(
        volume: V,
        camera: Arc<RwLock<PerspectiveCamera>>,
        render_options: RenderOptions,
    ) -> Self {
        let elements =
            (render_options.resolution.x as usize) * (render_options.resolution.y as usize);
        let buffer = Arc::new(Mutex::new(vec![0; elements * 3]));

        // Dummy channels
        // Replaced once started
        let (sender_void, _) = crossbeam::channel::unbounded();
        let never = crossbeam::channel::never();
        let communication = (sender_void, never);

        Self {
            communication,
            volume,
            shared_buffer: buffer,
            camera,
            render_options,
        }
    }

    pub fn start_rendering(self) -> JoinHandle<()> {
        std::thread::spawn(move || {
            let mut renderer = Renderer::new(self.volume, self.render_options);
            // Master loop
            loop {
                // Gather input
                let msg = self.communication.1.recv().unwrap();
                match msg {
                    RendererMessage::StartRendering => (),
                    RendererMessage::ShutDown => break,
                }

                {
                    // Lock buffer
                    let mut buffer = self.shared_buffer.lock().unwrap();

                    // Lock camera
                    let camera = self.camera.read().unwrap();

                    // Render
                    renderer.render_to_buffer(&camera, &mut buffer[..]);
                }

                // Send result
                self.communication.0.send(()).unwrap();
            }
        })
    }
}
