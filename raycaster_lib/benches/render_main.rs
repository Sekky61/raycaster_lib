use common::volume_files::*;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use render_benchmarks::{
    block::*, block_stream::*, float_linear::*, linear_stream::*, multi_thread::*,
};

mod common;
mod render_benchmarks;

const SAMPLE_SIZE: usize = 10;

// Single thread

// Linear Volume
criterion_group! {
    name = sequential_linear_small;
    config = Criterion::default().significance_level(0.1).sample_size(SAMPLE_SIZE);
    targets =
        render_linear_float<{SMALL_SHAPES_LIN_ID}>,
        render_linear_float_ert<{SMALL_SHAPES_LIN_ID}>,
        render_linear_float_ei<{SMALL_SHAPES_LIN_ID}>,
        render_linear_float_ert_ei<{SMALL_SHAPES_LIN_ID}>
}

// Linear Volume
criterion_group! {
    name = sequential_linear_small_float;
    config = Criterion::default().significance_level(0.1).sample_size(SAMPLE_SIZE);
    targets =
        render_linear_float<{SMALL_SHAPES_LIN_ID}>,
        render_linear_float_ert<{SMALL_SHAPES_LIN_ID}>,
        render_linear_float_ei<{SMALL_SHAPES_LIN_ID}>,
        render_linear_float_ert_ei<{SMALL_SHAPES_LIN_ID}>
}

// Linear Volume Streamed
criterion_group! {
    name = sequential_streamlinear_skull;
    config = Criterion::default().significance_level(0.1).sample_size(SAMPLE_SIZE);
    targets =
        render_streamlinear<{SMALL_SHAPES_LIN_ID}>,
        render_streamlinear_ert<{SMALL_SHAPES_LIN_ID}>,
        render_streamlinear_ei<{SMALL_SHAPES_LIN_ID}>,
        render_streamlinear_ert_ei<{SMALL_SHAPES_LIN_ID}>
}

// Block Volume
criterion_group! {
    name = sequential_block_skull;
    config = Criterion::default().significance_level(0.1).sample_size(SAMPLE_SIZE);
    targets =
        render_block<{SMALL_SHAPES_LIN_ID}>,
        render_block_ert<{SMALL_SHAPES_LIN_ID}>,
        render_block_ei<{SMALL_SHAPES_LIN_ID}>,
        render_block_ert_ei<{SMALL_SHAPES_LIN_ID}>
}

// Block Volume Streamed
// todo special volume must be used
criterion_group! {
    name = sequential_streamblock_skull;
    config = Criterion::default().significance_level(0.1).sample_size(SAMPLE_SIZE);
    targets =
        render_streamed<{SMALL_SHAPES_LIN_ID}>,
        render_streamed_ert<{SMALL_SHAPES_LIN_ID}>,
        render_streamed_ei<{SMALL_SHAPES_LIN_ID}>,
        render_streamed_ert_ei<{SMALL_SHAPES_LIN_ID}>
}

// Parallel
// todo special volume must be used
criterion_group! {
    name = parallel;
    config = Criterion::default().significance_level(0.1).sample_size(SAMPLE_SIZE);
    targets =
        render_parallel_mem<{MAIN_BLOCK_ID}>,
        render_parallel_stream<{MAIN_BLOCK_ID}>
}

criterion_group! {
    name = huge_volume;
    config = Criterion::default().significance_level(0.1).sample_size(SAMPLE_SIZE);
    targets =
        render_parallel_stream<{HUGE_ID}>
}

// Testing

criterion_group! {
    name = sequential_ei;
    config = Criterion::default().significance_level(0.1).sample_size(SAMPLE_SIZE);
    targets = render_linear_float_ei<{SMALL_SHAPES_LIN_ID}>
}

criterion_group! {
    name = parallel_params;
    config = Criterion::default().significance_level(0.1).sample_size(SAMPLE_SIZE);
    targets = render_parallel_mem<{SMALL_SHAPES_LIN_ID}>
}

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
