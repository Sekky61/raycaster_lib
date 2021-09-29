//extern crate minifb;

mod camera;

mod vol_reader;
use nalgebra::vector;
use vol_reader::Volume;

use minifb::{Key, Window, WindowOptions};

use crate::camera::Camera;

const WIDTH: usize = 512;
const HEIGHT: usize = 512;

fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    let mut camera = Camera::new();

    // Limit to max ~60 fps update rate
    //window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
    window.limit_update_rate(Some(std::time::Duration::from_millis(100)));

    let vol = Volume::from_file("Skull.vol");
    println!("{}", vol);

    let mut plane = 0;

    let mut time = 0;

    let cam_res = camera.get_resolution();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for i in buffer.iter_mut() {
            *i = 0; // write something more funny here!
        }

        let w = window.is_key_down(Key::W);
        let a = window.is_key_down(Key::A);
        let s = window.is_key_down(Key::S);
        let d = window.is_key_down(Key::D);
        let plus = window.is_key_down(Key::NumPadPlus);
        let minus = window.is_key_down(Key::NumPadMinus);

        let x_p = if plus { 0.5 } else { 0.0 };
        let x_m = if minus { -0.5 } else { 0.0 };
        let y_p = if w { 0.5 } else { 0.0 };
        let y_m = if s { -0.5 } else { 0.0 };

        let z_p = if a { 0.5 } else { 0.0 };
        let z_m = if d { -0.5 } else { 0.0 };

        let change = vector![z_p + z_m, x_p + x_m, y_p + y_m];

        camera.change_pos(change);

        let cast_buf = camera.cast_rays();

        time += 1;

        for h in 0..HEIGHT {
            for w in 0..WIDTH {
                buffer[h * WIDTH + w] = cast_buf[h * cam_res.0 + w];
            }
        }

        //let max_w = std::cmp::min(WIDTH, vol.z); //todo flipped?
        //let max_h = std::cmp::min(HEIGHT, vol.y);
        // for h in 0..max_h {
        //     for w in 0..max_w {
        //         let color = vol.get_3d(plane, h, w);
        //         buffer[h * WIDTH + w] = color.to_int();
        //     }
        // }

        plane += 1;

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
