use std::{
    sync::{Arc, Mutex, RwLock},
    thread::JoinHandle,
};

use crossbeam::channel::{Receiver, Sender};
use nalgebra::vector;

use crate::{
    camera::{Camera, PerspectiveCamera},
    common::{PixelBox, ViewportBox},
    render::{render_front::RenderThread, RenderOptions, RendererMessage},
    volumetric::{BlockVolume, Volume},
};

use super::workers::{CompositorWorker, RenderWorker};
use super::{
    communication::CommsBuilder,
    messages::{RenderTask, SubFrameResult, ToMasterMsg, ToWorkerMsg},
};

pub struct ParalelRenderer {
    volume: BlockVolume,
    camera: Arc<RwLock<PerspectiveCamera>>, // In read mode during the render, write inbetween renders
    render_options: RenderOptions,
    buffer: Arc<Mutex<Vec<u8>>>,
    communication: (Sender<()>, Receiver<RendererMessage>),
}

impl RenderThread for ParalelRenderer {
    fn get_shared_buffer(&self) -> Arc<Mutex<Vec<u8>>> {
        self.buffer.clone()
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

impl ParalelRenderer {
    pub fn new(
        volume: BlockVolume,
        camera: Arc<RwLock<PerspectiveCamera>>,
        render_options: RenderOptions,
    ) -> Self {
        let elements = render_options.resolution.0 * render_options.resolution.1;
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

                // inlined function because borrow checker defeated me (scope cannot leave closure)
                let (master_comms, ren_handles, comp_handles) = {
                    // R = 4 C = 4 RC = 8
                    let comms = CommsBuilder::<4, 4, 8>::new();

                    let mut renderers = Vec::with_capacity(4);
                    let mut compositors = Vec::with_capacity(4);

                    let resolution = vector![
                        self.render_options.resolution.0,
                        self.render_options.resolution.1
                    ];

                    let blocks_ref = &volume.data[..];

                    let tf = volume.get_tf();

                    for id in 0..4 {
                        // Create render thread
                        let ren_comms = comms.renderer(id);
                        let camera_ref = self.camera.clone();
                        let handle = s.spawn(move |_| {
                            println!("Started renderer {id}");

                            let renderer = RenderWorker::new(
                                id, camera_ref, tf, resolution, ren_comms, blocks_ref,
                            );

                            renderer.run();
                        });

                        renderers.push(handle);
                    }

                    let compositor_areas = self.generate_compositor_areas(4);

                    for (id, assigned_area) in compositor_areas.into_iter().enumerate() {
                        // Create compositor thread

                        let comp_comms = comms.compositor(id);
                        let camera_ref = self.camera.clone();
                        let pixels = assigned_area;

                        let handle = s.spawn(move |_| {
                            println!("Started compositor {id}");

                            let compositor = CompositorWorker::new(
                                id, camera_ref, pixels, resolution, comp_comms, blocks_ref,
                            );

                            compositor.run();
                        });

                        compositors.push(handle);
                    }

                    (comms.master(), renderers, compositors)
                };

                // Master loop
                loop {
                    // Gather input
                    #[cfg(debug_assertions)]
                    println!("Master : waiting for input");
                    let msg = self.communication.1.recv().unwrap();
                    match msg {
                        RendererMessage::StartRendering => (),
                        RendererMessage::ShutDown => break,
                    }

                    #[cfg(debug_assertions)]
                    println!("Master : start rendering");

                    // Send go live messages
                    for worker in master_comms.command_sender.iter() {
                        worker.send(ToWorkerMsg::GoLive).unwrap();
                    }

                    // Lock buffer
                    let mut buffer = self.buffer.lock().unwrap();

                    // Render
                    let task_sender = master_comms.task_sender.clone();
                    let result_receiver = master_comms.result_receiver.clone();
                    self.render(task_sender, result_receiver, &mut buffer[..]);

                    // Send idle messages
                    // Renderers need to let go of camera
                    for worker in master_comms.command_sender.iter() {
                        worker.send(ToWorkerMsg::GoIdle).unwrap();
                    }

                    // Send result
                    self.communication.0.send(()).unwrap();

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

    fn render(
        &self,
        task_sender: Sender<RenderTask>,
        result_receiver: Receiver<ToMasterMsg>,
        buffer: &mut [u8],
    ) {
        // Read lock until end of function
        let camera = self
            .camera
            .read()
            .expect("Cannot acquire read lock to camera");

        // get subvolume distances
        let mut block_order = Vec::with_capacity(self.volume.data.len());
        for (id, block) in self.volume.data.iter().enumerate() {
            let distance = camera.box_distance(&block.bound_box);
            block_order.push((id, distance));
        }
        block_order.sort_unstable_by(|i1, i2| i1.1.partial_cmp(&i2.1).unwrap());

        // Send rendering tasks
        //
        // Sent in order of camera distance (asc)
        // for Load balancing
        for order in 0..self.volume.data.len() {
            // Find out if block is empty, in which case dont send it
            let block = &self.volume.data[block_order[order].0]; // TODO

            // Send task
            println!("Master: send task {order}");
            task_sender.send(RenderTask::new(order)).unwrap();
        }

        #[cfg(debug_assertions)]
        println!("Master : sent all tasks");

        // Get subcanvases from compositors and save them to buffer
        for i in 1..=4 {
            let msg = result_receiver.recv().unwrap();
            match msg {
                ToMasterMsg::Subframe(res) => {
                    self.copy_subframe(buffer, res);

                    #[cfg(debug_assertions)]
                    println!("Master : copied subframe ({}/{})", i, 4);
                }
            }
        }
    }

    // Segment viewport into n subframes
    // Calc pixel sizes and offsets
    // todo return only PixelBox
    fn generate_compositor_areas(&self, n: usize) -> Vec<PixelBox> {
        if n == 4 {
            let box1 = ViewportBox::from_points(vector![0.0, 0.0], vector![0.5, 0.5]);
            let box2 = ViewportBox::from_points(vector![0.5, 0.0], vector![1.0, 0.5]);
            let box3 = ViewportBox::from_points(vector![0.0, 0.5], vector![0.5, 1.0]);
            let box4 = ViewportBox::from_points(vector![0.5, 0.5], vector![1.0, 1.0]);
            let res_vec = vector![
                self.render_options.resolution.0,
                self.render_options.resolution.1
            ];
            let box1_pix = box1.get_pixel_range(res_vec);
            let box2_pix = box2.get_pixel_range(res_vec);
            let box3_pix = box3.get_pixel_range(res_vec);
            let box4_pix = box4.get_pixel_range(res_vec);
            return vec![box1_pix, box2_pix, box3_pix, box4_pix];
        } else {
            todo!()
        }
    }

    fn copy_subframe(&self, buffer: &mut [u8], res: SubFrameResult) {
        let SubFrameResult {
            data,
            offset,
            width,
        } = res;

        let data = &data[..];

        let mut ptr = offset;
        let mut ptr_data = 0;
        while (ptr_data + width) <= data.len() {
            buffer[ptr..ptr + width].copy_from_slice(&data[ptr_data..ptr_data + width]);

            ptr += self.render_options.resolution.0;
            ptr_data += width;
        }
    }
}
