use std::ops::Range;

use nalgebra::{vector, Vector2};

// A 2D range, rectangle described by two points
// Expected values in range <0;1>
#[derive(Clone, Copy, Default, Debug)]
pub struct ViewportBox {
    pub lower: Vector2<f32>,
    pub upper: Vector2<f32>,
}

impl ViewportBox {
    pub fn from_points(lower: Vector2<f32>, upper: Vector2<f32>) -> Self {
        Self { lower, upper }
    }

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

        let res_f = resolution.map(|v| v as f32);

        // Converting to integer rounds down
        let low_pixel = (self.lower.component_mul(&res_f)).map(|v| v as usize);
        let high_pixel = (self.upper.component_mul(&res_f)).map(|v| v as usize);

        PixelBox::new(low_pixel.x..high_pixel.x, low_pixel.y..high_pixel.y)
    }

    // True if rectangles share any area (in other words, if bounds cross)
    // Touching boundboxes do not cross
    pub fn crosses(&self, other: &ViewportBox) -> bool {
        let outside = self.upper.x <= other.lower.x
            || self.lower.x >= other.upper.x
            || self.upper.y <= other.lower.y
            || self.lower.y >= other.upper.y;
        !outside
    }

    // Touching boundboxes do not intersect
    pub fn intersection(&self, other: &ViewportBox) -> Option<ViewportBox> {
        let result = self.intersection_unchecked(other);
        if result.lower.x >= result.upper.x || result.lower.y >= result.upper.y {
            None
        } else {
            Some(result)
        }
    }

    //
    pub fn intersection_unchecked(&self, other: &ViewportBox) -> ViewportBox {
        let lower = vector![
            f32::max(self.lower.x, other.lower.x),
            f32::max(self.lower.y, other.lower.y)
        ];
        let upper = vector![
            f32::min(self.upper.x, other.upper.x),
            f32::min(self.upper.y, other.upper.y)
        ];

        ViewportBox { lower, upper }
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

    #[test]
    fn touching_dont_cross() {
        let a = ViewportBox {
            lower: vector![0.0, 0.0],
            upper: vector![1.0, 1.0],
        };
        let b = ViewportBox {
            lower: vector![1.0, 0.0],
            upper: vector![2.0, 1.0],
        };

        let c = a.intersection(&b);

        assert!(!a.crosses(&b));
        assert!(c.is_none());
    }
}
