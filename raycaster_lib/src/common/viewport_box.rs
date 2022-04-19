use std::{
    cmp::{max, min},
    ops::Range,
};

use nalgebra::{vector, Vector2};

/// A 2D range; rectangle described by two points.
/// Though not enforced, expected values are in range <0;1>.
/// This is in alignment with main usecase - calculations in viewport.
#[derive(Clone, Copy, Default, Debug)]
pub struct ViewportBox {
    /// Lowest point of rectangle
    pub lower: Vector2<f32>,
    /// Highest point of rectangle
    pub upper: Vector2<f32>,
}

impl ViewportBox {
    /// Construct `ViewportBox` from its parts.
    pub fn from_points(lower: Vector2<f32>, upper: Vector2<f32>) -> Self {
        Self { lower, upper }
    }

    /// Returns empty box.
    pub fn new() -> Self {
        Self {
            lower: vector![f32::INFINITY, f32::INFINITY],
            upper: vector![f32::NEG_INFINITY, f32::NEG_INFINITY],
        }
    }

    /// Expand range to include point \[x,y\].
    pub fn add_point(&mut self, x: f32, y: f32) {
        // Todo possibly use vectors
        self.upper.x = f32::clamp(f32::max(self.upper.x, x), 0.0, 1.0);
        self.upper.y = f32::clamp(f32::max(self.upper.y, y), 0.0, 1.0);
        self.lower.x = f32::clamp(f32::min(self.lower.x, x), 0.0, 1.0);
        self.lower.y = f32::clamp(f32::min(self.lower.y, y), 0.0, 1.0);
    }

    /// Returns size of rectangle
    pub fn size(&self) -> Vector2<f32> {
        self.upper - self.lower
    }

    /// Convert `ViewportBox`, which is in normalized coordinates into
    /// pixel ranges.
    ///
    /// Parameter `resolution` is resolution of rectangle \[0,0\],\[1,1\], in other words
    /// the resolution of camera.
    pub fn get_pixel_range(&self, resolution: Vector2<u16>) -> PixelBox {
        // Floor values down to nearest pixel
        // todo might save time to pass resolution as f32

        let res_f = resolution.map(|v| v as f32);

        // Converting to integer rounds down
        let low_pixel = (self.lower.component_mul(&res_f)).map(|v| v as u16);
        let high_pixel = (self.upper.component_mul(&res_f)).map(|v| v as u16);

        PixelBox::new(low_pixel.x..high_pixel.x, low_pixel.y..high_pixel.y)
    }

    /// Checks if rectangles share any area (in other words, if bounds cross).
    /// Touching boundboxes do not cross.
    /// To get actual intersection area, see [ViewportBox::intersection].
    pub fn crosses(&self, other: &ViewportBox) -> bool {
        let outside = self.upper.x <= other.lower.x
            || self.lower.x >= other.upper.x
            || self.upper.y <= other.lower.y
            || self.lower.y >= other.upper.y;
        !outside
    }

    /// Returns result of intersection between two rectangles.
    /// Touching boundboxes do not intersect.
    pub fn intersection(&self, other: &ViewportBox) -> Option<ViewportBox> {
        let result = self.intersection_unchecked(other);
        if result.lower.x >= result.upper.x || result.lower.y >= result.upper.y {
            None
        } else {
            Some(result)
        }
    }

    /// Returns intersection of two 2D boxes
    /// If the boxes do not intersect, data is faulty.
    /// If you are not sure if boxes intersect, use safe variant [ViewportBox::intersection].
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

/// 2D range of pixels.
/// Usually product of calling [ViewportBox::get_pixel_range].
///
/// Example: if `x = 0..10`, the width is `10`, tenth pixel is index `[9]`.
#[derive(Clone, Debug)]
pub struct PixelBox {
    /// Pixel range on the `x` axis
    pub x: Range<u16>,
    /// Pixel range on the `y` axis
    pub y: Range<u16>,
}

// Implemented with open ended ranges
impl PixelBox {
    /// Construct new `PixelBox` from two ranges.
    pub fn new(x: Range<u16>, y: Range<u16>) -> Self {
        Self { x, y }
    }

    /// Returns width of the range
    pub fn width(&self) -> u16 {
        self.x.end - self.x.start
    }

    /// Returns height of the range
    pub fn height(&self) -> u16 {
        self.y.end - self.y.start
    }

    /// Returns number of pixels in range.
    pub fn items(&self) -> u32 {
        // Can be at most 32bit
        (self.width() as u32) * (self.height() as u32)
    }

    /// Checks if rectangles share any area (in other words, if bounds cross)
    pub fn crosses(&self, other: &PixelBox) -> bool {
        let outside = self.x.end <= other.x.start
            || self.x.start >= other.x.end
            || self.y.end <= other.y.start
            || self.y.start >= other.y.end;
        !outside
    }

    /// Returns intersection of two `PixelRange`s, if one exists.
    /// Touching boundboxes do not intersect.
    pub fn intersection(&self, other: &PixelBox) -> Option<PixelBox> {
        let result = self.intersection_unchecked(other);
        if result.x.start >= result.x.end || result.y.start >= result.y.end {
            None
        } else {
            Some(result)
        }
    }

    /// Unsafe variant of [PixelBox::intersection].
    pub fn intersection_unchecked(&self, other: &PixelBox) -> PixelBox {
        let lower_x = max(self.x.start, other.x.start);
        let lower_y = max(self.y.start, other.y.start);

        let upper_x = min(self.x.end, other.x.end);
        let upper_y = min(self.y.end, other.y.end);

        PixelBox {
            x: lower_x..upper_x,
            y: lower_y..upper_y,
        }
    }

    /// Returns 'linear' offset of smaller `PixelBox` inside a bigger one.
    /// Inputs are not checked.
    /// Usually called with bigger box being screen and therefore getting
    /// offset of pixelbox in framebuffer.
    pub fn offset_in_unchecked(&self, smaller: &PixelBox) -> u32 {
        //  _______
        // |   __  |
        // |  |__| |
        // |_______| | < offset y
        //  __
        //   ^offset x

        let offset_x = (smaller.x.start - self.x.start) as u32;
        let offset_y = (smaller.y.start - self.y.start) as u32;
        let bigger_width = self.width() as u32;

        offset_y * bigger_width + offset_x
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

    #[test]
    fn pixelbox_intersection() {
        let a = PixelBox { x: 0..10, y: 0..20 };
        let b = PixelBox { x: 4..11, y: 0..5 };

        let c = a.intersection(&b);

        assert!(c.is_some());

        let pb = c.unwrap(); // Does intersect

        assert_eq!(pb.x, 4..10);
        assert_eq!(pb.y, 0..5);
    }

    #[test]
    fn offset_in() {
        let a = PixelBox {
            x: 0..700,
            y: 0..700,
        };
        let b = PixelBox {
            x: 0..350,
            y: 350..700,
        };
        let c = PixelBox {
            x: 350..700,
            y: 350..700,
        };

        let offset = a.offset_in_unchecked(&b);
        let expected = 700 * 350;
        assert_eq!(offset, expected);

        let offset = a.offset_in_unchecked(&c);
        let expected = 700 * 350 + 350;
        assert_eq!(offset, expected);
    }
}
