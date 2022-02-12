use criterion::{black_box, criterion_group, criterion_main, Criterion};

use nalgebra::point;
use raycaster_lib::{
    camera::TargetCamera,
    render::{RenderOptions, Renderer},
    volumetric::{
        from_file, parse::skull_parser, BuildVolume, LinearVolume, Volume, VolumeMetadata,
    },
};

fn get_volume<V>() -> V
where
    V: Volume + BuildVolume<VolumeMetadata>,
{
    from_file("volumes/Skull.vol", skull_parser).unwrap()
}

fn get_ui_from_usize(c: &mut Criterion) {
    let volume = get_volume();
    let camera = TargetCamera::new(512, 512);

    let mut renderer = Renderer::<LinearVolume, _>::new(volume, camera);
    renderer.set_render_options(RenderOptions {
        ray_termination: true,
        empty_index: true,
        multi_thread: false,
    });

    let pos = point![40, 28, 10]; // vector![40.3, 28.9, 10.7];

    c.bench_function("get blocktype from usize position", |b| {
        b.iter(|| {
            renderer
                .empty_index
                .get_index_from_usize(black_box(3), black_box(&pos))
        });
    });
}

fn get_ui_from_float(c: &mut Criterion) {
    let camera = TargetCamera::new(512, 512);
    let volume = get_volume();

    let mut renderer = Renderer::<LinearVolume, _>::new(volume, camera);
    renderer.set_render_options(RenderOptions {
        ray_termination: true,
        empty_index: true,
        multi_thread: false,
    });

    let pos = point![40.3, 28.9, 10.7];

    c.bench_function("get blocktype from float position", |b| {
        b.iter(|| {
            renderer
                .empty_index
                .get_index_from_float(black_box(3), black_box(&pos))
        });
    });
}

criterion_group!(get_ei_fl_vs_usize, get_ui_from_float, get_ui_from_usize);
criterion_main!(get_ei_fl_vs_usize);
