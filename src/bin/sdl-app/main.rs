use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;

use nalgebra::vector;
use raycaster_lib::{
    render::BufferStatus,
    volumetric::{BlockVolume, LinearVolume},
    RenderOptions, Renderer, TargetCamera,
};

const WIDTH: usize = 512;
const HEIGHT: usize = 512;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("rust-sdl2 demo: Video", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();

    let volume = raycaster_lib::vol_reader::from_file("Skull.vol")
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
                Event::MouseButtonDown { x, y, .. } => {
                    println!("mouse btn down at ({},{})", x, y);
                }
                _ => {}
            }
        }
        // The rest of the game loop goes here...
        println!("Game loop");

        let mut buf_vec = vec![0; 3 * 512 * 512];
        raycast_renderer.render_to_buffer(buf_vec.as_mut_slice());
        raycast_renderer
            .camera
            .change_pos(vector![20.0, 20.0, 20.0]);

        // Create a red-green gradient
        texture.with_lock(None, |buffer: &mut [u8], active_frame: usize| {
            buffer[..(512 * 512 * 3)].clone_from_slice(&buf_vec[..(512 * 512 * 3)]);
        })?;

        println!("About to clear");

        canvas.clear();
        canvas.copy(&texture, None, Some(Rect::new(50, 50, 512, 512)))?;
        canvas.present();
    }

    Ok(())
}
