use std::{cmp::min, ops::Range};

use nalgebra::{point, vector, Point2, Vector2};

// A 2D range, rectangle described by two points
// Expected values in range <0;1>
#[derive(Clone, Copy, Default)]
pub struct ViewportBox {
    pub lower: Vector2<f32>,
    pub upper: Vector2<f32>,
}

impl ViewportBox {
    // Maximum viewport, flipped
    pub fn new() -> Self {
        Self {
            lower: vector![f32::INFINITY, f32::INFINITY],
            upper: vector![f32::NEG_INFINITY, f32::NEG_INFINITY],
        }
    }

    // todo params as vector
    pub fn add_point(&mut self, x: f32, y: f32) {
        self.upper.x = f32::max(self.upper.x, x);
        self.upper.y = f32::max(self.upper.y, y);
        self.lower.x = f32::min(self.lower.x, x);
        self.lower.y = f32::min(self.lower.y, y);
    }

    pub fn size(&self) -> Vector2<f32> {
        self.upper - self.lower
    }

    pub fn get_pixel_range(&self, resolution: Vector2<usize>) -> PixelBox {
        // Approach: floor values down to nearest pixel
        // Two adjacent boxes can share one line of pixels

        let (width, height) = (resolution.x, resolution.y);
        let res_f = resolution.map(|v| v as f32);

        let low_pixel = (self.lower.component_mul(&res_f)).map(|v| v as usize);
        let high_pixel = (self.upper.component_mul(&res_f)).map(|v| v as usize);

        PixelBox::new(low_pixel.x..high_pixel.x, low_pixel.y..high_pixel.y)
    }

    // True if rectangles share any area (in other words, if bounds cross)
    pub fn crosses(&self, other: &ViewportBox) -> bool {
        let outside = self.upper.x < other.lower.x
            || self.lower.x > other.upper.x
            || self.upper.y < other.lower.y
            || self.lower.y > other.upper.y;
        !outside
    }

    pub fn intersection(&self, other: &ViewportBox) -> ViewportBox {
        todo!()
    }
}

pub struct PixelBox {
    pub x: Range<usize>,
    pub y: Range<usize>,
}

impl PixelBox {
    pub fn new(x: Range<usize>, y: Range<usize>) -> Self {
        Self { x, y }
    }

    pub fn items(&self) -> usize {
        (self.x.end - self.x.start) * (self.y.end - self.y.start)
    }
}

#[cfg(test)]
mod test {

    use nalgebra::vector;

    use super::*;

    fn from_slice(slice: &[(f32, f32)]) -> ViewportBox {
        let mut vp = ViewportBox::new();
        for (x, y) in slice {
            vp.add_point(*x, *y);
        }
        vp
    }

    #[test]
    fn viewport() {
        let mut vp = ViewportBox::new();

        vp.add_point(0.5, 0.5);

        assert_eq!(vp.lower, vector![0.5, 0.5]);
        assert_eq!(vp.upper, vector![0.5, 0.5]);

        vp.add_point(0.6, 0.6);

        assert_eq!(vp.lower, vector![0.5, 0.5]);
        assert_eq!(vp.upper, vector![0.6, 0.6]);

        vp.add_point(0.5, 0.7);

        assert_eq!(vp.lower, vector![0.5, 0.5]);
        assert_eq!(vp.upper, vector![0.6, 0.7]);

        vp.add_point(0.5, 0.4);

        assert_eq!(vp.lower, vector![0.5, 0.4]);
        assert_eq!(vp.upper, vector![0.6, 0.7]);

        vp.add_point(0.3, 0.2);

        assert_eq!(vp.lower, vector![0.3, 0.2]);
        assert_eq!(vp.upper, vector![0.6, 0.7]);

        vp.add_point(0.2, 0.8);

        assert_eq!(vp.lower, vector![0.2, 0.2]);
        assert_eq!(vp.upper, vector![0.6, 0.8]);
    }

    #[test]
    fn cross() {
        let a = ViewportBox {
            lower: vector![0.0, 0.0],
            upper: vector![1.0, 1.0],
        };

        let b = ViewportBox {
            lower: vector![0.5, 0.5],
            upper: vector![1.5, 1.5],
        };

        let c = ViewportBox {
            lower: vector![1.1, 1.1],
            upper: vector![2.0, 1.4],
        };

        let d = ViewportBox {
            lower: vector![0.3, -0.1],
            upper: vector![0.6, 0.3],
        };

        let e = ViewportBox {
            lower: vector![-0.6, 0.1],
            upper: vector![0.3, 0.3],
        };

        assert!(a.crosses(&b));
        assert!(!a.crosses(&c));
        assert!(b.crosses(&c));

        assert!(a.crosses(&d));
        assert!(!b.crosses(&d));
        assert!(!c.crosses(&d));

        assert!(a.crosses(&e));
    }
}
