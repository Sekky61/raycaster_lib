mod camera;
mod ray;
pub mod render;
pub mod volumetric;

pub use camera::{Camera, TargetCamera};
pub use render::{RenderOptions, Renderer};
pub use volumetric::vol_reader;
pub use volumetric::EmptyIndexes;

use crate::volumetric::LinearVolume;

pub fn render_frame(width: usize, height: usize) -> Vec<u8> {
    let camera = TargetCamera::new(width, height);
    let read_result = vol_reader::from_file("volumes/Skull.vol");

    let volume_b = match read_result {
        Ok(vol) => vol,
        Err(message) => {
            eprint!("{}", message);
            std::process::exit(1);
        }
    };

    let volume = volume_b.build();

    let mut renderer = Renderer::<LinearVolume, TargetCamera>::new(volume, camera);
    renderer.set_render_options(RenderOptions {
        ray_termination: true,
        empty_index: false,
        multi_thread: false,
    });

    let mut buffer = vec![0; 3 * width * height];

    renderer.render_to_buffer(buffer.as_mut_slice());

    buffer
}
