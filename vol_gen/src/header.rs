/*
    vol_gen
    Author: Michal Majer
    Date: 2022-05-05
*/

use byteorder::{ByteOrder, LittleEndian};

use crate::{config::Config, orders::SampleOrder};

// Describe header
#[derive(Debug)]
pub enum HeaderFormat {
    Default,
}

pub fn generate_header(cfg: &Config) -> Vec<u8> {
    match cfg.header_format {
        HeaderFormat::Default => generate_default_header(cfg),
    }
}
/// Length of default header
const DEFAULT_HEADER_LEN: usize = 3 * 4 + 3 * 4 + 2;
const DEFAULT_HEADER_LINEAR_1: u8 = 1;
const DEFAULT_HEADER_LINEAR_2: u8 = 0;
const DEFAULT_HEADER_Z_1: u8 = 2;

/// Default header
///
/// Generated into vector
///
/// # Header description
///
/// little-endian, total length 26B:
/// 1. resolution -- 3x 32bit ints (x,y,z)
/// 2. cell shape -- 3x 32bit floats
/// 3. sample_order -- 2x 8bit -- first byte sample_order, second byte parameter to the sample_order
/// 4. data -- x*y*z 8bit values; order depending on sample_order (4)
fn generate_default_header(cfg: &Config) -> Vec<u8> {
    let mut vec = vec![0; DEFAULT_HEADER_LEN];
    let slice = &mut vec[..];

    LittleEndian::write_u32(&mut slice[0..4], cfg.dims.x);
    LittleEndian::write_u32(&mut slice[4..8], cfg.dims.y);
    LittleEndian::write_u32(&mut slice[8..12], cfg.dims.z);
    LittleEndian::write_f32(&mut slice[12..16], cfg.cell_shape.x);
    LittleEndian::write_f32(&mut slice[16..20], cfg.cell_shape.y);
    LittleEndian::write_f32(&mut slice[20..24], cfg.cell_shape.z);

    match cfg.save_buffer_order {
        SampleOrder::Linear => {
            slice[24] = DEFAULT_HEADER_LINEAR_1;
            slice[25] = DEFAULT_HEADER_LINEAR_2
        }
        SampleOrder::Z(s) => {
            slice[24] = DEFAULT_HEADER_Z_1;
            slice[25] = s
        }
    }

    vec
}
