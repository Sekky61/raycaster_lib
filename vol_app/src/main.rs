use std::time::Instant;

use pushrod::{
    base_widget::BaseWidget,
    box_widget::BoxWidget,
    engine::Engine,
    geometry::{Point, Size},
    text_widget::{TextAlignment, TextWidget},
    widget::{SystemWidget, Widget},
};
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::{event::Event, pixels::Color};

use raycaster_lib::{
    vol_reader, volumetric::BlockVolume, Camera, RenderOptions, Renderer, TargetCamera,
};

const WIN_W: u32 = 980;
const WIN_H: u32 = 720;

const RENDER_WIDTH_U: usize = 700;
const RENDER_HEIGHT_U: usize = 700;

const RENDER_WIDTH: u32 = RENDER_WIDTH_U as u32;
const RENDER_HEIGHT: u32 = RENDER_HEIGHT_U as u32;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("Vol app", WIN_W, WIN_H)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    // frame_rate has no effect now
    let mut engine = Engine::new(Size::new(WIN_W, WIN_H), 60);

    let ms_counter_widget_id = build_gui(&mut engine);

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();

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

    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, RENDER_WIDTH, RENDER_HEIGHT)
        .map_err(|e| e.to_string())?;

    let mut ren_tex = texture_creator
        .create_texture(
            sdl2::pixels::PixelFormatEnum::RGB24,
            sdl2::render::TextureAccess::Streaming,
            RENDER_WIDTH,
            RENDER_HEIGHT,
        )
        .expect("Couldn't make render texture");

    let mut buf_vec = create_rendering_buffer(RENDER_WIDTH_U, RENDER_HEIGHT_U);

    let mut event_pump = sdl_context.event_pump()?;

    let mut start_time = Instant::now();

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
            raycast_renderer.camera.get_user_input(&event);

            // GUI
            // Temporary bypass, performance
            match event {
                Event::MouseMotion { .. } => {}
                _ => {
                    let event_result = engine.widget_cache.handle_event(event);

                    if let Some(handler) = &engine.event_handler {
                        // Needs to support handling of multiple events being generated
                        // here.

                        if !event_result.is_empty() {
                            handler.process_event(event_result);
                        }
                    }
                }
            }
        }

        raycast_renderer.render_to_buffer(buf_vec.as_mut_slice());

        // Update texture

        ren_tex
            .update(
                Rect::new(0, 0, RENDER_WIDTH, RENDER_HEIGHT),
                buf_vec.as_slice(),
                3 * RENDER_WIDTH_U,
            )
            .expect("Couldn't copy framebuffer to texture");

        // Copy render result to frame buffer (draw to screen)

        canvas.copy(
            &ren_tex,
            None,
            Some(Rect::new(270, 10, RENDER_WIDTH, RENDER_HEIGHT)),
        )?;

        // Draw GUI
        engine.widget_cache.draw_loop(&mut canvas);

        canvas.present();

        let duration = start_time.elapsed();
        start_time = Instant::now();

        // TODO events?
        if let Some(widget) = engine.widget_cache.get_mut(ms_counter_widget_id) {
            if let SystemWidget::Text(ms_counter) = widget {
                let ms_text = duration.as_millis().to_string();
                ms_counter.set_text(ms_text.as_str());
            }
        }

        println!("Draw {:?}", duration);
    }

    Ok(())
}

fn create_rendering_buffer(width: usize, height: usize) -> Vec<u8> {
    vec![0; 3 * width * height]
}

fn build_gui(engine: &mut Engine) -> i32 {
    let mut base_widget = BaseWidget::new(Point::new(0, 0), Size::new(WIN_W as u32, WIN_H as u32));
    base_widget.set_color(Color::RGBA(20, 20, 20, 255));
    let _base_widget_id = engine.add_widget(SystemWidget::Base(Box::new(base_widget)));

    let mut box_widget1 = BoxWidget::new(Point::new(10, 10), Size::new(250, 700), Color::BLUE, 3);
    box_widget1.set_color(Color::CYAN);
    let _box_widget_id1 = engine.add_widget(SystemWidget::Box(Box::new(box_widget1)));

    let mut ms_counter = TextWidget::new(
        Point::new(20, 20),
        Size::new(200, 50),
        "def ms".into(),
        TextAlignment::AlignLeft,
    );
    ms_counter.set_invalidated(true);
    let _ms_widget_id1 = engine.add_widget(SystemWidget::Text(Box::new(ms_counter)));
    _ms_widget_id1
}
