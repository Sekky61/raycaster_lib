use crate::common::{Algorithm, BenchOptions, DEFAULT_CAMERA_POSITIONS, HEIGHT, WIDTH};
use criterion::Criterion;
use raycaster_lib::render::RenderOptions;

pub fn render_parallel(c: &mut Criterion) {
    let render_options = RenderOptions {
        resolution: (WIDTH, HEIGHT),
        ray_termination: true,
        empty_index: true,
    };

    let bench_options = BenchOptions::new(
        render_options,
        format!("Render MT | {WIDTH}x{HEIGHT} | no optim"),
        Algorithm::Parallel,
        &DEFAULT_CAMERA_POSITIONS,
    );

    let benchmark = bench_options.get_benchmark();

    benchmark(c);
}
