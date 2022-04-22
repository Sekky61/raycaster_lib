use common::volume_files::*;
use criterion::{criterion_group, criterion_main, Criterion};
use render_benchmarks::{block::*, block_stream::*, linear::*, linear_stream::*, multi_thread::*};

mod common;
mod render_benchmarks;

const SAMPLE_SIZE: usize = 10;

// Single thread

// Linear Volume
criterion_group! {
    name = sequential_linear_skull;
    config = Criterion::default().significance_level(0.1).sample_size(SAMPLE_SIZE);
    targets = render_linear<{SKULL_ID}>, render_linear_ert<{SKULL_ID}>, render_linear_ei<{SKULL_ID}>, render_linear_ert_ei<{SKULL_ID}>
}

// Linear Volume Streamed
criterion_group! {
    name = sequential_streamlinear_skull;
    config = Criterion::default().significance_level(0.1).sample_size(SAMPLE_SIZE);
    targets = render_streamlinear<{SKULL_ID}>, render_streamlinear_ert<{SKULL_ID}>, render_streamlinear_ei<{SKULL_ID}>, render_streamlinear_ert_ei<{SKULL_ID}>
}

// Block Volume
criterion_group! {
    name = sequential_block_skull;
    config = Criterion::default().significance_level(0.1).sample_size(SAMPLE_SIZE);
    targets = render_block<{SKULL_ID}>, render_block_ert<{SKULL_ID}>, render_block_ei<{SKULL_ID}>, render_block_ert_ei<{SKULL_ID}>
}

// Block Volume Streamed
// todo special volume must be used
criterion_group! {
    name = sequential_streamblock_skull;
    config = Criterion::default().significance_level(0.1).sample_size(SAMPLE_SIZE);
    targets = render_streamblock<{SKULL_ID}>, render_streamblock_ert<{SKULL_ID}>, render_streamblock_ei<{SKULL_ID}>, render_streamblock_ert_ei<{SKULL_ID}>
}

// Parallel
// todo special volume must be used
criterion_group! {
    name = parallel;
    config = Criterion::default().significance_level(0.1).sample_size(SAMPLE_SIZE);
    targets = render_parallel_mem<{SKULL_ID}>, render_parallel_stream<{SKULL_ID}>
}

criterion_group! {
    name = huge_volume;
    config = Criterion::default().significance_level(0.1).sample_size(SAMPLE_SIZE);
    targets = render_parallel_stream<{VOL_4K_ID}>
}

// Testing

criterion_group! {
    name = sequential_ei;
    config = Criterion::default().significance_level(0.1).sample_size(SAMPLE_SIZE);
    targets = render_linear_ei<{SKULL_ID}>
}

criterion_group! {
    name = parallel_params;
    config = Criterion::default().significance_level(0.1).sample_size(SAMPLE_SIZE);
    targets = render_parallel_mem<{SKULL_ID}>
}

// Testing mains

//criterion_main!(parallel);
//criterion_main!(sequential_ei);

// Find best parallel worker params

//criterion_main!(parallel_params);

// BP main

criterion_main!(
    // Ram vs Stream
    sequential_linear_skull,
    sequential_streamlinear_skull,
    parallel,
    // Linear vs Block
    sequential_block_skull,
    // Bigger than RAM
    huge_volume // todo 2K performance
);
