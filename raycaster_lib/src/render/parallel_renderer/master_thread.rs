/*
    raycaster_lib
    Author: Michal Majer
    Date: 2022-05-05
*/

use std::{cell::UnsafeCell, ops::DerefMut, sync::Arc, thread::JoinHandle};

use crossbeam::channel::{Receiver, Sender};
use parking_lot::Mutex;

use crate::{
    render::{render_front::RenderThread, RenderOptions, RendererMessage},
    volumetric::{Blocked, Volume},
    PerspectiveCamera,
};

use super::{
    communication::CommsBuilder,
    composition::Canvas,
    messages::ToWorkerMsg,
    workers::{CompWorker, RenderWorker},
};

/// Parameters determined with experiments for 6c12t CPU
/// todo dynamic detection
const RENDERER_COUNT: u8 = 9;
const COMPOSITER_COUNT: u8 = 1;
const WORKER_COUNT: u8 = RENDERER_COUNT + COMPOSITER_COUNT;
const TILE_SIDE: u16 = 10;

/// Parallel renderer's entity for being controled by [`RendererFront`].
pub struct ParalelRenderer<BV>
where
    BV: Volume + Blocked,
{
    volume: BV,
    camera: SendableCamera, // In read mode during the render, write inbetween renders
    render_options: RenderOptions,
    buffer: Arc<Mutex<Vec<u8>>>,
    communication: (Sender<()>, Receiver<RendererMessage>),
}

pub struct SendableCamera(UnsafeCell<PerspectiveCamera>);

impl std::ops::Deref for SendableCamera {
    type Target = UnsafeCell<PerspectiveCamera>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SendableCamera {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

unsafe impl Sync for SendableCamera {}

impl<BV> RenderThread for ParalelRenderer<BV>
where
    BV: Volume + Blocked + 'static,
{
    fn get_shared_buffer(&self) -> Arc<Mutex<Vec<u8>>> {
        self.buffer.clone()
    }

    fn start(self) -> JoinHandle<()> {
        println!("Starting renderer | {}", <BV as Volume>::get_name());
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
    /// Construct new `ParalelRenderer`.
    pub fn new(volume: BV, camera: PerspectiveCamera, render_options: RenderOptions) -> Self {
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
            camera: SendableCamera(UnsafeCell::new(camera)),
            render_options,
            buffer,
            communication,
        }
    }

    /// Spawns 'master thread'.
    /// This thread controls rendering cycle, communicates with user.
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

                    let cam_ref = &self.camera;

                    for id in 0..RENDERER_COUNT {
                        // Create render thread
                        let ren_comms = comms.renderer(id as usize);
                        let handle = s
                            .builder()
                            .name(format!("Ren{id}"))
                            .spawn(move |_| {
                                #[cfg(debug_assertions)]
                                println!("Started renderer {id}");

                                let mut renderer = RenderWorker::new(
                                    id as usize,
                                    cam_ref,
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

                    let sample_step = match msg {
                        RendererMessage::StartRendering {
                            sample_step,
                            camera,
                        } => {
                            if let Some(cam) = camera {
                                let cam_ref = unsafe { self.camera.0.get().as_mut().unwrap() };
                                *cam_ref = cam;
                            }
                            sample_step
                        }
                        RendererMessage::ShutDown => break,
                    };

                    #[cfg(debug_assertions)]
                    println!("Master : start rendering");

                    // Prepare canvas (mainly queues)
                    {
                        let blocks = volume.get_blocks();
                        let empty_blocks_index = volume.get_empty_blocks();
                        let cam_ref = unsafe { self.camera.get().as_ref().unwrap() };

                        canvas.build_queues(
                            cam_ref,
                            blocks,
                            empty_blocks_index,
                            self.render_options,
                        );
                    }

                    #[cfg(debug_assertions)]
                    println!("Master : queues built");

                    // Send go live messages
                    for worker in master_comms.command_sender.iter() {
                        worker.send(ToWorkerMsg::GoLive { sample_step }).unwrap();
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
