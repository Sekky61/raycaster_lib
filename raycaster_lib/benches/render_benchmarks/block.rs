use std::marker::PhantomData;

use crate::common::volume_files::*;
use crate::common::{Algorithm, BenchOptions, Memory, DEFAULT_CAMERA_POSITIONS, RESOLUTION};
use criterion::Criterion;
use raycaster_lib::{render::RenderOptions, volumetric::volumes::BlockVolume};

pub fn render_block<const VOL_ID: usize>(c: &mut Criterion) {
    let render_options = RenderOptions::builder()
        .resolution(RESOLUTION)
        .early_ray_termination(false)
        .empty_space_skipping(false)
        .build_unchecked();

    let path = get_path(VOL_ID);

    let bench_options = BenchOptions::new(
        render_options,
        Algorithm::Linear,
        &DEFAULT_CAMERA_POSITIONS,
        path,
        PhantomData::<BlockVolume>,
        Memory::Ram,
        Some(16), // fallback
    );

    let benchmark = bench_options.get_benchmark();

    benchmark(c);
}

pub fn render_block_ert<const VOL_ID: usize>(c: &mut Criterion) {
    let render_options = RenderOptions::builder()
        .resolution(RESOLUTION)
        .early_ray_termination(true)
        .empty_space_skipping(false)
        .build_unchecked();

    let path = get_path(VOL_ID);

    let bench_options = BenchOptions::new(
        render_options,
        Algorithm::Linear,
        &DEFAULT_CAMERA_POSITIONS,
        path,
        PhantomData::<BlockVolume>,
        Memory::Ram,
        Some(16), // fallback
    );

    let benchmark = bench_options.get_benchmark();

    benchmark(c);
}

pub fn render_block_ei<const VOL_ID: usize>(c: &mut Criterion) {
    let render_options = RenderOptions::builder()
        .resolution(RESOLUTION)
        .early_ray_termination(false)
        .empty_space_skipping(true)
        .build_unchecked();

    let path = get_path(VOL_ID);

    let bench_options = BenchOptions::new(
        render_options,
        Algorithm::Linear,
        &DEFAULT_CAMERA_POSITIONS,
        path,
        PhantomData::<BlockVolume>,
        Memory::Ram,
        Some(16), // fallback
    );

    let benchmark = bench_options.get_benchmark();

    benchmark(c);
}

pub fn render_block_ert_ei<const VOL_ID: usize>(c: &mut Criterion) {
    let render_options = RenderOptions::builder()
        .resolution(RESOLUTION)
        .early_ray_termination(true)
        .empty_space_skipping(true)
        .build_unchecked();

    let path = get_path(VOL_ID);

    let bench_options = BenchOptions::new(
        render_options,
        Algorithm::Linear,
        &DEFAULT_CAMERA_POSITIONS,
        path,
        PhantomData::<BlockVolume>,
        Memory::Ram,
        Some(16), // fallback
    );

    let benchmark = bench_options.get_benchmark();

    benchmark(c);
}
