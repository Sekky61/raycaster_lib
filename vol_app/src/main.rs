mod gui;

use std::time::Instant;

use gui::{Gui, WIN_H, WIN_W};
use nalgebra::{point, vector, Vector3};
use sdl2::{event::Event, keyboard::Keycode, rect::Rect};

use raycaster_lib::{
    camera::PerspectiveCamera,
    premade::{parse::skull_parser, transfer_functions::skull_tf},
    render::{RenderOptions, Renderer},
    volumetric::{BuildVolume, LinearVolume, StreamVolume},
};

const RENDER_WIDTH_U: usize = 700;
const RENDER_HEIGHT_U: usize = 700;

const RENDER_WIDTH: u32 = RENDER_WIDTH_U as u32;
const RENDER_HEIGHT: u32 = RENDER_HEIGHT_U as u32;

fn main() -> Result<(), String> {
    // create SDL
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Vol app", WIN_W, WIN_H)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();

    // Create texture to render into
    let mut ren_tex = texture_creator
        .create_texture(
            sdl2::pixels::PixelFormatEnum::RGB24,
            sdl2::render::TextureAccess::Streaming,
            RENDER_WIDTH,
            RENDER_HEIGHT,
        )
        .expect("Couldn't make render texture");

    // Buffer to render into
    let mut buf_vec = create_rendering_buffer(RENDER_WIDTH_U, RENDER_HEIGHT_U);

    // Create GUI

    let mut gui = Gui::new();
    gui.build_gui();

    let volume: LinearVolume =
        raycaster_lib::volumetric::from_file("volumes/Skull.vol", skull_parser, skull_tf).unwrap();

    let pos = point![300.0, 300.0, 300.0];
    let dir = vector![-1.0, -1.0, -1.0];
    let camera = PerspectiveCamera::new(pos, dir);

    let mut raycast_renderer = Renderer::<_, _>::new(volume, camera);

    raycast_renderer.set_render_options(RenderOptions {
        resolution: (RENDER_WIDTH_U, RENDER_HEIGHT_U),
        ray_termination: true,
        empty_index: true,
        multi_thread: false,
    });

    // Main loop

    let mut state = State {
        left_mouse_held: false,
        right_mouse_held: false,
    };

    let mut event_pump = sdl_context.event_pump()?;
    let mut start_time = Instant::now(); // todo move to state

    'running: loop {
        // Handle events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
            // Camera control
            state.get_user_input(&mut raycast_renderer.camera, &event);

            // GUI
            match event {
                Event::MouseMotion { .. } => {} // Temporary bypass, performance
                _ => {
                    let _event_result = gui.handle_event(event);
                }
            }
        }

        // Frame time counter

        let duration = start_time.elapsed();
        start_time = Instant::now();

        // Update gui
        // TODO events?

        // let new_cam_pos = raycast_renderer.camera.get_position();
        // gui.send_cam_pos(new_cam_pos);
        gui.send_frame_time(duration);
        // gui.send_spherical_pos(raycast_renderer.camera.get_spherical());

        // Render frame, update texture and copy to canvas
        raycast_renderer.render_to_buffer(buf_vec.as_mut_slice());

        ren_tex
            .update(
                Rect::new(0, 0, RENDER_WIDTH, RENDER_HEIGHT),
                buf_vec.as_slice(),
                3 * RENDER_WIDTH_U,
            )
            .expect("Couldn't copy framebuffer to texture");

        canvas.copy(
            &ren_tex,
            None,
            Some(Rect::new(270, 10, RENDER_WIDTH, RENDER_HEIGHT)),
        )?;

        // Draw GUI
        gui.cache.draw_loop(&mut canvas);

        canvas.present();

        println!("Draw {:?}", duration);
    }

    Ok(())
}

fn create_rendering_buffer(width: usize, height: usize) -> Vec<u8> {
    vec![0; 3 * width * height]
}

// todo bitfield?
pub enum MouseButtonHeld {
    None,
    Left,
    Right,
    Both,
}

pub struct State {
    pub left_mouse_held: bool,
    pub right_mouse_held: bool,
}

impl State {
    fn get_user_input(&mut self, cam: &mut PerspectiveCamera, event: &sdl2::event::Event) {
        match event {
            Event::MouseMotion { xrel, yrel, .. } => {
                // When mouse button is down, drag camera around

                match (self.left_mouse_held, self.right_mouse_held) {
                    (false, false) => (),
                    (true, false) => {
                        // move on the plane described by camera position and normal
                        let drag_diff = (*xrel as f32, *yrel as f32);
                        cam.change_pos_plane(-drag_diff.0 * 1.2, -drag_diff.1 * 1.2);
                    }
                    (false, true) => {
                        // change camera direction
                        let drag_diff = (*xrel as f32, *yrel as f32);
                        cam.look_around(drag_diff.0 * -0.01, drag_diff.1 * -0.01);
                    }
                    (true, true) => {
                        // rotate around origin
                        let drag_diff = (*xrel as f32, *yrel as f32);
                        let axisangle = Vector3::y() * (std::f32::consts::FRAC_PI_8 * drag_diff.0);
                        let rot = nalgebra::Rotation3::new(axisangle);

                        cam.change_pos_matrix(rot);
                    }
                }
            }
            Event::MouseButtonDown { mouse_btn, .. } => match mouse_btn {
                sdl2::mouse::MouseButton::Left => self.left_mouse_held = true,
                sdl2::mouse::MouseButton::Right => self.right_mouse_held = true,
                _ => (),
            },
            Event::MouseButtonUp { mouse_btn, .. } => match mouse_btn {
                sdl2::mouse::MouseButton::Left => self.left_mouse_held = false,
                sdl2::mouse::MouseButton::Right => self.right_mouse_held = false,
                _ => (),
            },
            Event::MouseWheel { y, .. } => {
                // y        ... vertical scroll
                // +1 unit  ... 1 step of wheel down (negative -> scroll up)

                cam.change_pos_view_dir((*y as f32) * 5.0);
            }
            _ => {}
        }
    }
}
