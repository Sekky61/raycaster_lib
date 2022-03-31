mod common;
use std::sync::{Arc, RwLock};

use common::*;
use raycaster_lib::render::{RenderThread, RendererFront, RendererMessage, SerialRenderer};

//type CamPos = (Point3<f32>, Vector3<f32>); // todo add to benchoptions

const DEFAULT_POSITIONS: [(Point3<f32>, Vector3<f32>); 2] = [
    (point![100.0, 100.0, 100.0], vector![-1.0, -1.0, -1.0]),
    (point![100.0, 100.0, 300.0], vector![-0.2, -0.2, -1.0]),
];

fn render_lin_ret_closure(bench_options: BenchOptions) -> impl FnOnce(&mut Criterion) {
    // todo struct bench_render_options
    move |c: &mut Criterion| {
        let camera = PerspectiveCamera::new(POSITION, DIRECTION);
        let shared_camera = Arc::new(RwLock::new(camera));
        let volume: LinearVolume = get_volume();

        let BenchOptions {
            render_options,
            bench_name,
        } = bench_options;

        let serial_r = SerialRenderer::new(volume, shared_camera, render_options);

        let mut front = RendererFront::new();

        let positions = CameraPositions::from_slice(&DEFAULT_POSITIONS);
        let cam = serial_r.get_camera();
        let sender = front.get_sender();
        let finish_sender = sender.clone();
        let receiver = front.get_receiver();

        front.start_rendering(serial_r);

        c.bench_function(&bench_name, move |b| {
            let sender_in = sender.clone();
            let receiver = receiver.clone();
            let cam = cam.clone();
            let mut positions = positions.clone();

            b.iter_batched(
                move || {
                    // Setup
                    let (pos, dir) = match positions.next() {
                        Some(t) => t,
                        None => {
                            positions.reset();
                            positions.next().unwrap()
                        }
                    };
                    {
                        // set another cam pos
                        let mut cam_guard = cam.write().unwrap();

                        cam_guard.set_pos(pos);
                        cam_guard.set_direction(dir);
                    }
                },
                move |()| {
                    // measured part
                    sender_in.send(RendererMessage::StartRendering).unwrap();
                    receiver.recv().unwrap();
                },
                criterion::BatchSize::PerIteration,
            );
        });
        // Cleanup
        finish_sender.send(RendererMessage::ShutDown).unwrap();
        front.finish();
    }
}

fn render_linear(c: &mut Criterion) {
    let render_options = RenderOptions {
        resolution: (WIDTH, HEIGHT),
        ray_termination: false,
        empty_index: false,
    };

    let bench_options = BenchOptions::new(
        render_options,
        format!("Render ST | linear | {WIDTH}x{HEIGHT} | no optim"),
    );
    render_lin_ret_closure(bench_options)(c)
}

fn render_linear_ert(c: &mut Criterion) {
    let render_options = RenderOptions {
        resolution: (WIDTH, HEIGHT),
        ray_termination: true,
        empty_index: false,
    };

    let bench_options = BenchOptions::new(
        render_options,
        format!("Render ST | linear | {WIDTH}x{HEIGHT} | ERT"),
    );
    render_lin_ret_closure(bench_options)(c)
}

fn render_linear_ei(c: &mut Criterion) {
    let render_options = RenderOptions {
        resolution: (WIDTH, HEIGHT),
        ray_termination: false,
        empty_index: true,
    };

    let bench_options = BenchOptions::new(
        render_options,
        format!("Render ST | linear | {WIDTH}x{HEIGHT} | EI"),
    );
    render_lin_ret_closure(bench_options)(c)
}

fn render_linear_ert_ei(c: &mut Criterion) {
    let render_options = RenderOptions {
        resolution: (WIDTH, HEIGHT),
        ray_termination: true,
        empty_index: true,
    };

    let bench_options = BenchOptions::new(
        render_options,
        format!("Render ST | linear | {WIDTH}x{HEIGHT} | ERT + EI"),
    );
    render_lin_ret_closure(bench_options)(c)
}

criterion_group! {
    name = benches;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default().significance_level(0.1).sample_size(10);
    targets = render_linear, render_linear_ert, render_linear_ei, render_linear_ert_ei
}

criterion_main!(benches);
