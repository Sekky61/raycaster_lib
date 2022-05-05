/*
    raycaster_lib
    Author: Michal Majer
    Date: 2022-05-05
*/

use std::marker::PhantomData;

use crate::common::volume_files::*;
use crate::common::{Algorithm, BenchOptions, Memory, RESOLUTION};
use criterion::Criterion;
use raycaster_lib::render::RenderOptions;
use raycaster_lib::volumetric::volumes::LinearVolume;

pub fn render_ramlinear<const VOL_ID: usize>(c: &mut Criterion) {
    let render_options = RenderOptions::builder()
        .resolution(RESOLUTION)
        .early_ray_termination(false)
        .empty_space_skipping(false)
        .build_unchecked();

    let path = get_path(VOL_ID);

    let bench_options = BenchOptions::new(
        render_options,
        Algorithm::Linear,
        &get_cam_pos(VOL_ID),
        path,
        PhantomData::<LinearVolume>,
        Memory::Ram,
        None,
    );

    let benchmark = bench_options.get_benchmark();

    benchmark(c);
}

pub fn render_ramlinear_ert<const VOL_ID: usize>(c: &mut Criterion) {
    let render_options = RenderOptions::builder()
        .resolution(RESOLUTION)
        .early_ray_termination(true)
        .empty_space_skipping(false)
        .build_unchecked();

    let path = get_path(VOL_ID);

    let bench_options = BenchOptions::new(
        render_options,
        Algorithm::Linear,
        &get_cam_pos(VOL_ID),
        path,
        PhantomData::<LinearVolume>,
        Memory::Ram,
        None,
    );

    let benchmark = bench_options.get_benchmark();

    benchmark(c);
}

pub fn render_ramlinear_ei<const VOL_ID: usize>(c: &mut Criterion) {
    let render_options = RenderOptions::builder()
        .resolution(RESOLUTION)
        .early_ray_termination(false)
        .empty_space_skipping(true)
        .build_unchecked();

    let path = get_path(VOL_ID);

    let bench_options = BenchOptions::new(
        render_options,
        Algorithm::Linear,
        &get_cam_pos(VOL_ID),
        path,
        PhantomData::<LinearVolume>,
        Memory::Ram,
        None,
    );

    let benchmark = bench_options.get_benchmark();

    benchmark(c);
}

pub fn render_ramlinear_ert_ei<const VOL_ID: usize>(c: &mut Criterion) {
    let render_options = RenderOptions::builder()
        .resolution(RESOLUTION)
        .early_ray_termination(true)
        .empty_space_skipping(true)
        .build_unchecked();

    let path = get_path(VOL_ID);

    let bench_options = BenchOptions::new(
        render_options,
        Algorithm::Linear,
        &get_cam_pos(VOL_ID),
        path,
        PhantomData::<LinearVolume>,
        Memory::Ram,
        None,
    );

    let benchmark = bench_options.get_benchmark();

    benchmark(c);
}
