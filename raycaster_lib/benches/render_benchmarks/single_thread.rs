use crate::common::{Algorithm, BenchOptions, DEFAULT_CAMERA_POSITIONS, HEIGHT, RESOLUTION, WIDTH};
use criterion::Criterion;
use raycaster_lib::render::RenderOptions;

pub fn render_linear(c: &mut Criterion) {
    let render_options = RenderOptions {
        resolution: RESOLUTION,
        ray_termination: false,
        empty_index: false,
    };

    let bench_options = BenchOptions::new(
        render_options,
        format!("Render ST | linear | {WIDTH}x{HEIGHT} | no optim"),
        Algorithm::Linear,
        &DEFAULT_CAMERA_POSITIONS,
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

    let bench_options = BenchOptions::new(
        render_options,
        format!("Render ST | linear | {WIDTH}x{HEIGHT} | ERT"),
        Algorithm::Linear,
        &DEFAULT_CAMERA_POSITIONS,
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

    let bench_options = BenchOptions::new(
        render_options,
        format!("Render ST | linear | {WIDTH}x{HEIGHT} | EI"),
        Algorithm::Linear,
        &DEFAULT_CAMERA_POSITIONS,
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

    let bench_options = BenchOptions::new(
        render_options,
        format!("Render ST | linear | {WIDTH}x{HEIGHT} | ERT + EI"),
        Algorithm::Linear,
        &DEFAULT_CAMERA_POSITIONS,
    );

    let benchmark = bench_options.get_benchmark();

    benchmark(c);
}
