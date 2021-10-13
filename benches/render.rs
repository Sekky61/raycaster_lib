use criterion::{black_box, criterion_group, criterion_main, Criterion};

use raycaster_lib::renderer::Renderer;
use raycaster_lib::{render_frame, render_to_byte_buffer, Camera};

use raycaster_lib::volumetric::{vol_reader, LinearVolume};

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

        let volume = LinearVolume::from(volume_b);

        let renderer = Renderer::new(volume, camera);

        let mut buffer: Vec<u8> = vec![0; 512 * 512 * 3];

        b.iter(|| render_to_byte_buffer(black_box(&renderer), black_box(&mut buffer)));
    });
}

criterion_group!(benches, full_render, pure_render);
criterion_main!(benches);
