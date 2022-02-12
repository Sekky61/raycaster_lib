use crate::color::{self, RGBA};

// R G B A -- A <0;1>
#[allow(dead_code)]
pub fn skull_tf(sample: u8) -> RGBA {
    if sample > 170 {
        RGBA::new(220.0, 0.0, 20.0, 0.1)
    } else if sample > 130 {
        RGBA::new(0.0, 220.0, 0.0, 0.04)
    } else {
        color::zero()
    }
}

// R G B A -- A <0;1>
#[allow(dead_code)]
pub fn c60large_tf(sample: u8) -> RGBA {
    if sample > 230 && sample < 255 {
        RGBA::new(200.0, 0.0, 0.0, 0.5)
    } else if sample > 200 && sample < 230 {
        RGBA::new(0.0, 180.0, 0.0, 0.3)
    } else if sample > 80 && sample < 120 {
        RGBA::new(2.0, 2.0, 60.0, 0.02)
    } else {
        color::zero()
    }
}

// R G B A -- A <0;1>
// uses just 12 bits
#[allow(dead_code)]
pub fn beetle_tf(sample: u16) -> RGBA {
    if sample > 10000 {
        RGBA::new(255.0, 0.0, 0.0, 0.01)
    } else if sample > 5000 {
        RGBA::new(0.0, 255.0, 0.0, 0.01)
    } else if sample > 1900 {
        RGBA::new(0.0, 0.0, 255.0, 0.01)
    } else if sample > 800 {
        RGBA::new(10.0, 10.0, 10.0, 0.01)
    } else {
        color::zero()
    }
}
