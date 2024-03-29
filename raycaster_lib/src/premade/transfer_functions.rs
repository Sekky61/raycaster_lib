/*
    raycaster_lib
    Author: Michal Majer
    Date: 2022-05-05
*/

use crate::color::{self, RGBA};

// R G B A -- A <0;1>
// Skull has u8 samples (0;255)
#[allow(dead_code)]
pub fn skull_tf(sample: f32) -> RGBA {
    if sample > 40.0 && sample < 240.0 {
        RGBA::new(227.0, 218.0, 201.0, 1.0)
    } else {
        color::zero()
    }
}

// R G B A -- A <0;1>
#[allow(dead_code)]
pub fn foot_tf(sample: f32) -> RGBA {
    if sample > 40.0 && sample <= 105.0 {
        RGBA::new(252.0, 139.0, 101.0, 0.1)
    } else if sample > 130.0 {
        RGBA::new(250.0, 250.0, 250.0, 0.96)
    } else {
        color::zero()
    }
}

// // R G B A -- A <0;1>
// // Skull has u8 samples (0;255)
#[allow(dead_code)]
pub fn shapes_tf(sample: f32) -> RGBA {
    // relevant data between 90 and 110
    if sample > 85.0 && sample <= 95.0 {
        RGBA::new(255.0, 30.0, 60.0, 0.02)
    } else if sample > 95.0 && sample <= 100.0 {
        RGBA::new(10.0, 60.0, 180.0, 0.3)
    } else if sample > 100.0 && sample < 115.0 {
        RGBA::new(90.0, 210.0, 20.0, 0.6)
    } else {
        color::zero()
    }
}

// R G B A -- A <0;1>
// Skull has u8 samples (0;255)
#[allow(dead_code)]
pub fn white_tf(sample: f32) -> RGBA {
    if sample > 10.0 {
        RGBA::new(255.0, 255.0, 255.0, 0.3)
    } else {
        color::zero()
    }
}

// R G B A -- A <0;1>
#[allow(dead_code)]
pub fn anything_tf(sample: f32) -> RGBA {
    if sample > 0.0 {
        RGBA::new(255.0, 255.0, 255.0, 1.0)
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
