use criterion::{criterion_group, criterion_main, Criterion};
use render_benchmarks::{block::*, linear::*, multi_thread::*};

mod common;
mod render_benchmarks;

criterion_group! {
    name = sequential_linear;
    config = Criterion::default().significance_level(0.1).sample_size(10);
    targets = render_linear, render_linear_ert, render_linear_ei, render_linear_ert_ei
}

criterion_group! {
    name = sequential_block;
    config = Criterion::default().significance_level(0.1).sample_size(10);
    targets = render_block, render_block_ert, render_block_ei, render_block_ert_ei
}

criterion_group! {
    name = sequential_ei;
    config = Criterion::default().significance_level(0.1).sample_size(10);
    targets = render_linear_ei
}

criterion_group! {
    name = parallel;
    config = Criterion::default().significance_level(0.1).sample_size(10);
    targets = render_parallel_mem, render_parallel_stream
}

criterion_main!(sequential_linear, sequential_block, parallel);
//criterion_main!(parallel);
//criterion_main!(sequential_ei);
