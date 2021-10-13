pub mod camera;

pub mod volume;
use volume::vol_reader;
use volume::Volume;

pub mod renderer;

use camera::{BoundBox, Camera};

use crate::volume::LinearVolume;

pub fn render_frame(width: usize, height: usize) -> Vec<u8> {
    let camera = Camera::new(width, height);
    let read_result = vol_reader::from_file("Skull.vol");
    //let volume = Volume::white_vol();

    let volume_b = match read_result {
        Ok(vol) => vol,
        Err(message) => {
            eprint!("{}", message);
            std::process::exit(1);
        }
    };

    let volume = LinearVolume::from(volume_b);

    let bbox = BoundBox::from_volume(volume);

    let mut buffer: Vec<u8> = vec![0; width * height * 3];

    render_to_byte_buffer(&camera, &bbox, &mut buffer);

    buffer
}

pub fn render_to_byte_buffer<V>(camera: &Camera, bbox: &BoundBox<V>, buffer: &mut [u8])
where
    V: Volume,
{
    camera.cast_rays_bytes(bbox, buffer);
}
