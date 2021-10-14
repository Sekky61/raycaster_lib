mod camera;
mod ray;
pub mod render;
pub mod volumetric;

pub use camera::Camera;
pub use volumetric::vol_reader;

use crate::{render::Renderer, volumetric::LinearVolume};

pub use render::{MULTI_THREAD, SINGLE_THREAD};

pub fn render_frame(width: usize, height: usize) -> Vec<u8> {
    let camera = Camera::new(width, height);
    let read_result = vol_reader::from_file("Skull.vol");

    let volume_b = match read_result {
        Ok(vol) => vol,
        Err(message) => {
            eprint!("{}", message);
            std::process::exit(1);
        }
    };

    let volume = volume_b.build();

    let renderer = Renderer::<LinearVolume, SINGLE_THREAD>::new(volume, camera);

    let mut buffer: Vec<u8> = vec![0; width * height * 3];

    renderer.render(&mut buffer);

    buffer
}

pub fn render_to_byte_buffer(renderer: &Renderer<LinearVolume, SINGLE_THREAD>, buffer: &mut [u8]) {
    renderer.render(buffer);
}
