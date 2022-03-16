mod common;
use common::*;

fn render_linear(c: &mut Criterion) {
    let camera = PerspectiveCamera::new(POSITION, DIRECTION);
    let volume = get_volume();

    let render_options = RenderOptions {
        resolution: (WIDTH, HEIGHT),
        ray_termination: true,
        empty_index: true,
    };
    let mut renderer = Renderer::<LinearVolume>::new(volume, render_options);

    let mut buffer = vec![0; 3 * WIDTH * HEIGHT];

    c.bench_function("render linear 512x512", |b| {
        b.iter(|| renderer.render_to_buffer(&camera, buffer.as_mut_slice()));
    });
}

fn render_linear_ei(c: &mut Criterion) {
    let camera = PerspectiveCamera::new(POSITION, DIRECTION);
    let volume = get_volume();

    let render_options = RenderOptions {
        resolution: (WIDTH, HEIGHT),
        ray_termination: true,
        empty_index: true,
    };
    let mut renderer = Renderer::<LinearVolume>::new(volume, render_options);

    let mut buffer = vec![0; 3 * WIDTH * HEIGHT];

    c.bench_function("render linear 512x512 empty index", |b| {
        b.iter(|| renderer.render_to_buffer(&camera, buffer.as_mut_slice()));
    });
}

fn render_block(c: &mut Criterion) {
    let camera = PerspectiveCamera::new(POSITION, DIRECTION);
    let volume = get_volume();

    let render_options = RenderOptions {
        resolution: (WIDTH, HEIGHT),
        ray_termination: true,
        empty_index: true,
    };
    let mut renderer = Renderer::<LinearVolume>::new(volume, render_options);

    let mut buffer = vec![0; 3 * WIDTH * HEIGHT];

    c.bench_function("render block 512x512", |b| {
        b.iter(|| renderer.render_to_buffer(&camera, buffer.as_mut_slice()));
    });
}

fn render_block_ei(c: &mut Criterion) {
    let camera = PerspectiveCamera::new(POSITION, DIRECTION);
    let volume = get_volume();

    let render_options = RenderOptions {
        resolution: (WIDTH, HEIGHT),
        ray_termination: true,
        empty_index: true,
    };
    let mut renderer = Renderer::<LinearVolume>::new(volume, render_options);

    let mut buffer = vec![0; 3 * WIDTH * HEIGHT];

    c.bench_function("render block 512x512 empty index", |b| {
        b.iter(|| renderer.render_to_buffer(&camera, buffer.as_mut_slice()));
    });
}

criterion_group! {
    name = benches;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default().significance_level(0.1).sample_size(10);
    targets = render_linear, render_linear_ei, render_block, render_block_ei
}

criterion_main!(benches);
