use byteorder::{BigEndian, ByteOrder};

use crate::{
    config::Config,
    sample_order::{HeaderFormat, SampleOrder},
};

pub fn generate_header(cfg: Config) -> Vec<u8> {
    match cfg.header_format {
        HeaderFormat::Default => generate_default_header(cfg),
    }
}

const DEFAULT_HEADER_LEN: usize = 3 * 4 + 1 + 3 * 4 + 2;
const DEFAULT_HEADER_LINEAR_1: u8 = 1;
const DEFAULT_HEADER_LINEAR_2: u8 = 0;
const DEFAULT_HEADER_Z_1: u8 = 2;

/// Default header
/// big-endian
/// 1. resolution -- 3x 32bit ints (x,y,z)
/// 2. 1 byte padding
/// 3. cell shape -- 3x 32bit floats
/// 4. sample_order -- 1x 16bit -- first byte sample_order, second byte parameter to the sample_order
/// 5. data -- x*y*z 8bit values; order depending on sample_order (4)
fn generate_default_header(cfg: Config) -> Vec<u8> {
    let mut vec = Vec::with_capacity(DEFAULT_HEADER_LEN);
    let mut slice = &mut vec[..];

    BigEndian::write_u32(&mut slice[0..4], cfg.dims.x);
    BigEndian::write_u32(&mut slice[4..8], cfg.dims.y);
    BigEndian::write_u32(&mut slice[8..12], cfg.dims.z);

    BigEndian::write_f32(&mut slice[13..17], cfg.cell_shape.x);
    BigEndian::write_f32(&mut slice[17..21], cfg.cell_shape.y);
    BigEndian::write_f32(&mut slice[21..25], cfg.cell_shape.z);

    match cfg.save_buffer_order {
        SampleOrder::Linear => {
            slice[25] = DEFAULT_HEADER_LINEAR_1;
            slice[26] = DEFAULT_HEADER_LINEAR_2
        }
        SampleOrder::Z(s) => {
            slice[25] = DEFAULT_HEADER_Z_1;
            slice[26] = s
        }
    }

    vec
}
