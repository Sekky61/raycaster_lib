pub mod block;
pub mod camera;
pub mod vol_reader;
pub mod volume;

use camera::{BoundBox, Camera};

pub fn render_frame(width: usize, height: usize) -> Vec<u8> {
    let camera = Camera::new(width, height);
    let read_result = vol_reader::from_file("Skull.vol");
    //let volume = Volume::white_vol();

    let volume = match read_result {
        Ok(vol) => vol,
        Err(message) => {
            eprint!("{}", message);
            std::process::exit(1);
        }
    };

    let bbox = BoundBox::from_volume(volume);

    let mut buffer: Vec<u8> = vec![0; width * height * 3];

    render_to_byte_buffer(&camera, &bbox, &mut buffer);

    buffer
}

pub fn render_to_byte_buffer(camera: &Camera, bbox: &BoundBox, buffer: &mut [u8]) {
    camera.cast_rays_bytes(bbox, buffer);
}
