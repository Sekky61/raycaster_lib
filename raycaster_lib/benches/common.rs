// Wrong behavior
#![allow(dead_code)]

use std::sync::Arc;

pub use criterion::{criterion_group, criterion_main, Criterion};
pub use nalgebra::{point, vector, Point3, Vector2, Vector3};
use parking_lot::RwLock;
pub use raycaster_lib::{
    premade::{
        parse::{from_file, skull_parser},
        transfer_functions::skull_tf,
    },
    render::{ParalelRenderer, RenderOptions, RendererFront, RendererMessage, SerialRenderer},
    volumetric::{volumes::*, BuildVolume, DataSource, Volume, VolumeMetadata},
    PerspectiveCamera,
};

pub const WIDTH: u16 = 700;
pub const HEIGHT: u16 = 700;

pub const RESOLUTION: Vector2<u16> = vector![WIDTH, HEIGHT];

pub const POSITION: Point3<f32> = point![QUADRANT_DISTANCE, QUADRANT_DISTANCE, QUADRANT_DISTANCE];
pub const DIRECTION: Vector3<f32> = vector![-1.0, -1.0, -1.0];

type CamPos = (Point3<f32>, Vector3<f32>); // todo add to benchoptions

const BLOCK_SIDE: usize = 32;

pub const QUADRANT_DISTANCE: f32 = 300.0;

#[rustfmt::skip]
pub const DEFAULT_CAMERA_POSITIONS_MULTIPLE: [(Point3<f32>, Vector3<f32>); 14] = [ // todo average 14 samples together
    // View volume from each quadrant
    (point![QUADRANT_DISTANCE, QUADRANT_DISTANCE, QUADRANT_DISTANCE], vector![-1.0, -1.0, -1.0]),
    (point![QUADRANT_DISTANCE, QUADRANT_DISTANCE, -QUADRANT_DISTANCE], vector![-1.0, -1.0, 1.0]),
    (point![QUADRANT_DISTANCE, -QUADRANT_DISTANCE, QUADRANT_DISTANCE], vector![-1.0, 1.0, -1.0]),
    (point![QUADRANT_DISTANCE, -QUADRANT_DISTANCE, -QUADRANT_DISTANCE], vector![-1.0, 1.0, 1.0]),
    (point![-QUADRANT_DISTANCE, QUADRANT_DISTANCE, QUADRANT_DISTANCE], vector![1.0, -1.0, -1.0]),
    (point![-QUADRANT_DISTANCE, QUADRANT_DISTANCE, -QUADRANT_DISTANCE], vector![1.0, -1.0, 1.0]),
    (point![-QUADRANT_DISTANCE, -QUADRANT_DISTANCE, QUADRANT_DISTANCE], vector![1.0, 1.0, -1.0]),
    (point![-QUADRANT_DISTANCE, -QUADRANT_DISTANCE, -QUADRANT_DISTANCE], vector![1.0, 1.0, 1.0]),
    // View volume from each axis
    (point![QUADRANT_DISTANCE, 0.0, 0.0], vector![-1.0, 0.0, 0.0]),
    (point![0.0, QUADRANT_DISTANCE, 0.0], vector![0.0, -1.0, 0.0]),
    (point![0.0, 0.0, QUADRANT_DISTANCE], vector![0.0, 0.0, -1.0]),
    (point![-QUADRANT_DISTANCE, 0.0, 0.0], vector![1.0, 0.0, 0.0]),
    (point![0.0, -QUADRANT_DISTANCE, 0.0], vector![0.0, 1.0, 0.0]),
    (point![0.0, 0.0, -QUADRANT_DISTANCE], vector![0.0, 0.0, 1.0]),
];

pub const DEFAULT_CAMERA_POSITIONS: [(Point3<f32>, Vector3<f32>); 1] = [(
    point![QUADRANT_DISTANCE, QUADRANT_DISTANCE, QUADRANT_DISTANCE],
    vector![-1.0, -1.0, -1.0],
)];

pub fn get_volume<V>() -> V
where
    V: Volume + BuildVolume<u8>,
{
    let parser_add_block_side = move |src: DataSource<u8>| {
        let mut res = skull_parser(src);
        match &mut res {
            Ok(ref mut m) => {
                if m.block_side.is_none() {
                    m.block_side = Some(BLOCK_SIDE);
                }
            }
            Err(_) => (),
        }
        res
    };
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")); // should be library root (!not workspace dir!)
    path.push("../volumes/Skull.vol");
    from_file(path, parser_add_block_side, skull_tf).unwrap()
}

