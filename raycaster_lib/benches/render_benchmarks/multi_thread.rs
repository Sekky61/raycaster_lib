use crate::common::{get_volume, Algorithm, BenchOptions, DEFAULT_CAMERA_POSITIONS, RESOLUTION};
use criterion::Criterion;
use raycaster_lib::{
    render::RenderOptions,
    volumetric::{BlockVolume, StreamBlockVolume},
};

pub fn render_parallel_mem(c: &mut Criterion) {
    let render_options = RenderOptions {
        resolution: RESOLUTION,
        ray_termination: true,
        empty_index: true,
    };

    let volume: BlockVolume = get_volume();

    let bench_options = BenchOptions::new(
        render_options,
        Algorithm::Parallel,
        &DEFAULT_CAMERA_POSITIONS,
        volume,
    );

    let benchmark = bench_options.get_benchmark();

    benchmark(c);
}

pub fn render_parallel_stream(c: &mut Criterion) {
    let render_options = RenderOptions {
        resolution: RESOLUTION,
        ray_termination: true,
        empty_index: true,
    };

    let volume: StreamBlockVolume = get_volume();

    let bench_options = BenchOptions::new(
        render_options,
        Algorithm::Parallel,
        &DEFAULT_CAMERA_POSITIONS,
        volume,
    );

    let benchmark = bench_options.get_benchmark();

    benchmark(c);
}
