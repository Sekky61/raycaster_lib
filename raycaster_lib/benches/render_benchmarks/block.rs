use crate::common::{
    get_volume, Algorithm, BenchOptions, Memory, DEFAULT_CAMERA_POSITIONS, RESOLUTION, SKULL_PATH,
};
use criterion::Criterion;
use raycaster_lib::{render::RenderOptions, volumetric::volumes::BlockVolume};

pub fn render_block(c: &mut Criterion) {
    let render_options = RenderOptions::builder()
        .resolution(RESOLUTION)
        .early_ray_termination(false)
        .empty_space_skipping(false)
        .build_unchecked();

    let volume: BlockVolume = get_volume(SKULL_PATH);

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

pub fn render_block_ert(c: &mut Criterion) {
    let render_options = RenderOptions::builder()
        .resolution(RESOLUTION)
        .early_ray_termination(true)
        .empty_space_skipping(false)
        .build_unchecked();

    let volume: BlockVolume = get_volume(SKULL_PATH);

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

pub fn render_block_ei(c: &mut Criterion) {
    let render_options = RenderOptions::builder()
        .resolution(RESOLUTION)
        .early_ray_termination(false)
        .empty_space_skipping(true)
        .build_unchecked();

    let volume: BlockVolume = get_volume(SKULL_PATH);

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

pub fn render_block_ert_ei(c: &mut Criterion) {
    let render_options = RenderOptions::builder()
        .resolution(RESOLUTION)
        .early_ray_termination(true)
        .empty_space_skipping(true)
        .build_unchecked();

    let volume: BlockVolume = get_volume(SKULL_PATH);

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
