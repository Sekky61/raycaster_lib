use crate::common::{get_volume, Algorithm, BenchOptions, DEFAULT_CAMERA_POSITIONS, RESOLUTION};
use criterion::Criterion;
use raycaster_lib::{render::RenderOptions, volumetric::LinearVolume};

pub fn render_linear(c: &mut Criterion) {
    let render_options = RenderOptions {
        resolution: RESOLUTION,
        ray_termination: false,
        empty_index: false,
    };

    let volume: LinearVolume = get_volume();

    let bench_options = BenchOptions::new(
        render_options,
        Algorithm::Linear,
        &DEFAULT_CAMERA_POSITIONS,
        volume,
    );

    let benchmark = bench_options.get_benchmark();

    benchmark(c);
}

pub fn render_linear_ert(c: &mut Criterion) {
    let render_options = RenderOptions {
        resolution: RESOLUTION,
        ray_termination: true,
        empty_index: false,
    };

    let volume: LinearVolume = get_volume();

    let bench_options = BenchOptions::new(
        render_options,
        Algorithm::Linear,
        &DEFAULT_CAMERA_POSITIONS,
        volume,
    );

    let benchmark = bench_options.get_benchmark();

    benchmark(c);
}

pub fn render_linear_ei(c: &mut Criterion) {
    let render_options = RenderOptions {
        resolution: RESOLUTION,
        ray_termination: false,
        empty_index: true,
    };

    let volume: LinearVolume = get_volume();

    let bench_options = BenchOptions::new(
        render_options,
        Algorithm::Linear,
        &DEFAULT_CAMERA_POSITIONS,
        volume,
    );

    let benchmark = bench_options.get_benchmark();

    benchmark(c);
}

pub fn render_linear_ert_ei(c: &mut Criterion) {
    let render_options = RenderOptions {
        resolution: RESOLUTION,
        ray_termination: true,
        empty_index: true,
    };

    let volume: LinearVolume = get_volume();

    let bench_options = BenchOptions::new(
        render_options,
        Algorithm::Linear,
        &DEFAULT_CAMERA_POSITIONS,
        volume,
    );

    let benchmark = bench_options.get_benchmark();

    benchmark(c);
}
