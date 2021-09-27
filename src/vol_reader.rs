use std::convert::TryInto;
use std::fmt::Display;
use std::path::Path;
use std::{fs::File, io::Read};

use nom::bytes::complete::take;
use nom::IResult;

#[derive(Debug)]
pub struct RGBColor(u8, u8, u8);

impl RGBColor {
    pub fn from_char(val: u8) -> RGBColor {
        RGBColor(val, val, val)
    }

    pub fn from_vals(r: u8, g: u8, b: u8) -> RGBColor {
        RGBColor(r, g, b)
    }

    pub fn to_int(&self) -> u32 {
        let r = self.0 as u32;
        let g = self.1 as u32;
        let b = self.2 as u32;

        (r << 16) + (g << 8) + b
    }
}

pub struct Frame {
    width: usize,
    height: usize,
    data: Vec<u8>,
}

impl Frame {
    pub fn from_data(width: usize, height: usize, data: &[u8]) -> Frame {
        Frame {
            width,
            height,
            data: data.to_owned(),
        }
    }

    pub fn get_data(&self, x: usize, y: usize) -> Option<u8> {
        let start = y * self.height + x;
        self.data.get(start).copied()
    }
}

#[derive(Default)]
pub struct Volume {
    pub x: usize,
    pub y: usize,
    pub z: usize,
    border: u32,
    scale_x: f32,
    scale_y: f32,
    scale_z: f32,
    frames: Vec<Frame>,
}

impl Volume {
    pub fn from_file<P>(path: P) -> Volume
    where
        P: AsRef<Path>,
    {
        let mut fb = Vec::with_capacity(5_000_000);
        File::open(path)
            .expect("no file skull")
            .read_to_end(&mut fb)
            .expect("cannot read");

        // assert_eq!(parser("(abc)"), Ok(("", "abc")));

        let x_bytes: [u8; 4] = fb.get(0..4).expect("no bytes x").try_into().expect("wrong");
        let x = u32::from_be_bytes(x_bytes);

        let y_bytes: [u8; 4] = fb.get(4..8).expect("no bytes y").try_into().expect("wrong");
        let y = u32::from_be_bytes(y_bytes);

        let z_bytes: [u8; 4] = fb
            .get(8..12)
            .expect("no bytes z")
            .try_into()
            .expect("wrong");
        let z = u32::from_be_bytes(z_bytes);

        let xs_bytes: [u8; 4] = fb
            .get(16..20)
            .expect("no bytes x scale")
            .try_into()
            .expect("wrong");
        let scale_x = f32::from_be_bytes(xs_bytes);

        let ys_bytes: [u8; 4] = fb
            .get(20..24)
            .expect("no bytes y scale")
            .try_into()
            .expect("wrong");
        let scale_y = f32::from_be_bytes(ys_bytes);

        let zs_bytes: [u8; 4] = fb
            .get(24..28)
            .expect("no bytes z scale")
            .try_into()
            .expect("wrong");
        let scale_z = f32::from_be_bytes(zs_bytes);

        let rest = &fb[28..];

        println!(
            "Rest: {} | Rest / 68 = {} | Rest / 256*256 = {}",
            rest.len(),
            rest.len() / 68,
            rest.len() / (256 * 256)
        );

        let mut frames_iter = rest.chunks((y * z) as usize);

        println!(
            "Chunks: {} | Chunk size = {}",
            frames_iter.len(),
            frames_iter.next().expect("ddd").len(),
        );

        let mut frames = Vec::new();

        println!("{}", frames_iter.len());

        for frame_data in frames_iter {
            let frame = Frame::from_data(y as usize, z as usize, frame_data);
            frames.push(frame);
        }

        let x = x as usize;
        let y = y as usize;
        let z = z as usize;

        Volume {
            x,
            y,
            z,
            border: 0,
            scale_x,
            scale_y,
            scale_z,
            frames,
        }
    }

    pub fn get_3d(&self, x: usize, y: usize, z: usize) -> RGBColor {
        let plane = self.frames.get(x).expect("out of range frame");
        let val = plane.get_data(y, z);
        match val {
            Some(v) => RGBColor::from_char(v),
            None => panic!("bad read"),
        }
    }
}

impl Display for Volume {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(
            fmt,
            "coords {} {} {} | scale {} {} {} | data count {}",
            self.x,
            self.y,
            self.z,
            self.scale_x,
            self.scale_y,
            self.scale_z,
            self.frames.len()
        )
    }
}
