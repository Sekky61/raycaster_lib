use criterion::{black_box, criterion_group, criterion_main, Criterion};

use nalgebra::vector;
use raycaster_lib::render::Renderer;
use raycaster_lib::volumetric::{vol_reader, BlockVolume, LinearVolume};
use raycaster_lib::{render_frame, Camera, RenderOptions};

fn full_render(c: &mut Criterion) {
    c.bench_function("file read, alloc, render 512x512", |b| {
        b.iter(|| render_frame(black_box(512), black_box(512)));
    });
}

fn render_linear(c: &mut Criterion) {
    let camera = Camera::new(512, 512);
    let read_result = vol_reader::from_file("Skull.vol");

    let volume_b = match read_result {
        Ok(vol) => vol,
        Err(message) => {
            eprint!("{}", message);
            std::process::exit(1);
        }
    };

    let volume = volume_b.build();

    let mut renderer = Renderer::<LinearVolume>::new(volume, camera);
    renderer.set_render_options(RenderOptions {
        ray_termination: true,
        empty_index: true,
        multi_thread: false,
    });

    c.bench_function("render linear 512x512", |b| {
        b.iter(|| renderer.render_to_buffer());
    });
}

fn render_block(c: &mut Criterion) {
    let camera = Camera::new(512, 512);
    let read_result = vol_reader::from_file("Skull.vol");

    let volume_b = match read_result {
        Ok(vol) => vol,
        Err(message) => {
            eprint!("{}", message);
            std::process::exit(1);
        }
    };

    let volume = volume_b.build();

    let mut renderer = Renderer::<BlockVolume>::new(volume, camera);
    renderer.set_render_options(RenderOptions {
        ray_termination: true,
        empty_index: true,
        multi_thread: false,
    });
    c.bench_function("render block 512x512", |b| {
        b.iter(|| renderer.render_to_buffer());
    });
}

criterion_group! {
    name = benches;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default().significance_level(0.1).sample_size(10);
    targets = render_linear, render_block
}

criterion_main!(benches);
