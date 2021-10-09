//extern crate minifb;

use std::sync::{Arc, Mutex};

mod camera;

mod block;
mod vol_reader;
mod volume;

use nalgebra::vector;
use sixtyfps::{Image, Rgb8Pixel, SharedPixelBuffer};
use volume::Volume;

use crate::camera::{BoundBox, Camera};

const WIDTH: usize = 256;
const HEIGHT: usize = 256;

sixtyfps::sixtyfps! {
    import { Slider, HorizontalBox, VerticalBox, GroupBox, ComboBox } from "sixtyfps_widgets.60";

    export MainWindow := Window {
        title: "Raycast demo app";
        preferred-width: 800px;
        preferred-height: 600px;

        property original-image <=> original.source;

        property x_coord <=> x_slider.value;
        property y_coord <=> y_slider.value;
        property z_coord <=> z_slider.value;

        callback x_changed;
        callback y_changed;
        callback z_changed;

        HorizontalBox {
            VerticalBox {
                Text {
                    font-size: 20px;
                    text: "Original Image";
                    horizontal-alignment: center;
                }
                original := Image { }

            }
        }
        x_slider := Slider {
            width: 200px;
            height: 20px;
            value: 200;
            minimum: -300;
            maximum: 500;
            changed => {
                // emit the callback
                root.x_changed()
            }
        }
        y_slider := Slider {
            y: 30px;
            width: 200px;
            height: 20px;
            value: 250;
            minimum: -300;
            maximum: 500;
            changed => {
                // emit the callback
                root.y_changed()
            }
        }
        z_slider := Slider {
            y: 60px;
            width: 200px;
            height: 20px;
            value: 270;
            minimum: -300;
            maximum: 500;
            changed => {
                // emit the callback
                root.z_changed()
            }
        }
    }
}

fn render_to_byte_buffer(camera: &Camera, bbox: &BoundBox, buffer: &mut [u8]) {
    camera.cast_rays_bytes(bbox, buffer);
}

fn main() {
    // window instance
    let main_window = MainWindow::new();

    let mut camera = Camera::new(WIDTH, HEIGHT);
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

    // threading communication
    //let (tx, rx) = mpsc::channel();

    // shared state (camera coordinates)
    let camera_init = vector![-4.0, 108.0, -85.0];
    main_window.set_x_coord(camera_init.x);
    main_window.set_y_coord(camera_init.y);
    main_window.set_z_coord(camera_init.z);
    let global_coords = Arc::new(Mutex::new(camera_init));

    // setting x coordinate
    let main_window_weak = main_window.as_weak();
    let gl_coords_x = global_coords.clone();

    main_window.on_x_changed(move || {
        let win = main_window_weak.unwrap();
        let x: f32 = win.get_x_coord();

        println!("x_changed {}", x);

        let mut old_coords = gl_coords_x.lock().unwrap();

        (*old_coords).x = x;
    });

    // setting y coordinate
    let main_window_weak = main_window.as_weak();
    let gl_coords_y = global_coords.clone();
    //let tx_y = tx.clone();

    main_window.on_y_changed(move || {
        let win = main_window_weak.unwrap();
        let y: f32 = win.get_y_coord();

        println!("y_changed {}", y);

        let mut old_coords = gl_coords_y.lock().unwrap();

        (*old_coords).y = y;
    });

    // setting z coordinate
    let main_window_weak = main_window.as_weak();
    let gl_coords_z = global_coords.clone();
    //let tx_z = tx;

    main_window.on_z_changed(move || {
        let win = main_window_weak.unwrap();
        let z: f32 = win.get_z_coord();

        println!("z_changed {}", z);

        let mut old_coords = gl_coords_z.lock().unwrap();

        (*old_coords).z = z;
    });

    let main_window_weak = main_window.as_weak();

    // rendering thread
    std::thread::spawn(move || loop {
        let new_pos = global_coords.try_lock();

        if let Ok(guard) = new_pos {
            camera.set_pos(*guard);
        }

        let mut buf = vec![0u8; WIDTH * HEIGHT * 4];
        let window_handle_copy = main_window_weak.clone();

        render_to_byte_buffer(&camera, &bbox, buf.as_mut_slice());

        sixtyfps::invoke_from_event_loop(move || {
            let pixel_buffer =
                SharedPixelBuffer::<Rgb8Pixel>::clone_from_slice(&buf[..], WIDTH, HEIGHT);
            let image = Image::from_rgb8(pixel_buffer);
            window_handle_copy.unwrap().set_original_image(image)
        });
    });

    main_window.run(); // blocking
}

// minifb backend

/*
use minifb::{Key, Window, WindowOptions};

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

    let vol = Volume::from_file("Skull.vol");
    let boxx = BoundBox::from_volume(vol);

    println!("Box {:?}", boxx);

    let mut frame_buffer = vec![0; WIDTH * HEIGHT];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // for i in buffer.iter_mut() {
        //     *i = 0; // write something more funny here!
        // }

        let w = window.is_key_down(Key::W);
        let a = window.is_key_down(Key::A);
        let s = window.is_key_down(Key::S);
        let d = window.is_key_down(Key::D);
        let plus = window.is_key_down(Key::NumPadPlus);
        let minus = window.is_key_down(Key::NumPadMinus);

        let step = 2.0;

        let x_p = if plus { step } else { 0.0 };
        let x_m = if minus { -step } else { 0.0 };
        let y_p = if w { step } else { 0.0 };
        let y_m = if s { -step } else { 0.0 };

        let z_p = if a { step } else { 0.0 };
        let z_m = if d { -step } else { 0.0 };

        let change = vector![z_p + z_m, x_p + x_m, y_p + y_m];

        camera.change_pos(change);

        camera.cast_rays(&boxx, frame_buffer.as_mut_slice());

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(&frame_buffer[..], WIDTH, HEIGHT)
            .unwrap();
    }
}*/
