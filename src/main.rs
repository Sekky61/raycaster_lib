//extern crate minifb;

mod camera;

mod vol_reader;
use nalgebra::vector;
use sixtyfps::{Image, Rgb8Pixel, SharedPixelBuffer};
use vol_reader::Volume;

use minifb::{Key, Window, WindowOptions};

use crate::camera::{BoundBox, Camera};

const WIDTH: usize = 512;
const HEIGHT: usize = 512;

sixtyfps::sixtyfps! {
    import { Slider, HorizontalBox, VerticalBox, GroupBox, ComboBox } from "sixtyfps_widgets.60";

    export MainWindow := Window {
        title: "SixtyFPS Image Filter Integration Example";
        preferred-width: 800px;
        preferred-height: 600px;

        property original-image <=> original.source;

        callback x_changed;
        callback y_changed;

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
            width: 100px;
            height: 20px;
            value: 42;
            changed => {
                // emit the callback
                root.x_changed()
            }
        }
        y_slider := Slider {
            y: 30px;
            width: 100px;
            height: 20px;
            value: 70;
            changed => {
                // emit the callback
                root.y_changed()
            }
        }
    }
}

fn on_x_changed(x: f32) {
    println!("x_changed {}", x);
}

fn render_to_pixel_buffer(camera: &Camera, bbox: &BoundBox, buffer: &mut [Rgb8Pixel]) {
    camera.cast_rays(bbox, buffer);
}

fn main() {
    let mut pixel_buffer = SharedPixelBuffer::<Rgb8Pixel>::new(WIDTH, HEIGHT);

    let camera = Camera::new(WIDTH, HEIGHT);
    let volume = Volume::from_file("Skull.vol");
    let bbox = BoundBox::from_volume(volume);
    render_to_pixel_buffer(&camera, &bbox, pixel_buffer.make_mut_slice());

    let image = Image::from_rgb8(pixel_buffer);

    let main_window = MainWindow::new();

    main_window.on_x_changed(|| {
        println!("x_changed ");
    });

    main_window.set_original_image(image);

    main_window.run();
}

/*
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
