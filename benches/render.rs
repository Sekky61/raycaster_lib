use criterion::{black_box, criterion_group, criterion_main, Criterion};

use raycaster_lib::{
    camera::{BoundBox, Camera},
    render_frame, render_to_byte_buffer,
};

use raycaster_lib::volume::vol_reader::from_file;

fn full_render(c: &mut Criterion) {
    c.bench_function("file read, alloc, render 512x512", |b| {
        b.iter(|| render_frame(black_box(512), black_box(512)));
    });
}

fn pure_render(c: &mut Criterion) {
    c.bench_function("render 512x512", |b| {
        let camera = Camera::new(512, 512);
        let read_result = vol_reader::from_file("Skull.vol");
        //let volume = Volume::white_vol();

        let volume = match read_result {
            Ok(vol) => vol,
            Err(message) => {
                eprint!("{}", message);
                std::process::exit(1);
            }
        };

        let bbox = BoundBox::from_volume(volume);

        let mut buffer: Vec<u8> = vec![0; 512 * 512 * 3];

        b.iter(|| {
            render_to_byte_buffer(
                black_box(&camera),
                black_box(&bbox),
                black_box(buffer.as_mut_slice()),
            )
        });
    });
}

criterion_group!(benches, full_render, pure_render);
criterion_main!(benches);
