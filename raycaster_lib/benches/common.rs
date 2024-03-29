/*
    raycaster_lib
    Author: Michal Majer
    Date: 2022-05-05
*/

// Wrong behavior
#![allow(dead_code)]

use std::{
    marker::PhantomData,
    path::{Path, PathBuf},
};

use criterion::BenchmarkId;
pub use criterion::{criterion_group, criterion_main, Criterion};
pub use nalgebra::{point, vector, Point3, Vector2, Vector3};
use raycaster_lib::{
    premade::{parse::generator_parser, transfer_functions::shapes_tf},
    volumetric::{MemoryType, StorageShape},
};
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

pub const RAY_STEP: f32 = 0.2;

pub const RESOLUTION: Vector2<u16> = vector![WIDTH, HEIGHT];

pub const POSITION: Point3<f32> = point![QUADRANT_DISTANCE, QUADRANT_DISTANCE, QUADRANT_DISTANCE];
pub const DIRECTION: Vector3<f32> = vector![-1.0, -1.0, -1.0];

/// Position and direction of camera.
type CamPos = (Point3<f32>, Vector3<f32>);

pub mod volume_files {

    use super::*;

    pub const SKULL_ID: usize = 0;
    pub const SKULL_PATH: &str = "../volumes/Skull.vol";
    pub const SKULL_CAM: [CamPos; 1] = camera_pos_single(300.0);

    pub const SKULL_BLOCK_ID: usize = 1;
    pub const SKULL_BLOCK_PATH: &str = "../volumes/Skull_block.vol";
    pub const SKULL_BLOCK_CAM: [CamPos; 1] = camera_pos_single(300.0);

    pub const SMALL_SHAPES_LIN_ID: usize = 2;
    pub const SMALL_SHAPES_LIN_PATH: &str = "../volumes/800shapes_lin.vol";
    pub const SMALL_SHAPES_LIN_CAM: [CamPos; 1] = camera_pos_single(1100.0);

    pub const SMALL_SHAPES_BLOCK_ID: usize = 3;
    pub const SMALL_SHAPES_BLOCK_PATH: &str = "../volumes/800shapes_block16.vol";
    pub const SMALL_SHAPES_BLOCK_CAM: [CamPos; 1] = camera_pos_single(1100.0);

    pub const MAIN_LIN_ID: usize = 4;
    pub const MAIN_LIN_PATH: &str = "../volumes/2kshapes_lin.vol";
    pub const MAIN_LIN_CAM: [CamPos; 1] = camera_pos_single(2500.0);

    pub const MAIN_BLOCK_ID: usize = 5;
    pub const MAIN_BLOCK_PATH: &str = "../volumes/2kshapes_block16.vol";
    pub const MAIN_BLOCK_CAM: [CamPos; 1] = camera_pos_single(2500.0);

    pub const HUGE_ID: usize = 6;
    pub const HUGE_PATH: &str = "../volumes/4kshapes_block16.vol";
    pub const HUGE_CAM: [CamPos; 1] = camera_pos_single(4600.0); // todo

    pub fn get_path(vol_id: usize) -> &'static str {
        match vol_id {
            SKULL_ID => SKULL_PATH,
            SKULL_BLOCK_ID => SKULL_BLOCK_PATH,
            SMALL_SHAPES_LIN_ID => SMALL_SHAPES_LIN_PATH,
            SMALL_SHAPES_BLOCK_ID => SMALL_SHAPES_BLOCK_PATH,
            MAIN_LIN_ID => MAIN_LIN_PATH,
            MAIN_BLOCK_ID => MAIN_BLOCK_PATH,
            HUGE_ID => HUGE_PATH,
            _ => panic!("Unknown volume ID ({vol_id})"),
        }
    }

    pub fn get_cam_pos(vol_id: usize) -> [CamPos; 1] {
        match vol_id {
            SKULL_ID => SKULL_CAM,
            SKULL_BLOCK_ID => SKULL_BLOCK_CAM,
            SMALL_SHAPES_LIN_ID => SMALL_SHAPES_LIN_CAM,
            SMALL_SHAPES_BLOCK_ID => SMALL_SHAPES_BLOCK_CAM,
            MAIN_LIN_ID => MAIN_LIN_CAM,
            MAIN_BLOCK_ID => MAIN_BLOCK_CAM,
            HUGE_ID => HUGE_CAM,
            _ => panic!("Unknown volume ID ({vol_id})"),
        }
    }
}

