// minifb backend

use minifb::{Key, Window, WindowOptions};

use raycaster_lib::camera::{BoundBox, Camera};
use raycaster_lib::vol_reader;
use raycaster_lib::volume::Volume;

use nalgebra::vector;

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

    let mut camera = Camera::new(WIDTH, HEIGHT);

    // Limit to max ~60 fps update rate
    //window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
    window.limit_update_rate(Some(std::time::Duration::from_millis(100)));

    let vol = vol_reader::from_file("Skull.vol");
    let volume = match vol {
        Ok(vol) => vol,
        Err(e) => {
            panic!("{}", e)
        }
    };
    let boxx = BoundBox::from_volume(volume);

    println!("Box {:?}", boxx);

    let mut frame_buffer = vec![0; WIDTH * HEIGHT * 3];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let w = window.is_key_down(Key::W);
        let a = window.is_key_down(Key::A);
        let s = window.is_key_down(Key::S);
        let d = window.is_key_down(Key::D);
        let plus = window.is_key_down(Key::NumPadPlus);
        let minus = window.is_key_down(Key::NumPadMinus);

        let step = 30.0;

        let x_p = if plus { step } else { 0.0 };
        let x_m = if minus { -step } else { 0.0 };
        let y_p = if w { step } else { 0.0 };
        let y_m = if s { -step } else { 0.0 };

        let z_p = if a { step } else { 0.0 };
        let z_m = if d { -step } else { 0.0 };

        let change = vector![z_p + z_m, x_p + x_m, y_p + y_m];

        camera.change_pos(change);

        camera.cast_rays_bytes(&boxx, frame_buffer.as_mut_slice());

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        // window
        //     .update_with_buffer(&frame_buffer[..], WIDTH, HEIGHT)
        //     .unwrap();
    }
}
