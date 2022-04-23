use std::marker::PhantomData;

use crate::common::{volume_files::*, BLOCK_SIDE};
use crate::common::{Algorithm, BenchOptions, Memory, DEFAULT_CAMERA_POSITIONS, RESOLUTION};
use criterion::Criterion;
use raycaster_lib::{render::RenderOptions, volumetric::volumes::*};

pub fn render_parallel_mem<const VOL_ID: usize>(c: &mut Criterion) {
    let render_options = RenderOptions::builder()
        .resolution(RESOLUTION)
        .early_ray_termination(true)
        .empty_space_skipping(true)
        .build_unchecked();

    let path = get_path(VOL_ID);

    let bench_options = BenchOptions::new(
        render_options,
        Algorithm::Parallel,
        &DEFAULT_CAMERA_POSITIONS,
        path,
        PhantomData::<BlockVolume>,
        Memory::Ram,
        Some(BLOCK_SIDE),
    );

    let benchmark = bench_options.get_benchmark();

    benchmark(c);
}

pub fn render_parallel_stream<const VOL_ID: usize>(c: &mut Criterion) {
    let render_options = RenderOptions::builder()
        .resolution(RESOLUTION)
        .early_ray_termination(true)
        .empty_space_skipping(true)
        .build_unchecked();

    let path = get_path(VOL_ID);

    let bench_options = BenchOptions::new(
        render_options,
        Algorithm::Parallel,
        &DEFAULT_CAMERA_POSITIONS,
        path,
        PhantomData::<StreamBlockVolume>,
        Memory::Stream,
        Some(BLOCK_SIDE),
    );

    let benchmark = bench_options.get_benchmark();

    benchmark(c);
}

pub fn block_side_test(c: &mut Criterion) {
    let render_options = RenderOptions::builder()
        .resolution(RESOLUTION)
        .early_ray_termination(true)
        .empty_space_skipping(true)
        .build_unchecked();

    let path = get_path(SKULL_BLOCK_ID);

    let bench_options = BenchOptions::new(
        render_options,
        Algorithm::Parallel,
        &DEFAULT_CAMERA_POSITIONS,
        path,
        PhantomData::<BlockVolume>,
        Memory::Ram,
        Some(BLOCK_SIDE),
    );

    let benchmark = bench_options.block_size_bench();

    benchmark(c);
}
