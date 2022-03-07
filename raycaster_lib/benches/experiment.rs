mod common;
use common::*;

fn get_ui_from_usize(c: &mut Criterion) {
    return;

    let volume = get_volume();
    let camera = PerspectiveCamera::new(POSITION, DIRECTION);

    let mut renderer = Renderer::<LinearVolume, _>::new(volume, camera);
    renderer.set_render_options(RenderOptions {
        resolution: (WIDTH, HEIGHT),
        ray_termination: true,
        empty_index: true,
    });

    c.bench_function("get blocktype from usize position", |b| {
        b.iter(|| {
            // unused test
        });
    });
}

fn get_ui_from_float(c: &mut Criterion) {
    return;

    let camera = PerspectiveCamera::new(POSITION, DIRECTION);
    let volume = get_volume();

    let mut renderer = Renderer::<LinearVolume, _>::new(volume, camera);
    renderer.set_render_options(RenderOptions {
        resolution: (WIDTH, HEIGHT),
        ray_termination: true,
        empty_index: true,
    });

    c.bench_function("get blocktype from float position", |b| {
        b.iter(|| {
            // unused test
        });
    });
}

criterion_group!(get_ei_fl_vs_usize, get_ui_from_float, get_ui_from_usize);
criterion_main!(get_ei_fl_vs_usize);
