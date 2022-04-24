use std::marker::PhantomData;

use crate::common::{volume_files::*, BLOCK_SIDE};
use crate::common::{Algorithm, BenchOptions, Memory, RESOLUTION};
use criterion::Criterion;
use raycaster_lib::{render::RenderOptions, volumetric::volumes::*};

pub fn render_float_parallel<const VOL_ID: usize>(c: &mut Criterion) {
    let render_options = RenderOptions::builder()
        .resolution(RESOLUTION)
        .early_ray_termination(true)
        .empty_space_skipping(true)
        .build_unchecked();

    let path = get_path(VOL_ID);

    let bench_options = BenchOptions::new(
        render_options,
        Algorithm::Parallel,
        &get_cam_pos(VOL_ID),
        path,
        PhantomData::<FloatBlockVolume>,
        Memory::Ram,
        Some(BLOCK_SIDE),
    );

    let benchmark = bench_options.get_benchmark();

    benchmark(c);
}

pub fn render_parallel_ram<const VOL_ID: usize>(c: &mut Criterion) {
    let render_options = RenderOptions::builder()
        .resolution(RESOLUTION)
        .early_ray_termination(true)
        .empty_space_skipping(true)
        .build_unchecked();

    let path = get_path(VOL_ID);

    let bench_options = BenchOptions::new(
        render_options,
        Algorithm::Parallel,
        &get_cam_pos(VOL_ID),
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
        &get_cam_pos(VOL_ID),
        path,
        PhantomData::<BlockVolume>,
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
        &get_cam_pos(SKULL_BLOCK_ID),
        path,
        PhantomData::<FloatBlockVolume>,
        Memory::Ram,
        Some(BLOCK_SIDE),
    );

    let benchmark = bench_options.block_size_bench();

    benchmark(c);
}
