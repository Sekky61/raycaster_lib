mod gui;

use std::time::Instant;

use gui::{Gui, WIN_H, WIN_W};
use nalgebra::point;
use pushrod::widget::SystemWidget;
use sdl2::keyboard::Keycode;
use sdl2::{event::Event, rect::Rect};

use raycaster_lib::{
    vol_reader, volumetric::BlockVolume, Camera, RenderOptions, Renderer, TargetCamera,
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
    //let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string()).unwrap();

    let mut gui = Gui::new();
    gui.build_gui();

    // Build Renderer and Volume
    let volume = vol_reader::from_file("volumes/Skull.vol")
        .expect("bad read of file")
        .build();

    let camera = TargetCamera::new(RENDER_WIDTH as usize, RENDER_HEIGHT as usize);

    let mut raycast_renderer = Renderer::<BlockVolume, _>::new(volume, camera);

    raycast_renderer.set_render_options(RenderOptions {
        ray_termination: true,
        empty_index: true,
        multi_thread: false,
    });

    // Main loop

    let mut event_pump = sdl_context.event_pump()?;
    let mut start_time = Instant::now();

    let mut cam_pos = point![0.0, 0.0, 0.0];

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
            raycast_renderer.camera.get_user_input(&event);

            // GUI
            match event {
                Event::MouseMotion { .. } => {} // Temporary bypass, performance
                _ => {
                    let event_result = gui.handle_event(event);

                    if let Some(handler) = &gui.engine.event_handler {
                        // Needs to support handling of multiple events being generated
                        // here.

                        if !event_result.is_empty() {
                            handler.process_event(event_result);
                        }
                    }
                }
            }
        }

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
        gui.engine.widget_cache.draw_loop(&mut canvas);

        canvas.present();

        // Camera coords

        let new_cam_pos = raycast_renderer.camera.get_position();
        if cam_pos != new_cam_pos {
            cam_pos = new_cam_pos;
            if let Some(SystemWidget::Text(cam_pos_widget)) =
                gui.engine.widget_cache.get_mut(gui.cam_pos_id)
            {
                let coord_text = format!(
                    "[ {:>6.2} , {:>6.2} , {:>6.2} ]",
                    cam_pos.x, cam_pos.y, cam_pos.z
                );
                cam_pos_widget.set_text(coord_text.as_str());
            }
        }

        // Frame time counter

        let duration = start_time.elapsed();
        start_time = Instant::now();

        // TODO events?
        if let Some(SystemWidget::Text(ms_counter)) =
            gui.engine.widget_cache.get_mut(gui.ms_counter_id)
        {
            let ms_text = duration.as_millis().to_string();
            ms_counter.set_text(ms_text.as_str());
        }

        println!("Draw {:?}", duration);
    }

    drop(gui);

    Ok(())
}

fn create_rendering_buffer(width: usize, height: usize) -> Vec<u8> {
    vec![0; 3 * width * height]
}
