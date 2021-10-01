use std::convert::TryInto;
use std::fmt::Display;
use std::path::Path;
use std::{fs::File, io::Read};

use nalgebra::{vector, Vector3};
use nom::bytes::complete::take;
use nom::IResult;

#[derive(Debug)]
pub struct RGBColor(pub u8, pub u8, pub u8);

impl RGBColor {
    pub fn from_char(val: u8) -> RGBColor {
        RGBColor(val, val, val)
    }

    pub fn from_vals(r: u8, g: u8, b: u8) -> RGBColor {
        RGBColor(r, g, b)
    }

    pub fn from_slice(slice: &[f32]) -> RGBColor {
        RGBColor(slice[0] as u8, slice[1] as u8, slice[2] as u8)
    }

    pub fn to_int(&self) -> u32 {
        let r = self.0 as u32;
        let g = self.1 as u32;
        let b = self.2 as u32;

        (r << 16) + (g << 8) + b
    }
}

#[derive(Debug)]
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

impl std::fmt::Debug for Volume {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Volume")
            .field("x", &self.x)
            .field("y", &self.y)
            .field("z", &self.z)
            .field("border", &self.border)
            .field("scale_x", &self.scale_x)
            .field("scale_y", &self.scale_y)
            .field("scale_z", &self.scale_z)
            .finish()
    }
}

impl Volume {
    pub fn white_vol() -> Volume {
        Volume {
            x: 2,
            y: 2,
            z: 2,
            border: 0,
            scale_x: 1.0,
            scale_y: 1.0,
            scale_z: 1.0,
            frames: vec![],
        }
    }

    pub fn get_dims(&self) -> Vector3<f32> {
        vector![
            self.x as f32 * self.scale_x,
            self.y as f32 * self.scale_y,
            self.z as f32 * self.scale_z
        ]
    }

    pub fn sample_at(&self, pos: Vector3<f32>) -> f32 {
        let lows: Vec<f32> = pos.as_slice().iter().map(|&f| f.floor()).collect();
        let x_low = lows[0] as usize;
        let y_low = lows[1] as usize;
        let z_low = lows[2] as usize;

        let x_t = pos.x - lows[0];
        let y_t = pos.y - lows[1];
        let z_t = pos.z - lows[2];

        let c000 = self.get_3d(x_low, y_low, z_low) as f32;
        let c001 = self.get_3d(x_low, y_low, z_low + 1) as f32;
        let c010 = self.get_3d(x_low, y_low + 1, z_low) as f32;
        let c011 = self.get_3d(x_low, y_low + 1, z_low + 1) as f32;
        let c100 = self.get_3d(x_low + 1, y_low, z_low) as f32;
        let c101 = self.get_3d(x_low + 1, y_low, z_low + 1) as f32;
        let c110 = self.get_3d(x_low + 1, y_low + 1, z_low) as f32;
        let c111 = self.get_3d(x_low + 1, y_low + 1, z_low + 1) as f32;

        let c00 = c000 * (1.0 - x_t) + c100 * x_t;
        let c01 = c001 * (1.0 - x_t) + c101 * x_t;
        let c10 = c010 * (1.0 - x_t) + c110 * x_t;
        let c11 = c011 * (1.0 - x_t) + c111 * x_t;

        let c0 = c00 * (1.0 - y_t) + c10 * y_t;
        let c1 = c01 * (1.0 - y_t) + c11 * y_t;

        let c = c0 * (1.0 - z_t) + c1 * z_t;

        c
    }

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

    pub fn get_3d(&self, x: usize, y: usize, z: usize) -> u8 {
        //println!("Getting {} {} {}", x, y, z);
        let plane = self.frames.get(x);
        let plane = match plane {
            Some(p) => p,
            None => return 0,
        }; //.expect("out of range frame");
        let val = plane.get_data(y, z);
        match val {
            Some(v) => v,
            None => 0,
        }
    }

    pub fn is_in(&self, pos: Vector3<f32>) -> bool {
        let x_f = self.x as f32;
        let y_f = self.y as f32;
        let z_f = self.z as f32;

        x_f > pos.x && y_f > pos.y && z_f > pos.z && pos.x > 0.0 && pos.y > 0.0 && pos.z > 0.0
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
