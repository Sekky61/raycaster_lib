use crate::common::{get_volume, Algorithm, BenchOptions, DEFAULT_CAMERA_POSITIONS, RESOLUTION};
use criterion::Criterion;
use raycaster_lib::{render::RenderOptions, volumetric::volumes::*};

pub fn render_parallel_mem(c: &mut Criterion) {
    let render_options = RenderOptions::builder()
        .resolution(RESOLUTION)
        .early_ray_termination(true)
        .empty_space_skipping(true)
        .build_unchecked();

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
    let render_options = RenderOptions::builder()
        .resolution(RESOLUTION)
        .early_ray_termination(true)
        .empty_space_skipping(true)
        .build_unchecked();

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
