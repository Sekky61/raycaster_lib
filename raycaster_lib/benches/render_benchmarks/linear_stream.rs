use crate::common::{
    get_volume, Algorithm, BenchOptions, Memory, DEFAULT_CAMERA_POSITIONS, RESOLUTION,
};
use criterion::Criterion;
use raycaster_lib::{render::RenderOptions, volumetric::volumes::StreamVolume};

pub fn render_streamlinear(c: &mut Criterion) {
    let render_options = RenderOptions::builder()
        .resolution(RESOLUTION)
        .early_ray_termination(false)
        .empty_space_skipping(false)
        .build_unchecked();

    let volume: StreamVolume = get_volume();

    let bench_options = BenchOptions::new(
        render_options,
        Algorithm::Linear,
        &DEFAULT_CAMERA_POSITIONS,
        volume,
        Memory::Ram,
    );

    let benchmark = bench_options.get_benchmark();

    benchmark(c);
}

pub fn render_streamlinear_ert(c: &mut Criterion) {
    let render_options = RenderOptions::builder()
        .resolution(RESOLUTION)
        .early_ray_termination(true)
        .empty_space_skipping(false)
        .build_unchecked();

    let volume: StreamVolume = get_volume();

    let bench_options = BenchOptions::new(
        render_options,
        Algorithm::Linear,
        &DEFAULT_CAMERA_POSITIONS,
        volume,
        Memory::Ram,
    );

    let benchmark = bench_options.get_benchmark();

    benchmark(c);
}

pub fn render_streamlinear_ei(c: &mut Criterion) {
    let render_options = RenderOptions::builder()
        .resolution(RESOLUTION)
        .early_ray_termination(false)
        .empty_space_skipping(true)
        .build_unchecked();

    let volume: StreamVolume = get_volume();

    let bench_options = BenchOptions::new(
        render_options,
        Algorithm::Linear,
        &DEFAULT_CAMERA_POSITIONS,
        volume,
        Memory::Ram,
    );

    let benchmark = bench_options.get_benchmark();

    benchmark(c);
}

pub fn render_streamlinear_ert_ei(c: &mut Criterion) {
    let render_options = RenderOptions::builder()
        .resolution(RESOLUTION)
        .early_ray_termination(true)
        .empty_space_skipping(true)
        .build_unchecked();

    let volume: StreamVolume = get_volume();

    let bench_options = BenchOptions::new(
        render_options,
        Algorithm::Linear,
        &DEFAULT_CAMERA_POSITIONS,
        volume,
        Memory::Ram,
    );

    let benchmark = bench_options.get_benchmark();

    benchmark(c);
}
