use pushrod::{
    base_widget::BaseWidget,
    box_widget::BoxWidget,
    engine::Engine,
    geometry::{Point, Size},
    widget::{SystemWidget, Widget},
};
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::{event::Event, pixels::Color};

use raycaster_lib::{volumetric::BlockVolume, Camera, RenderOptions, Renderer, TargetCamera};

const WIN_W: u32 = 1280;
const WIN_H: u32 = 720;

const WIDTH: usize = 512;
const HEIGHT: usize = 512;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Vol app", WIN_W, WIN_H)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut engine = Engine::new(Size::new(WIN_W, WIN_H), 60);

    let mut base_widget = BaseWidget::new(Point::new(0, 0), Size::new(WIN_W as u32, WIN_H as u32));
    base_widget.set_color(Color::RGBA(0, 0, 0, 255));
    let base_widget_id = engine.add_widget(SystemWidget::Base(Box::new(base_widget)));

    let mut box_widget1 = BoxWidget::new(Point::new(10, 20), Size::new(50, 50), Color::BLUE, 3);
    box_widget1.set_color(Color::CYAN);
    let box_widget_id1 = engine.add_widget(SystemWidget::Box(Box::new(box_widget1)));

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();

    let volume = raycaster_lib::vol_reader::from_file("volumes/Skull.vol")
        .expect("bad read of file")
        .build();

    let camera = TargetCamera::new(WIDTH, HEIGHT);

    let mut raycast_renderer = Renderer::<BlockVolume, _>::new(volume, camera);

    raycast_renderer.set_render_options(RenderOptions {
        ray_termination: true,
        empty_index: true,
        multi_thread: false,
    });

    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, WIDTH as u32, HEIGHT as u32)
        .map_err(|e| e.to_string())?;

    let mut buf_vec = vec![0; 3 * 512 * 512];
    raycast_renderer.render_to_buffer(buf_vec.as_mut_slice());

    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
            raycast_renderer.camera.get_user_input(&event);

            // GUI
            let event_result = engine.widget_cache.handle_event(event);

            if let Some(handler) = &engine.event_handler {
                // Needs to support handling of multiple events being generated
                // here.

                if !event_result.is_empty() {
                    handler.process_event(event_result);
                }
            }
        }
        // The rest of the game loop goes here...
        println!("Game loop");

        let mut buf_vec = vec![0; 3 * 512 * 512];
        raycast_renderer.render_to_buffer(buf_vec.as_mut_slice());

        // Create a red-green gradient
        texture.with_lock(None, |buffer: &mut [u8], active_frame: usize| {
            buffer[..(512 * 512 * 3)].clone_from_slice(&buf_vec[..(512 * 512 * 3)]);
        })?;

        println!("About to clear");

        // If canvas is clearing, you have to invalidate gui
        //canvas.clear();

        canvas.copy(&texture, None, Some(Rect::new(200, 100, 512, 512)))?;

        engine.widget_cache.draw_loop(&mut canvas);

        canvas.present();
    }

    Ok(())
}
