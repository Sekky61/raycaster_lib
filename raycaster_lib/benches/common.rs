use arrayvec::ArrayVec;
pub use criterion::{criterion_group, criterion_main, Criterion};
pub use nalgebra::{point, vector, Point3, Vector3};
pub use raycaster_lib::{
    render::{RenderOptions, Renderer},
    volumetric::{BlockVolume, BuildVolume, LinearVolume, Volume, VolumeMetadata},
    PerspectiveCamera,
};

pub const WIDTH: usize = 512;
pub const HEIGHT: usize = 512;

pub const POSITION: Point3<f32> = point![300.0, 300.0, 300.0];
pub const DIRECTION: Vector3<f32> = vector![-1.0, -1.0, -1.0];

use raycaster_lib::premade::{
    parse::{from_file, skull_parser},
    transfer_functions::skull_tf,
};

pub fn get_volume<V>() -> V
where
    V: Volume + BuildVolume<u8>,
{
    from_file("../volumes/Skull.vol", skull_parser, skull_tf).unwrap()
}

#[derive(Clone)]
pub struct CameraPositions<const C: usize> {
    /// Positions with directions
    positions: ArrayVec<(Point3<f32>, Vector3<f32>), C>,
    state: usize,
}

impl<const C: usize> CameraPositions<C> {
    pub fn new(positions: ArrayVec<(Point3<f32>, Vector3<f32>), C>) -> Self {
        assert_ne!(C, 0);
        Self {
            positions,
            state: 0,
        }
    }

    pub fn from_slice(slice: &[(Point3<f32>, Vector3<f32>); C]) -> Self {
        assert_ne!(C, 0);
        let positions = ArrayVec::from(*slice);
        Self {
            positions,
            state: 0,
        }
    }

    pub fn reset(&mut self) {
        self.state = 1;
    }
}

impl<const C: usize> Iterator for CameraPositions<C> {
    type Item = (Point3<f32>, Vector3<f32>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.state < self.positions.len() {
            let r = Some(self.positions[self.state]);
            self.state += 1;
            r
        } else {
            None
        }
    }
}

pub struct BenchOptions {
    pub render_options: RenderOptions,
    pub bench_name: String,
}

impl BenchOptions {
    pub fn new(render_options: RenderOptions, bench_name: String) -> Self {
        Self {
            render_options,
            bench_name,
        }
    }
}
