use crate::color::{self, RGBA};

// R G B A -- A <0;1>
// Skull has u8 samples (0;255)
#[allow(dead_code)]
pub fn skull_tf(sample: f32) -> RGBA {
    if sample > 170.0 {
        //RGBA::new(220.0, 0.0, 20.0, 0.14)
        RGBA::new(220.0, 40.0, 60.0, 0.9)
    } else if sample > 5.0 {
        RGBA::new(40.0, 190.0, 5.0, 0.16)
    } else {
        color::zero()
    }
}

// R G B A -- A <0;1>
#[allow(dead_code)]
pub fn c60large_tf(sample: f32) -> RGBA {
    if sample > 230.0 && sample < 255.0 {
        RGBA::new(200.0, 0.0, 0.0, 0.5)
    } else if sample > 200.0 && sample < 230.0 {
        RGBA::new(0.0, 180.0, 0.0, 0.3)
    } else if sample > 80.0 && sample < 120.0 {
        RGBA::new(2.0, 2.0, 60.0, 0.02)
    } else {
        color::zero()
    }
}

// R G B A -- A <0;1>
// Values <0;4095>
// uses just 12 bits -- todo are upper bits zero?
#[allow(dead_code)]
pub fn beetle_tf(sample: f32) -> RGBA {
    if sample > 3000.0 {
        RGBA::new(255.0, 0.0, 0.0, 0.1)
    } else if sample > 2000.0 {
        RGBA::new(0.0, 255.0, 0.0, 0.1)
    } else if sample > 1500.0 {
        RGBA::new(0.0, 0.0, 255.0, 0.1)
    } else if sample > 800.0 {
        RGBA::new(10.0, 10.0, 10.0, 0.1)
    } else {
        color::zero()
    }
}