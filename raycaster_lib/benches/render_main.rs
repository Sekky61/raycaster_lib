use criterion::{criterion_group, criterion_main, Criterion};
use render_benchmarks::{block::*, block_stream::*, linear::*, linear_stream::*, multi_thread::*};

mod common;
mod render_benchmarks;

const SAMPLE_SIZE: usize = 10;

// Single thread

// Linear Volume
criterion_group! {
    name = sequential_linear;
    config = Criterion::default().significance_level(0.1).sample_size(SAMPLE_SIZE);
    targets = render_linear, render_linear_ert, render_linear_ei, render_linear_ert_ei
}

// Linear Volume Streamed
criterion_group! {
    name = sequential_streamlinear;
    config = Criterion::default().significance_level(0.1).sample_size(SAMPLE_SIZE);
    targets = render_streamlinear, render_streamlinear_ert, render_streamlinear_ei, render_streamlinear_ert_ei
}

// Block Volume
criterion_group! {
    name = sequential_block;
    config = Criterion::default().significance_level(0.1).sample_size(SAMPLE_SIZE);
    targets = render_block, render_block_ert, render_block_ei, render_block_ert_ei
}

// Block Volume Streamed
// todo special volume must be used
criterion_group! {
    name = sequential_streamblock;
    config = Criterion::default().significance_level(0.1).sample_size(SAMPLE_SIZE);
    targets = render_streamblock, render_streamblock_ert, render_streamblock_ei, render_streamblock_ert_ei
}

// Parallel
// todo special volume must be used
criterion_group! {
    name = parallel;
    config = Criterion::default().significance_level(0.1).sample_size(SAMPLE_SIZE);
    targets = render_parallel_mem, render_parallel_stream
}

// Testing

criterion_group! {
    name = sequential_ei;
    config = Criterion::default().significance_level(0.1).sample_size(SAMPLE_SIZE);
    targets = render_linear_ei
}

criterion_group! {
    name = fast_bench;
    config = Criterion::default().significance_level(0.1).sample_size(SAMPLE_SIZE);
    targets = render_linear_ert_ei, render_parallel_mem
}

criterion_main!(
    sequential_linear,
    sequential_streamlinear,
    sequential_block,
    sequential_streamblock,
    parallel
);

//criterion_main!(parallel);
//criterion_main!(sequential_ei);
//criterion_main!(fast_bench);
