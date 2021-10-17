mod camera;
mod ray;
pub mod render;
pub mod volumetric;

pub use camera::Camera;
pub use render::Renderer;
pub use volumetric::vol_reader;

use crate::volumetric::LinearVolume;

pub use render::{MULTI_THREAD, SINGLE_THREAD};

use render::Render;

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

    let mut renderer = Renderer::<LinearVolume, SINGLE_THREAD>::new(volume, camera);

    renderer.render();

    renderer.get_buffer()
}
