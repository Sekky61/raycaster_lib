use common::volume_files::*;
use criterion::{criterion_group, criterion_main, Criterion};
use render_benchmarks::{
    block_ram::*, block_stream::*, float_block::*, float_linear::*, linear_ram::*,
    linear_stream::*, multi_thread::*,
};

mod common;
mod render_benchmarks;

const SAMPLE_SIZE: usize = 10;

// Single thread

// Float Volume and optimizations
criterion_group! {
    name = sequential_linear_float_small;
    config = Criterion::default().significance_level(0.1).sample_size(SAMPLE_SIZE);
    targets =
        render_linear_float<{SMALL_SHAPES_LIN_ID}>,
        render_linear_float_ert<{SMALL_SHAPES_LIN_ID}>,
        render_linear_float_ei<{SMALL_SHAPES_LIN_ID}>,
        render_linear_float_ert_ei<{SMALL_SHAPES_LIN_ID}>
}

// Linear Volume in RAM and optimizations
criterion_group! {
    name = sequential_linear_ram_small;
    config = Criterion::default().significance_level(0.1).sample_size(SAMPLE_SIZE);
    targets =
    render_ramlinear<{SMALL_SHAPES_LIN_ID}>,
    render_ramlinear_ert<{SMALL_SHAPES_LIN_ID}>,
    render_ramlinear_ei<{SMALL_SHAPES_LIN_ID}>,
    render_ramlinear_ert_ei<{SMALL_SHAPES_LIN_ID}>
}

// Linear Volume Streamed and optimizations
criterion_group! {
    name = sequential_linear_stream_small;
    config = Criterion::default().significance_level(0.1).sample_size(SAMPLE_SIZE);
    targets =
        render_streamlinear<{SMALL_SHAPES_LIN_ID}>,
        render_streamlinear_ert<{SMALL_SHAPES_LIN_ID}>,
        render_streamlinear_ei<{SMALL_SHAPES_LIN_ID}>,
        render_streamlinear_ert_ei<{SMALL_SHAPES_LIN_ID}>
}

// Block Volume Float and optimizations
criterion_group! {
    name = sequential_block_float_small;
    config = Criterion::default().significance_level(0.1).sample_size(SAMPLE_SIZE);
    targets =
        render_block_float<{SMALL_SHAPES_LIN_ID}>,
        render_block_float_ert<{SMALL_SHAPES_LIN_ID}>,
        render_block_float_ei<{SMALL_SHAPES_LIN_ID}>,
        render_block_float_ert_ei<{SMALL_SHAPES_LIN_ID}>
}

// Block Volume stream and optimizations
criterion_group! {
    name = sequential_block_stream_small;
    config = Criterion::default().significance_level(0.1).sample_size(SAMPLE_SIZE);
    targets =
        render_streamblock<{SMALL_SHAPES_LIN_ID}>,
        render_streamblock_ert<{SMALL_SHAPES_LIN_ID}>,
        render_streamblock_ei<{SMALL_SHAPES_LIN_ID}>,
        render_streamblock_ert_ei<{SMALL_SHAPES_LIN_ID}>
}

// Block Volume RAM and optimizations
criterion_group! {
    name = sequential_block_ram_small;
    config = Criterion::default().significance_level(0.1).sample_size(SAMPLE_SIZE);
    targets =
        render_ramblock<{SMALL_SHAPES_LIN_ID}>,
        render_ramblock_ert<{SMALL_SHAPES_LIN_ID}>,
        render_ramblock_ei<{SMALL_SHAPES_LIN_ID}>,
        render_ramblock_ert_ei<{SMALL_SHAPES_LIN_ID}>
}

// 2K section

// Linear Volume in RAM and optimizations
criterion_group! {
    name = sequential_linear_ram_2k;
    config = Criterion::default().significance_level(0.1).sample_size(SAMPLE_SIZE);
    targets =
    render_ramlinear<{MAIN_BLOCK_ID}>,
    render_ramlinear_ert_ei<{MAIN_BLOCK_ID}>
}

// Parallel, 2K volume, full optimisations
criterion_group! {
    name = parallel_2k;
    config = Criterion::default().significance_level(0.1).sample_size(SAMPLE_SIZE);
    targets =
        render_float_parallel<{MAIN_BLOCK_ID}>,
        render_parallel_ram<{MAIN_BLOCK_ID}>,
        render_parallel_stream<{MAIN_BLOCK_ID}>
}

// Streamed huge volume, 4K volume
criterion_group! {
    name = parallel_4k;
    config = Criterion::default().significance_level(0.1).sample_size(SAMPLE_SIZE);
    targets =
        render_parallel_stream<{HUGE_ID}>
}

// Experiment, finding optimal block size
criterion_group! {
    name = block_side;
    config = Criterion::default().significance_level(0.1).sample_size(SAMPLE_SIZE);
    targets = block_side_test
}

// Testing mains

//criterion_main!(parallel);
criterion_main!(block_side);
//criterion_main!(sequential_ei);

// Find best parallel worker params

//criterion_main!(parallel_params);

// BP main
/*
criterion_main!(
    // Ram vs Stream
    sequential_linear_skull,
    sequential_streamlinear_skull,
    parallel,
    // Linear vs Block
    sequential_block_skull,
    // Bigger than RAM
    huge_volume // todo 2K performance
                // todo compare camera angles
);
*/
