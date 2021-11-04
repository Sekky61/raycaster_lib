use criterion::{black_box, criterion_group, criterion_main, Criterion};

use nalgebra::vector;
use raycaster_lib::render::Renderer;
use raycaster_lib::volumetric::{vol_reader, LinearVolume};
use raycaster_lib::{Camera, RenderOptions};

fn get_ui_from_usize(c: &mut Criterion) {
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

    let pos = vector![40, 28, 10]; // vector![40.3, 28.9, 10.7];

    c.bench_function("get blocktype from usize position", |b| {
        b.iter(|| {
            renderer
                .empty_index
                .get_index_from_usize(black_box(3), black_box(&pos))
        });
    });
}

fn get_ui_from_float(c: &mut Criterion) {
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

    let pos = vector![40.3, 28.9, 10.7];

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
