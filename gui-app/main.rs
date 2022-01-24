//! A simple example that demonstrates using conrod within a basic `winit` window loop, using
//! `glium` to render the `conrod_core::render::Primitives` to screen.

#[macro_use]
extern crate conrod_core;
extern crate conrod_glium;
extern crate conrod_winit;
extern crate glium;

mod support;

use std::{thread, time::Duration};

use parking_lot::RwLock;
use std::sync::Arc;

use conrod_core::{widget, Colorable, Positionable, Sizeable, Widget};
use glium::Surface;
use nalgebra::vector;
use raycaster_lib::{
    render::BufferStatus,
    volumetric::{BlockVolume, LinearVolume},
    RenderOptions, Renderer,
};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 700;

widget_ids! {
    struct Ids { canvas, oval, range_slider, rendered_tex }
}

use conrod_core::image::Id;

struct frame_buffers {
    active_frame: bool,
    frame1: Id, // false
    frame2: Id, // true
}

fn main() {
    // Build the window.
    let event_loop = glium::glutin::event_loop::EventLoop::new();
    let window = glium::glutin::window::WindowBuilder::new()
        .with_title("Hello Conrod!")
        .with_inner_size(glium::glutin::dpi::LogicalSize::new(WIDTH, HEIGHT));
    let context = glium::glutin::ContextBuilder::new()
        .with_vsync(true)
        .with_multisampling(4);
    let display = glium::Display::new(window, context, &event_loop).unwrap();

    // construct our `Ui`.
    let mut ui = conrod_core::UiBuilder::new([WIDTH as f64, HEIGHT as f64]).build();

    // A unique identifier for each widget.
    let ids = Ids::new(ui.widget_id_generator());

    // Generate the widget identifiers.
    // widget_ids!(struct Ids { text, rendered_tex });
    // let ids = Ids::new(ui.widget_id_generator());

    // Add a `Font` to the `Ui`'s `font::Map` from file.
    ui.fonts.insert_from_file("./Roboto-Regular.ttf").unwrap();

    // A type used for converting `conrod_core::render::Primitives` into `Command`s that can be used
    // for drawing to the glium `Surface`.
    let mut renderer = conrod_glium::Renderer::new(&display).unwrap();

    let volume = raycaster_lib::vol_reader::from_file("Skull.vol")
        .expect("bad read of file")
        .build();

    let camera = raycaster_lib::Camera::new(512, 512);

    let mut raycast_renderer = Renderer::<BlockVolume>::new(volume, camera);

    raycast_renderer.set_render_options(RenderOptions {
        ray_termination: true,
        empty_index: true,
        multi_thread: false,
    });

    let mut image_map = conrod_core::image::Map::new();

    let empty_tex =
        glium::texture::Texture2d::empty(&display, 512, 512).expect("empty texture error");

    let render_output_id = image_map.insert(empty_tex);

    let frame1 = Arc::new(RwLock::new(vec![0; 3 * 512 * 512]));
    let frame1_cpy = frame1.clone();

    let frame2 = Arc::new(RwLock::new(vec![0; 3 * 512 * 512]));
    let frame2_cpy = frame2.clone();

    let bufferstatus = Arc::new(RwLock::new(BufferStatus::new()));
    let bufferstatus_cpy = bufferstatus.clone();

    thread::spawn(move || loop {
        let render_target = {
            let fr = bufferstatus_cpy.read();
            fr.get_render_target()
        };

        let mut buff_lock = match render_target {
            1 => frame1_cpy.write(),
            2 => frame2_cpy.write(),
            _ => continue,
        };

        raycast_renderer.render_to_buffer(&mut (buff_lock));

        drop(buff_lock);
        {
            let mut status = bufferstatus_cpy.write();
            match render_target {
                1 => (*status).0 = true,
                2 => (*status).1 = true,
                _ => continue,
            };
        };

        raycast_renderer.change_camera_pos(vector![20.0, 20.0, 20.0]);
    });

    let texture_1 = glium::texture::Texture2d::empty(&display, 512, 512).unwrap();

    // The image map describing each of our widget->image mappings (in our case, none).
    // let mut image_map = conrod_core::image::Map::new();

    let render_target_id = image_map.insert(texture_1);

    let mut oval_range = (0.25, 0.75);

    support::run_loop(display, event_loop, move |request, display| {
        match request {
            support::Request::Event {
                event,
                should_update_ui,
                should_exit,
            } => {
                // Use the `winit` backend feature to convert the winit event to a conrod one.
                if let Some(event) = support::convert_event(&event, &display.gl_window().window()) {
                    ui.handle_event(event);
                    *should_update_ui = true;
                }

                let finished_render = {
                    let fr = bufferstatus.read();
                    fr.get_finished_target()
                };

                if finished_render != 0 {
                    let buff_lock = match finished_render {
                        1 => frame1.read(),
                        2 => frame2.read(),
                        _ => panic!("Should not happen"),
                    };

                    let raw_image =
                        glium::texture::RawImage2d::from_raw_rgb(buff_lock.to_owned(), (512, 512));

                    drop(buff_lock);

                    {
                        let mut fr = bufferstatus.write();
                        match finished_render {
                            1 => (*fr).0 = false,
                            2 => (*fr).1 = false,
                            _ => panic!("Should not happen 2"),
                        }
                    };

                    let rendered_texture =
                        glium::texture::Texture2d::new(display, raw_image).unwrap();
                    image_map.replace(render_target_id, rendered_texture);
                    *should_update_ui = true;
                    println!("UI update set true");
                };

                match event {
                    glium::glutin::event::Event::WindowEvent { event, .. } => match event {
                        // Break from the loop upon `Escape`.
                        glium::glutin::event::WindowEvent::CloseRequested
                        | glium::glutin::event::WindowEvent::KeyboardInput {
                            input:
                                glium::glutin::event::KeyboardInput {
                                    virtual_keycode:
                                        Some(glium::glutin::event::VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *should_exit = true,
                        _ => {}
                    },
                    _ => {}
                }
            }
            support::Request::SetUi { needs_redraw } => {
                // let mut im_vec = vec![0; 512 * 512 * 3];
                // raycast_renderer.change_camera_pos(vector![20.0, 20.0, 20.0]);
                // raycast_renderer.render(&mut im_vec[..]);

                set_ui(ui.set_widgets(), &ids, &mut oval_range, render_target_id);

                //*needs_redraw |= ui.has_changed();
                *needs_redraw |= true;
            }
            support::Request::Redraw => {
                // Render the `Ui` and then display it on the screen.
                let primitives = ui.draw();

                renderer.fill(display, primitives, &image_map);
                let mut target = display.draw();
                target.clear_color_srgb(0.0, 0.0, 0.0, 1.0);

                renderer.draw(display, &mut target, &image_map).unwrap();
                target.finish().unwrap();
            }
        }
    });
}

conrod_winit::v023_conversion_fns!();

// Declare the `WidgetId`s and instantiate the widgets.
fn set_ui(
    ref mut ui: conrod_core::UiCell,
    ids: &Ids,
    oval_range: &mut (conrod_core::Scalar, conrod_core::Scalar),
    image_id: Id,
) {
    use conrod_core::{color, widget, Colorable, Positionable, Sizeable, Widget};

    widget::Canvas::new()
        .color(color::DARK_CHARCOAL)
        .set(ids.canvas, ui);

    const PAD: conrod_core::Scalar = 20.0;
    let (ref mut start, ref mut end) = *oval_range;
    let min = 0.0;
    let max = 1.0;
    for (edge, value) in widget::RangeSlider::new(*start, *end, min, max)
        .color(color::LIGHT_BLUE)
        .padded_w_of(ids.canvas, PAD)
        .h(30.0)
        .mid_top_with_margin_on(ids.canvas, PAD)
        .set(ids.range_slider, ui)
    {
        match edge {
            widget::range_slider::Edge::Start => *start = value,
            widget::range_slider::Edge::End => *end = value,
        }
    }

    widget::Image::new(image_id)
        .w_h(512 as f64, 512 as f64)
        .middle()
        .set(ids.rendered_tex, ui);

    let range_slider_w = ui.w_of(ids.range_slider).unwrap();
    let w = (*end - *start) * range_slider_w;
    let h = 200.0;
    widget::Oval::fill([w, h])
        .mid_left_with_margin_on(ids.canvas, PAD + *start * range_slider_w)
        .color(color::LIGHT_BLUE)
        .down(50.0)
        .set(ids.oval, ui);
}