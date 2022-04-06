use std::{
    iter::Cycle,
    sync::{Arc, RwLock},
};

pub use criterion::{criterion_group, criterion_main, Criterion};
pub use nalgebra::{point, vector, Point3, Vector3};
pub use raycaster_lib::{
    render::{RenderOptions, Renderer},
    volumetric::{BlockVolume, BuildVolume, LinearVolume, Volume, VolumeMetadata},
    PerspectiveCamera,
};

pub const WIDTH: usize = 700;
pub const HEIGHT: usize = 700;

pub const POSITION: Point3<f32> = point![QUADRANT_DISTANCE, QUADRANT_DISTANCE, QUADRANT_DISTANCE];
pub const DIRECTION: Vector3<f32> = vector![-1.0, -1.0, -1.0];

type CamPos = (Point3<f32>, Vector3<f32>); // todo add to benchoptions

pub const QUADRANT_DISTANCE: f32 = 300.0;

#[rustfmt::skip]
pub const DEFAULT_CAMERA_POSITIONS: [(Point3<f32>, Vector3<f32>); 11] = [
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
];

use raycaster_lib::{
    premade::{
        parse::{from_file, skull_parser},
        transfer_functions::skull_tf,
    },
    render::{ParalelRenderer, RendererFront, RendererMessage, SerialRenderer},
};

pub fn get_volume<V>() -> V
where
    V: Volume + BuildVolume<u8>,
{
    from_file("../volumes/Skull.vol", skull_parser, skull_tf).unwrap()
}

#[derive(Clone)]
pub struct CameraPositions {
    /// Positions with directions
    pub it: Cycle<std::vec::IntoIter<(Point3<f32>, Vector3<f32>)>>,
}

impl CameraPositions {
    pub fn new(positions: Vec<CamPos>) -> Self {
        assert!(!positions.is_empty());
        let it = positions.into_iter().cycle();
        Self { it }
    }

    pub fn from_slice(slice: &[CamPos]) -> Self {
        assert!(!slice.is_empty());
        let positions = Vec::from(slice);
        let it = positions.into_iter().cycle();
        Self { it }
    }
}

pub enum Algorithm {
    Linear,
    Parallel,
}
pub struct BenchOptions {
    pub render_options: RenderOptions,
    pub bench_name: String,
    pub algorithm: Algorithm,
    pub camera_positions: CameraPositions,
}

impl BenchOptions {
    pub fn new(
        render_options: RenderOptions,
        bench_name: String,
        algorithm: Algorithm,
        camera_positions: &[CamPos],
    ) -> Self {
        let camera_positions = CameraPositions::from_slice(camera_positions);
        Self {
            render_options,
            bench_name,
            algorithm,
            camera_positions,
        }
    }

    pub fn get_benchmark(self) -> impl FnOnce(&mut Criterion) {
        // todo struct bench_render_options
        move |c: &mut Criterion| {
            let camera = PerspectiveCamera::new(POSITION, DIRECTION);
            let shared_camera = Arc::new(RwLock::new(camera));

            let BenchOptions {
                render_options,
                bench_name,
                algorithm,
                camera_positions,
            } = self;

            let mut front = RendererFront::new();

            // handles
            let cam = shared_camera.clone();
            let sender = front.get_sender();
            let finish_sender = sender.clone();
            let receiver = front.get_receiver();

            match algorithm {
                Algorithm::Parallel => {
                    let volume = get_volume();
                    let par_ren = ParalelRenderer::new(volume, shared_camera, render_options);
                    front.start_rendering(par_ren);
                }
                Algorithm::Linear => {
                    let volume: LinearVolume = get_volume();
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

                b.iter_batched(
                    move || {
                        // Setup
                        let (pos, dir) = match positions.it.next() {
                            Some(t) => t,
                            None => panic!("End of cyclic iterator, should not happen"),
                        };
                        {
                            // set another cam pos
                            let mut cam_guard = cam.write().unwrap();

                            cam_guard.set_pos(pos);
                            cam_guard.set_direction(dir);
                        }
                    },
                    move |()| {
                        // measured part
                        sender_in.send(RendererMessage::StartRendering).unwrap();
                        receiver.recv().unwrap();
                    },
                    criterion::BatchSize::PerIteration,
                );
            });
            // Cleanup
            finish_sender.send(RendererMessage::ShutDown).unwrap();
            front.finish();
        }
    }
}