pub const BLOCK_SIDE: u8 = 16;

pub const QUADRANT_DISTANCE: f32 = 300.0;

#[rustfmt::skip]
pub const CAMERA_POSITIONS_MULTIPLE: [CamPos; 14] = [
    // View volume from each quadrant
    (point![ QUADRANT_DISTANCE,  QUADRANT_DISTANCE,  QUADRANT_DISTANCE], vector![-1.0, -1.0, -1.0]),
    (point![ QUADRANT_DISTANCE,  QUADRANT_DISTANCE, -QUADRANT_DISTANCE], vector![-1.0, -1.0,  1.0]),
    (point![ QUADRANT_DISTANCE, -QUADRANT_DISTANCE,  QUADRANT_DISTANCE], vector![-1.0,  1.0, -1.0]),
    (point![ QUADRANT_DISTANCE, -QUADRANT_DISTANCE, -QUADRANT_DISTANCE], vector![-1.0,  1.0,  1.0]),
    (point![-QUADRANT_DISTANCE,  QUADRANT_DISTANCE,  QUADRANT_DISTANCE], vector![ 1.0, -1.0, -1.0]),
    (point![-QUADRANT_DISTANCE,  QUADRANT_DISTANCE, -QUADRANT_DISTANCE], vector![ 1.0, -1.0,  1.0]),
    (point![-QUADRANT_DISTANCE, -QUADRANT_DISTANCE,  QUADRANT_DISTANCE], vector![ 1.0,  1.0, -1.0]),
    (point![-QUADRANT_DISTANCE, -QUADRANT_DISTANCE, -QUADRANT_DISTANCE], vector![ 1.0,  1.0,  1.0]),
    // View volume from each axis
    (point![QUADRANT_DISTANCE, 0.0, 0.0], vector![-1.0, 0.0, 0.0]),
    (point![0.0, QUADRANT_DISTANCE, 0.0], vector![0.0, -1.0, 0.0]),
    (point![0.0, 0.0, QUADRANT_DISTANCE], vector![0.0, 0.0, -1.0]),
    (point![-QUADRANT_DISTANCE, 0.0, 0.0], vector![1.0, 0.0, 0.0]),
    (point![0.0, -QUADRANT_DISTANCE, 0.0], vector![0.0, 1.0, 0.0]),
    (point![0.0, 0.0, -QUADRANT_DISTANCE], vector![0.0, 0.0, 1.0]),
];

#[rustfmt::skip]
pub const fn camera_pos_all(distance: f32, neg_distance: f32) -> [CamPos; 14] {
    [
    // View volume from each quadrant
    (point![ distance,  distance,  distance], vector![-1.0, -1.0, -1.0]),
    (point![ distance,  distance, neg_distance], vector![-1.0, -1.0,  1.0]),
    (point![ distance, neg_distance,  distance], vector![-1.0,  1.0, -1.0]),
    (point![ distance, neg_distance, neg_distance], vector![-1.0,  1.0,  1.0]),
    (point![neg_distance,  distance,  distance], vector![ 1.0, -1.0, -1.0]),
    (point![neg_distance,  distance, neg_distance], vector![ 1.0, -1.0,  1.0]),
    (point![neg_distance, neg_distance,  distance], vector![ 1.0,  1.0, -1.0]),
    (point![neg_distance, neg_distance, neg_distance], vector![ 1.0,  1.0,  1.0]),
    // View volume from each axis
    (point![distance, 0.0, 0.0], vector![-1.0, 0.0, 0.0]),
    (point![0.0, distance, 0.0], vector![0.0, -1.0, 0.0]),
    (point![0.0, 0.0, distance], vector![0.0, 0.0, -1.0]),
    (point![neg_distance, 0.0, 0.0], vector![1.0, 0.0, 0.0]),
    (point![0.0, neg_distance, 0.0], vector![0.0, 1.0, 0.0]),
    (point![0.0, 0.0, neg_distance], vector![0.0, 0.0, 1.0]),
    ]
}

