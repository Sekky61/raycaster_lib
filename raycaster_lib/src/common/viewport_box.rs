use std::{cmp::min, ops::Range};

use nalgebra::{point, Point2, Vector2};

// a 2D range, rectangle described by two points
#[derive(Clone, Copy, Default)]
pub struct ViewportBox {
    pub lower: Point2<f32>,
    pub upper: Point2<f32>,
}

impl ViewportBox {
    // Maximum viewport, flipped
    pub fn new() -> Self {
        Self {
            lower: point![f32::INFINITY, f32::INFINITY],
            upper: point![f32::NEG_INFINITY, f32::NEG_INFINITY],
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

    pub fn get_pixel_range(&self, resolution: (usize, usize)) -> PixelBox {
        let (width, height) = resolution;
        let width_f = width as f32;
        let height_f = height as f32;

        let mut tile_pixel_size = self.size();
        tile_pixel_size.x = f32::ceil(tile_pixel_size.x * width_f);
        tile_pixel_size.y = f32::ceil(tile_pixel_size.y * height_f);

        let mut start_pixel = self.lower;
        start_pixel.x = f32::floor(start_pixel.x * width_f);
        start_pixel.y = f32::floor(start_pixel.y * height_f);

        let start_x = start_pixel.x as usize;
        let start_y = start_pixel.y as usize;

        let lim_x = tile_pixel_size.x as usize;
        let lim_y = tile_pixel_size.y as usize;

        let end_x = min(start_x + lim_x, width);
        let end_y = min(start_y + lim_y, height);

        PixelBox::new(start_x..end_x, start_y..end_y)
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

        assert_eq!(vp.lower, point![0.5, 0.5]);
        assert_eq!(vp.upper, point![0.5, 0.5]);

        vp.add_point(0.6, 0.6);

        assert_eq!(vp.lower, point![0.5, 0.5]);
        assert_eq!(vp.upper, point![0.6, 0.6]);

        vp.add_point(0.5, 0.7);

        assert_eq!(vp.lower, point![0.5, 0.5]);
        assert_eq!(vp.upper, point![0.6, 0.7]);

        vp.add_point(0.5, 0.4);

        assert_eq!(vp.lower, point![0.5, 0.4]);
        assert_eq!(vp.upper, point![0.6, 0.7]);

        vp.add_point(0.3, 0.2);

        assert_eq!(vp.lower, point![0.3, 0.2]);
        assert_eq!(vp.upper, point![0.6, 0.7]);

        vp.add_point(0.2, 0.8);

        assert_eq!(vp.lower, point![0.2, 0.2]);
        assert_eq!(vp.upper, point![0.6, 0.8]);
    }

    #[test]
    fn cross() {
        let a = ViewportBox {
            lower: point![0.0, 0.0],
            upper: point![1.0, 1.0],
        };

        let b = ViewportBox {
            lower: point![0.5, 0.5],
            upper: point![1.5, 1.5],
        };

        let c = ViewportBox {
            lower: point![1.1, 1.1],
            upper: point![2.0, 1.4],
        };

        let d = ViewportBox {
            lower: point![0.3, -0.1],
            upper: point![0.6, 0.3],
        };

        let e = ViewportBox {
            lower: point![-0.6, 0.1],
            upper: point![0.3, 0.3],
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
