use std::{sync::Arc, thread::JoinHandle};

use crossbeam::channel::{Receiver, Sender};
use parking_lot::{Mutex, RwLock};

use crate::{
    render::{render_front::RenderThread, RenderOptions, RendererMessage},
    volumetric::{Blocked, Volume},
    PerspectiveCamera,
};

use super::{communication::CommsBuilder, messages::ToWorkerMsg};
use super::{
    composition::Canvas,
    workers::{CompWorker, RenderWorker},
};

const RENDERER_COUNT: u8 = 5; // todo dynamic
const COMPOSITER_COUNT: u8 = 1;
const WORKER_COUNT: u8 = RENDERER_COUNT + COMPOSITER_COUNT;

const TILE_SIDE: u16 = 32;

pub struct ParalelRenderer<BV>
where
    BV: Volume + Blocked,
{
    volume: BV,
    camera: Arc<RwLock<PerspectiveCamera>>, // In read mode during the render, write inbetween renders
    render_options: RenderOptions,
    buffer: Arc<Mutex<Vec<u8>>>,
    communication: (Sender<()>, Receiver<RendererMessage>),
}

impl<BV> RenderThread for ParalelRenderer<BV>
where
    BV: Volume + Blocked + 'static,
{
    fn get_shared_buffer(&self) -> Arc<Mutex<Vec<u8>>> {
        self.buffer.clone()
    }

    fn get_camera(&self) -> Arc<RwLock<PerspectiveCamera>> {
        self.camera.clone()
    }

    fn start(self) -> JoinHandle<()> {
        println!("Starting renderer | {}", self.volume.get_name());
        self.start_rendering()
    }

    fn set_communication(&mut self, communication: (Sender<()>, Receiver<RendererMessage>)) {
        self.communication = communication;
    }
}

impl<BV> ParalelRenderer<BV>
where
    BV: Volume + Blocked + 'static,
{
    pub fn new(
        volume: BV,
        camera: Arc<RwLock<PerspectiveCamera>>,
        render_options: RenderOptions,
    ) -> Self {
        let elements: usize =
            (render_options.resolution.x as usize) * (render_options.resolution.y as usize);
        let buffer = Arc::new(Mutex::new(vec![0; elements * 3]));

        // Dummy channels
        // Replaced once started
        let (sender_void, _) = crossbeam::channel::unbounded();
        let never = crossbeam::channel::never();
        let communication = (sender_void, never);

        Self {
            volume,
            camera,
            render_options,
            buffer,
            communication,
        }
    }

    pub fn start_rendering(self) -> JoinHandle<()> {
        std::thread::spawn(move || {
            // Scope assures threads will be joined before exiting the scope
            crossbeam::scope(|s| {
                let volume = &self.volume;

                let canvas = Arc::new(Canvas::new(self.render_options.resolution, TILE_SIDE));

                #[cfg(debug_assertions)]
                println!("Master : build workers");

                // inlined function because borrow checker defeated me (scope cannot leave closure)
                let (master_comms, ren_handles, comp_handles) = {
                    // R, C, RC
                    let comms = CommsBuilder::new(WORKER_COUNT as usize);

                    let mut renderers = Vec::with_capacity(RENDERER_COUNT as usize);
                    let mut compositors = Vec::with_capacity(COMPOSITER_COUNT as usize);

                    for id in 0..RENDERER_COUNT {
                        // Create render thread
                        let ren_comms = comms.renderer(id as usize);
                        let camera_ref = self.camera.clone();
                        let handle = s
                            .builder()
                            .name(format!("Ren{id}"))
                            .spawn(move |_| {
                                #[cfg(debug_assertions)]
                                println!("Started renderer {id}");

                                let mut renderer = RenderWorker::new(
                                    id as usize,
                                    camera_ref,
                                    self.render_options,
                                    ren_comms,
                                    volume,
                                );

                                renderer.run();
                            })
                            .unwrap();

                        renderers.push(handle);
                    }

                    for id in RENDERER_COUNT..(RENDERER_COUNT + COMPOSITER_COUNT) {
                        // Create compositor thread

                        let comp_comms = comms.compositor(id as usize);
                        let canvas = canvas.clone();
                        let buffer = self.buffer.clone();

                        let handle = s
                            .builder()
                            .name(format!("Com{id}"))
                            .spawn(move |_| {
                                #[cfg(debug_assertions)]
                                println!("Started compositor {id}");

                                let compositor = CompWorker::new(
                                    id as u8,
                                    COMPOSITER_COUNT,
                                    canvas,
                                    buffer,
                                    comp_comms,
                                );

                                compositor.run();
                            })
                            .unwrap();

                        compositors.push(handle);
                    }

                    (comms.master(), renderers, compositors)
                };

                #[cfg(debug_assertions)]
                println!("Master : entering main loop");

                // Master loop
                loop {
                    // Gather input
                    #[cfg(debug_assertions)]
                    println!("Master : waiting for input");
                    let msg = self.communication.1.recv().unwrap();

                    let quality = match msg {
                        RendererMessage::StartRendering => true,
                        RendererMessage::StartRenderingFast => false,
                        RendererMessage::ShutDown => break,
                    };

                    #[cfg(debug_assertions)]
                    println!("Master : start rendering");

                    // Prepare canvas (mainly queues)
                    {
                        let blocks = volume.get_blocks();
                        let empty_blocks_index = volume.get_empty_blocks();
                        let camera = self.camera.read();

                        canvas.build_queues(
                            &camera,
                            blocks,
                            empty_blocks_index,
                            self.render_options,
                        );
                    }

                    #[cfg(debug_assertions)]
                    println!("Master : queues built");

                    // Send go live messages
                    for worker in master_comms.command_sender.iter() {
                        worker.send(ToWorkerMsg::GoLive { quality }).unwrap();
                    }

                    #[cfg(debug_assertions)]
                    println!("Master : workers ordered to work, waiting for canvas");

                    // Wait for rendered frame
                    master_comms.result_receiver.recv().unwrap();

                    // Send result
                    self.communication.0.send(()).unwrap();

                    // Send finish messages and join threads
                    for worker in master_comms.command_sender.iter() {
                        worker.send(ToWorkerMsg::GoIdle).unwrap();
                    }

                    #[cfg(debug_assertions)]
                    println!("Master : result sent");
                }

                // Send finish messages and join threads
                for worker in master_comms.command_sender.iter() {
                    worker.send(ToWorkerMsg::Finish).unwrap();
                }

                for h in ren_handles {
                    h.join().unwrap();
                }
                for h in comp_handles {
                    h.join().unwrap();
                }
            })
            .unwrap();
        })
    }
}
