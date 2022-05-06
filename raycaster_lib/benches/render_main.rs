/*
    raycaster_lib
    Author: Michal Majer
    Date: 2022-05-05
*/

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
        render_streamblock<{SMALL_SHAPES_BLOCK_ID}>,
        render_streamblock_ert<{SMALL_SHAPES_BLOCK_ID}>,
        render_streamblock_ei<{SMALL_SHAPES_BLOCK_ID}>,
        render_streamblock_ert_ei<{SMALL_SHAPES_BLOCK_ID}>
}

// Block Volume RAM and optimizations
criterion_group! {
    name = sequential_block_ram_small;
    config = Criterion::default().significance_level(0.1).sample_size(SAMPLE_SIZE);
    targets =
        render_ramblock<{SMALL_SHAPES_BLOCK_ID}>,
        render_ramblock_ert<{SMALL_SHAPES_BLOCK_ID}>,
        render_ramblock_ei<{SMALL_SHAPES_BLOCK_ID}>,
        render_ramblock_ert_ei<{SMALL_SHAPES_BLOCK_ID}>
}

// Camera angles

// Block Volume RAM and optimizations
criterion_group! {
    name = camera_angles_parallel;
    config = Criterion::default().significance_level(0.1).sample_size(SAMPLE_SIZE);
    targets =
        render_parallel_stream_cameras
}

// 2K section

// Linear Volume in RAM and optimizations
criterion_group! {
    name = sequential_linear_ram_2k;
    config = Criterion::default().significance_level(0.1).sample_size(SAMPLE_SIZE);
    targets =
    render_ramlinear<{MAIN_LIN_ID}>,
    render_ramlinear_ert_ei<{MAIN_LIN_ID}>
}

// Parallel, 2K volume, full optimisations
// todo render_parallel_ram<{MAIN_BLOCK_ID}> crashes on full RAM
criterion_group! {
    name = parallel_2k;
    config = Criterion::default().significance_level(0.1).sample_size(SAMPLE_SIZE);
    targets =
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

criterion_group! {
    name = parallel_other;
    config = Criterion::default().significance_level(0.1).sample_size(SAMPLE_SIZE);
    targets = render_parallel_float<{MAIN_BLOCK_ID}>, render_parallel_ram<{MAIN_BLOCK_ID}>
}

// Testing mains

//criterion_main!(parallel);
//criterion_main!(block_side);
//criterion_main!(sequential_ei);

// Find best parallel worker params

//criterion_main!(parallel_params);

// test main
criterion_main!(
    // Ram vs Stream vs Float in single thread
    sequential_linear_float_small,
    sequential_linear_ram_small,
    sequential_linear_stream_small,
    sequential_block_float_small,
    sequential_block_stream_small,
    sequential_block_ram_small,
    // Main: 2K volume ST and MT
    sequential_linear_ram_2k,
    parallel_2k,
    // 4K streamed (parallel only)
    //parallel_4k,
    // Camera angles

    // Determining block side
    //block_side
);
