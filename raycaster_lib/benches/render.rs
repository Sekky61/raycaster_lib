use criterion::{criterion_group, criterion_main, Criterion};

use raycaster_lib::{
    camera::TargetCamera,
    render::{RenderOptions, Renderer},
    volumetric::{
        parse::vol_parser, BlockVolume, BuildVolume, LinearVolume, ParsedVolumeBuilder, Volume,
        VolumeBuilder,
    },
};

const WIDTH: usize = 512;
const HEIGHT: usize = 512;

fn get_volume<V>() -> V
where
    V: Volume + BuildVolume<ParsedVolumeBuilder<u8>>,
{
    // Build Renderer and Volume
    let vb = VolumeBuilder::from_file("volumes/Skull.vol").expect("bad read of file");

    let parsed_vb = vol_parser(vb).unwrap();
    V::build(parsed_vb)
}

fn render_linear(c: &mut Criterion) {
    let camera = TargetCamera::new(WIDTH, HEIGHT);
    let volume = get_volume();

    let mut renderer = Renderer::<LinearVolume, _>::new(volume, camera);
    renderer.set_render_options(RenderOptions {
        ray_termination: true,
        empty_index: false,
        multi_thread: false,
    });

    let mut buffer = vec![0; 3 * WIDTH * HEIGHT];

    c.bench_function("render linear 512x512", |b| {
        b.iter(|| renderer.render_to_buffer(buffer.as_mut_slice()));
    });
}

fn render_linear_ei(c: &mut Criterion) {
    let camera = TargetCamera::new(WIDTH, HEIGHT);
    let volume = get_volume();

    let mut renderer = Renderer::<LinearVolume, _>::new(volume, camera);
    renderer.set_render_options(RenderOptions {
        ray_termination: true,
        empty_index: true,
        multi_thread: false,
    });

    let mut buffer = vec![0; 3 * WIDTH * HEIGHT];

    c.bench_function("render linear 512x512 empty index", |b| {
        b.iter(|| renderer.render_to_buffer(buffer.as_mut_slice()));
    });
}

fn render_block(c: &mut Criterion) {
    let camera = TargetCamera::new(WIDTH, HEIGHT);
    let volume = get_volume();

    let mut renderer = Renderer::<BlockVolume, _>::new(volume, camera);
    renderer.set_render_options(RenderOptions {
        ray_termination: true,
        empty_index: false,
        multi_thread: false,
    });

    let mut buffer = vec![0; 3 * WIDTH * HEIGHT];

    c.bench_function("render block 512x512", |b| {
        b.iter(|| renderer.render_to_buffer(buffer.as_mut_slice()));
    });
}

fn render_block_ei(c: &mut Criterion) {
    let camera = TargetCamera::new(WIDTH, HEIGHT);
    let volume = get_volume();

    let mut renderer = Renderer::<BlockVolume, _>::new(volume, camera);
    renderer.set_render_options(RenderOptions {
        ray_termination: true,
        empty_index: true,
        multi_thread: false,
    });

    let mut buffer = vec![0; 3 * WIDTH * HEIGHT];

    c.bench_function("render block 512x512 empty index", |b| {
        b.iter(|| renderer.render_to_buffer(buffer.as_mut_slice()));
    });
}

criterion_group! {
    name = benches;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default().significance_level(0.1).sample_size(10);
    targets = render_linear, render_block, render_linear_ei, render_block_ei
}

criterion_main!(benches);
