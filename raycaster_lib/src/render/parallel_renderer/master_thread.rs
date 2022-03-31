use std::{
    sync::{Arc, Mutex, RwLock},
    thread::JoinHandle,
};

use crossbeam::channel::{Receiver, Sender};
use nalgebra::vector;

use crate::{
    common::PixelBox,
    render::{render_front::RenderThread, RenderOptions, RendererMessage},
    volumetric::{BlockVolume, Volume},
    PerspectiveCamera,
};

use super::workers::{CompositorWorker, RenderWorker};
use super::{
    communication::CommsBuilder,
    messages::{RenderTask, SubFrameResult, ToMasterMsg, ToWorkerMsg},
};

pub const PAR_SIDE: usize = 16;

pub struct ParalelRenderer {
    volume: BlockVolume<PAR_SIDE>,
    camera: Arc<RwLock<PerspectiveCamera>>, // In read mode during the render, write inbetween renders
    render_options: RenderOptions,
    buffer: Arc<Mutex<Vec<u8>>>,
    compositor_canvases: Vec<PixelBox>,
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
        volume: BlockVolume<PAR_SIDE>,
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

        let compositor_canvases =
            ParalelRenderer::generate_compositor_areas(render_options.resolution, 4);

        Self {
            volume,
            camera,
            render_options,
            buffer,
            compositor_canvases,
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
                        let handle = s
                            .builder()
                            .name(format!("Ren{id}"))
                            .spawn(move |_| {
                                println!("Started renderer {id}");

                                let renderer = RenderWorker::new(
                                    id, camera_ref, tf, resolution, ren_comms, blocks_ref,
                                );

                                renderer.run();
                            })
                            .unwrap();

                        renderers.push(handle);
                    }

                    for (id, assigned_area) in self.compositor_canvases.iter().enumerate() {
                        // Create compositor thread

                        let comp_comms = comms.compositor(id);
                        let camera_ref = self.camera.clone();
                        let pixels = assigned_area;

                        let handle = s
                            .builder()
                            .name(format!("Com{id}"))
                            .spawn(move |_| {
                                println!("Started compositor {id}");

                                let compositor = CompositorWorker::new(
                                    id,
                                    camera_ref,
                                    pixels.clone(),
                                    resolution,
                                    comp_comms,
                                    blocks_ref,
                                );

                                compositor.run();
                            })
                            .unwrap();

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
            #[cfg(debug_assertions)]
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
    fn generate_compositor_areas(resolution: (usize, usize), n: usize) -> Vec<PixelBox> {
        if n == 4 {
            let pixels_1 = PixelBox::new(0..350, 0..350); // todo adapt to resolution
            let pixels_2 = PixelBox::new(350..700, 0..350);
            let pixels_3 = PixelBox::new(0..350, 350..700);
            let pixels_4 = PixelBox::new(350..700, 350..700);
            return vec![pixels_1, pixels_2, pixels_3, pixels_4];
        } else {
            todo!()
        }
    }

    fn copy_subframe(&self, buffer: &mut [u8], res: SubFrameResult) {
        let SubFrameResult { from_id, data } = res;

        let data = &data[..];

        let resolution = self.render_options.resolution;
        let full_pixelbox = PixelBox::new(0..resolution.0, 0..resolution.1);
        let comp_pixelbox = &self.compositor_canvases[from_id];

        let offset = full_pixelbox.offset_in_unchecked(comp_pixelbox) * 3;
        let width = full_pixelbox.width() * 3;
        let subframe_w = comp_pixelbox.width() * 3;

        let mut ptr = offset;
        let mut ptr_data = 0;
        while (ptr_data + subframe_w) <= data.len() {
            buffer[ptr..ptr + subframe_w].copy_from_slice(&data[ptr_data..ptr_data + subframe_w]);

            ptr += width;
            ptr_data += subframe_w;
        }
    }
}
