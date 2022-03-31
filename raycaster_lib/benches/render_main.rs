use criterion::{criterion_group, criterion_main, Criterion};
use render_benchmarks::{
    multi_thread::render_parallel,
    single_thread::{render_linear, render_linear_ei, render_linear_ert, render_linear_ert_ei},
};

mod common;
mod render_benchmarks;

criterion_group! {
    name = benches;
    config = Criterion::default().significance_level(0.1).sample_size(10); // todo more measure_time
    targets = render_linear, render_linear_ert, render_linear_ei, render_linear_ert_ei, render_parallel
}

criterion_main!(benches);
