use crate::common::volume_files::*;
use crate::common::{
    get_volume, Algorithm, BenchOptions, Memory, DEFAULT_CAMERA_POSITIONS, RESOLUTION,
};
use criterion::Criterion;
use raycaster_lib::{render::RenderOptions, volumetric::volumes::LinearVolume};

pub fn render_linear<const VOL_ID: usize>(c: &mut Criterion) {
    let render_options = RenderOptions::builder()
        .resolution(RESOLUTION)
        .early_ray_termination(false)
        .empty_space_skipping(false)
        .build_unchecked();

    let path = get_path(VOL_ID);

    let volume: LinearVolume = get_volume(path);

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

pub fn render_linear_ert<const VOL_ID: usize>(c: &mut Criterion) {
    let render_options = RenderOptions::builder()
        .resolution(RESOLUTION)
        .early_ray_termination(true)
        .empty_space_skipping(false)
        .build_unchecked();

    let path = get_path(VOL_ID);

    let volume: LinearVolume = get_volume(path);

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

pub fn render_linear_ei<const VOL_ID: usize>(c: &mut Criterion) {
    let render_options = RenderOptions::builder()
        .resolution(RESOLUTION)
        .early_ray_termination(false)
        .empty_space_skipping(true)
        .build_unchecked();

    let path = get_path(VOL_ID);

    let volume: LinearVolume = get_volume(path);

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

pub fn render_linear_ert_ei<const VOL_ID: usize>(c: &mut Criterion) {
    let render_options = RenderOptions::builder()
        .resolution(RESOLUTION)
        .early_ray_termination(true)
        .empty_space_skipping(true)
        .build_unchecked();

    let path = get_path(VOL_ID);

    let volume: LinearVolume = get_volume(path);

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