pub enum Algorithm {
    Linear,
    Parallel,
}

#[derive(PartialEq, Eq)]
pub enum Memory {
    Stream,
    Ram,
}

pub struct BenchOptions<V> {
    pub render_options: RenderOptions,
    pub algorithm: Algorithm,
    pub camera_positions: Vec<CamPos>,
    volume: V, // Ignored if rendering is parallel
    pub memory: Memory,
}

impl<V> BenchOptions<V>
where
    V: Volume + BuildVolume<u8> + 'static,
{
    pub fn new(
        render_options: RenderOptions,
        algorithm: Algorithm,
        camera_positions: &[CamPos],
        volume: V,
        memory: Memory,
    ) -> Self {
        let camera_positions = Vec::from(camera_positions);
        Self {
            render_options,
            algorithm,
            camera_positions,
            volume,
            memory,
        }
    }

    pub fn get_benchmark(self) -> impl FnOnce(&mut Criterion) {
        // todo struct bench_render_options
        move |c: &mut Criterion| {
            let camera = PerspectiveCamera::new(POSITION, DIRECTION);
            let shared_camera = Arc::new(RwLock::new(camera));

            let bench_name = self.generate_bench_name();

            let BenchOptions {
                render_options,
                algorithm,
                camera_positions,
                volume,
                memory,
            } = self;

            let mut front = RendererFront::new();

            // handles
            let cam = shared_camera.clone();
            let sender = front.get_sender();
            let finish_sender = sender.clone();
            let receiver = front.get_receiver();

            match algorithm {
                Algorithm::Parallel => {
                    if memory == Memory::Stream {
                        let volume: StreamBlockVolume = get_volume();
                        let par_ren = ParalelRenderer::new(volume, shared_camera, render_options);
                        front.start_rendering(par_ren);
                    } else {
                        let volume: BlockVolume = get_volume();
                        let par_ren = ParalelRenderer::new(volume, shared_camera, render_options);
                        front.start_rendering(par_ren);
                    };
                }
                Algorithm::Linear => {
                    let serial_r = SerialRenderer::new(volume, shared_camera, render_options);
                    front.start_rendering(serial_r);
                }
            }

            // Camera positions
            let mut positions = camera_positions;

            c.bench_function(&bench_name, move |b| {
                let sender_in = sender.clone();
                let receiver = receiver.clone();
                let cam = cam.clone();
                let positions = &mut positions;

                b.iter(move || {
                    let mut pos_iter = positions.iter();
                    // Setup
                    for (pos, dir) in pos_iter.by_ref() {
                        {
                            // set another cam pos
                            let mut cam_guard = cam.write();
                            cam_guard.set_pos(*pos);
                            cam_guard.set_direction(*dir);
                        }

                        // Render
                        sender_in.send(RendererMessage::StartRendering).unwrap();
                        receiver.recv().unwrap();
                    }
                });
            });
            // Cleanup
            finish_sender.send(RendererMessage::ShutDown).unwrap();
            front.finish();
        }
    }

    fn generate_bench_name(&self) -> String {
        let st_mt = match self.algorithm {
            Algorithm::Linear => "ST",
            Algorithm::Parallel => "MT",
        };

        let volume_type = self.volume.get_name();

        let memory = match self.memory {
            Memory::Stream => "Stream",
            Memory::Ram => "Ram",
        };

        let optim = match self.render_options {
            RenderOptions {
                early_ray_termination: false,
                empty_space_skipping: false,
                ..
            } => "no_optim",
            RenderOptions {
                early_ray_termination: true,
                empty_space_skipping: false,
                ..
            } => "ERT",
            RenderOptions {
                early_ray_termination: false,
                empty_space_skipping: true,
                ..
            } => "EI",
            RenderOptions {
                early_ray_termination: true,
                empty_space_skipping: true,
                ..
            } => "ERT + EI",
        };

        let w = self.render_options.resolution.x;
        let h = self.render_options.resolution.y;

        format!("Render {st_mt} | {volume_type} | {w}x{h} | {optim} | {memory}")
    }
}