pub const fn camera_pos_single(distance: f32) -> [CamPos; 1] {
    [(
        point![distance, distance, distance],
        vector![-1.0, -1.0, -1.0],
    )]
}

pub const CAMERA_POSITION_SINGLE: [CamPos; 1] = [(
    point![QUADRANT_DISTANCE, QUADRANT_DISTANCE, QUADRANT_DISTANCE],
    vector![-1.0, -1.0, -1.0],
)];

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
    volume: PhantomData<V>, // Ignored if rendering is parallel
    pub vol_path: PathBuf,
    /// For naming test and parallel renders
    pub memory: Memory,
    pub block_side: Option<u8>,
}

impl<V> BenchOptions<V>
where
    V: Volume + BuildVolume<u8> + 'static,
{
    pub fn new(
        render_options: RenderOptions,
        algorithm: Algorithm,
        camera_positions: &[CamPos],
        vol_path: &str,
        volume: PhantomData<V>,
        memory: Memory,
        block_side: Option<u8>,
    ) -> Self {
        let camera_positions = Vec::from(camera_positions);
        Self {
            render_options,
            algorithm,
            camera_positions,
            volume,
            memory,
            block_side,
            vol_path: vol_path.into(),
        }
    }

    /// Load and construct volume.
    /// Returned volume type is not linked to type specified in benchoptions.
    ///
    /// # Params
    /// * `path` - Path to volume file. Relative to `Cargo.toml` of the library.
    pub fn get_volume<V2, P>(&self, path: P) -> V2
    where
        V2: Volume + BuildVolume<u8>,
        P: AsRef<Path>,
    {
        let parser_add_block_side = move |src: DataSource<u8>| {
            let mut res = generator_parser(src);
            match &mut res {
                Ok(ref mut m) => {
                    if let Some(block_side) = self.block_side {
                        m.desired_data_shape = Some(StorageShape::Z(block_side));
                    }
                    match self.memory {
                        Memory::Stream => m.set_memory_type(MemoryType::Stream),
                        Memory::Ram => m.set_memory_type(MemoryType::Ram),
                    };
                    println!(
                        "Shape of {:?}, memory {:?}",
                        m.desired_data_shape, m.memory_type
                    );
                }
                Err(_) => (),
            }
            res
        };
        let mut full_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")); // should be library root (!not workspace dir!)
        full_path.push(path);
        from_file(full_path, parser_add_block_side, shapes_tf).unwrap()
    }

    pub fn get_benchmark(self) -> impl FnOnce(&mut Criterion) {
        move |c: &mut Criterion| {
            let camera = PerspectiveCamera::new(POSITION, DIRECTION);

            let bench_name = self.generate_bench_name();

            let mut front = RendererFront::new();

            // handles
            let sender = front.get_sender();
            let finish_sender = sender.clone();
            let receiver = front.get_receiver();

            match self.algorithm {
                Algorithm::Parallel => {
                    if self.memory == Memory::Stream {
                        let mut volume: BlockVolume = self.get_volume(&self.vol_path);

                        if self.render_options.empty_space_skipping {
                            volume.build_empty_index();
                        }

                        let par_ren =
                            ParalelRenderer::new(volume, camera.clone(), self.render_options);
                        front.start_rendering(par_ren);
                    } else {
                        let mut volume: FloatBlockVolume = self.get_volume(&self.vol_path);

                        if self.render_options.empty_space_skipping {
                            volume.build_empty_index();
                        }

                        let par_ren =
                            ParalelRenderer::new(volume, camera.clone(), self.render_options);
                        front.start_rendering(par_ren);
                    };
                }
                Algorithm::Linear => {
                    let mut volume: V = self.get_volume(&self.vol_path);

                    if self.render_options.empty_space_skipping {
                        volume.build_empty_index();
                    }

                    let serial_r = SerialRenderer::new(volume, camera.clone(), self.render_options);
                    front.start_rendering(serial_r);
                }
            }

            // Camera positions
            let mut positions = self.camera_positions;

            c.bench_function(&bench_name, move |b| {
                let mut camera = camera.clone();
                let sender_in = sender.clone();
                let receiver = receiver.clone();
                let positions = &mut positions;

                b.iter(move || {
                    let mut pos_iter = positions.iter();
                    // Setup
                    for (pos, dir) in pos_iter.by_ref() {
                        camera.set_pos(*pos);
                        camera.set_direction(*dir);

                        // Render
                        sender_in
                            .send(RendererMessage::StartRendering {
                                sample_step: RAY_STEP,
                                camera: Some(camera.clone()),
                            })
                            .unwrap();
                        receiver.recv().unwrap();
                    }
                });
            });
            // Cleanup
            finish_sender.send(RendererMessage::ShutDown).unwrap();
            front.finish();
        }
    }

    pub fn block_size_bench(mut self) -> impl FnOnce(&mut Criterion) {
        move |c: &mut Criterion| {
            let camera = PerspectiveCamera::new(POSITION, DIRECTION);

            let mut front = RendererFront::new();

            let mut group = c.benchmark_group("block_side_par");
            let cam = camera.clone();
            for size in (2_u8..=128).step_by(2) {
                // handles
                let sender = front.get_sender();

                // set block side
                self.block_side = Some(size);

                // start render
                match self.algorithm {
                    Algorithm::Parallel => {
                        if self.memory == Memory::Stream {
                            let volume: BlockVolume = self.get_volume(&self.vol_path);
                            let par_ren =
                                ParalelRenderer::new(volume, cam.clone(), self.render_options);
                            front.start_rendering(par_ren);
                        } else {
                            let volume: FloatBlockVolume = self.get_volume(&self.vol_path);
                            let par_ren =
                                ParalelRenderer::new(volume, cam.clone(), self.render_options);
                            front.start_rendering(par_ren);
                        };
                    }
                    Algorithm::Linear => panic!("bad 45"),
                }

                group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
                    let mut camera = camera.clone();
                    // Camera positions
                    let positions = self.camera_positions.clone();
                    let sender_in = sender.clone();
                    let receiver = front.get_receiver();
                    b.iter(move || {
                        let mut pos_iter = positions.iter();
                        // Setup
                        for (pos, dir) in pos_iter.by_ref() {
                            camera.set_pos(*pos);
                            camera.set_direction(*dir);

                            // Render
                            sender_in
                                .send(RendererMessage::StartRendering {
                                    sample_step: RAY_STEP,
                                    camera: Some(camera.clone()),
                                })
                                .unwrap();
                            receiver.recv().unwrap();
                        }
                    });
                });
            }
            group.finish();
        }
    }

    fn generate_bench_name(&self) -> String {
        let st_mt = match self.algorithm {
            Algorithm::Linear => "ST",
            Algorithm::Parallel => "MT",
        };

        let volume_type = <V as Volume>::get_name();

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
            } => "ERT+EI",
        };

        let w = self.render_options.resolution.x;
        let h = self.render_options.resolution.y;

        let path = &self.vol_path.file_name().unwrap();

        format!("{st_mt}_{volume_type}_{w}x{h}_{optim}_{memory}_{path:?}")
    }
}
