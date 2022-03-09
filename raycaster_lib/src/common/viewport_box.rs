use nalgebra::{point, Point2, Vector2};

// a 2D range, rectangle described by two points
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

    pub fn add_point(&mut self, x: f32, y: f32) {
        self.upper.x = f32::max(self.upper.x, x);
        self.upper.y = f32::max(self.upper.y, y);
        self.lower.x = f32::min(self.lower.x, x);
        self.lower.y = f32::min(self.lower.y, y);
    }

    pub fn size(&self) -> Vector2<f32> {
        self.upper - self.lower
    }
}

impl Default for ViewportBox {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {

    use super::*;

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
}
