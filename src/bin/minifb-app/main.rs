//extern crate minifb;

use nalgebra::vector;
use raycaster_lib::{
    render::BufferStatus,
    volumetric::{BlockVolume, LinearVolume},
    Camera, RenderOptions, Renderer,
};

use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 512;
const HEIGHT: usize = 512;

fn main() {
    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    //window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
    //window.limit_update_rate(Some(std::time::Duration::from_millis(100)));

    let volume = raycaster_lib::vol_reader::from_file("Skull.vol")
        .expect("bad read of file")
        .build();

    let camera = raycaster_lib::Camera::new(WIDTH, HEIGHT);

    let mut raycast_renderer = Renderer::<BlockVolume>::new(volume, camera);

    raycast_renderer.set_render_options(RenderOptions {
        ray_termination: true,
        empty_index: true,
        multi_thread: false,
    });

    let mut buf_vec = vec![0; 3 * WIDTH * HEIGHT];

    println!("b1");

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // for i in buffer.iter_mut() {
        //     *i = 0; // write something more funny here!
        // }
        println!("b2");

        let w = window.is_key_down(Key::W);
        let a = window.is_key_down(Key::A);
        let s = window.is_key_down(Key::S);
        let d = window.is_key_down(Key::D);
        let plus = window.is_key_down(Key::NumPadPlus);
        let minus = window.is_key_down(Key::NumPadMinus);

        println!("b3");

        let step = 10.0;

        let x_p = if plus { step } else { 0.0 };
        let x_m = if minus { -step } else { 0.0 };
        let y_p = if w { step } else { 0.0 };
        let y_m = if s { -step } else { 0.0 };

        let z_p = if a { step } else { 0.0 };
        let z_m = if d { -step } else { 0.0 };

        let change = vector![z_p + z_m, x_p + x_m, y_p + y_m];

        println!("b4");

        raycast_renderer.render_to_buffer(buf_vec.as_mut_slice());

        raycast_renderer.change_camera_pos(change);

        println!("b5");

        let converted: Vec<_> = buf_vec
            .as_slice()
            .chunks(3)
            .map(|ch| {
                ((ch[0] as u32) << 24)
                    | ((ch[1] as u32) << 16)
                    | ((ch[2] as u32) << 8)
                    | (255 as u32)
            })
            .collect();

        println!("b6");

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(converted.as_slice(), WIDTH, HEIGHT)
            .unwrap();
        println!("b7");
    }
}
