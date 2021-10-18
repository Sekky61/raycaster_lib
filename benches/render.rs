use criterion::{black_box, criterion_group, criterion_main, Criterion};

use raycaster_lib::render::Renderer;
use raycaster_lib::volumetric::{vol_reader, LinearVolume};
use raycaster_lib::{render_frame, Camera, RendererOptions};

fn full_render(c: &mut Criterion) {
    c.bench_function("file read, alloc, render 512x512", |b| {
        b.iter(|| render_frame(black_box(512), black_box(512)));
    });
}

fn pure_render(c: &mut Criterion) {
    c.bench_function("render 512x512", |b| {
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
        renderer.render_settings(RendererOptions {
            ray_termination: true,
            empty_index: false,
            multi_thread: false,
        });

        let mut buffer: Vec<u8> = vec![0; 512 * 512 * 3];

        b.iter(|| renderer.render());
    });
}

criterion_group!(benches, full_render, pure_render);
criterion_main!(benches);
