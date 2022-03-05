use nalgebra::{point, Point2, Point3, Vector2, Vector3};

// Todo rename to common / types

pub struct Ray {
    // todo t parameter
    pub origin: Point3<f32>,
    pub direction: Vector3<f32>, // todo Unit
}

impl Ray {
    pub fn from_3(origin: Point3<f32>, direction: Vector3<f32>) -> Ray {
        Ray { origin, direction }
    }

    pub fn point_from_t(&self, t: f32) -> Point3<f32> {
        self.origin + t * self.direction
    }

    pub fn get_direction(&self) -> Vector3<f32> {
        self.direction
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BoundBox {
    pub lower: Point3<f32>,
    pub upper: Point3<f32>,
}

impl BoundBox {
    pub fn new(lower: Point3<f32>, upper: Point3<f32>) -> BoundBox {
        BoundBox { lower, upper }
    }

    pub fn from_position_dims(position: Point3<f32>, dimensions: Vector3<f32>) -> BoundBox {
        BoundBox {
            lower: position,
            upper: position + dimensions,
        }
    }

    pub fn position(&self) -> Point3<f32> {
        self.lower
    }

    pub fn dims(&self) -> Vector3<f32> {
        self.upper - self.lower
    }

    pub fn is_in(&self, pos: &Point3<f32>) -> bool {
        self.upper.x > pos.x
            && self.upper.y > pos.y
            && self.upper.z > pos.z
            && pos.x > self.lower.x
            && pos.y > self.lower.y
            && pos.z > self.lower.z
    }
}

pub struct BoundBoxIterator {
    pub lower: Point3<f32>,
    pub upper: Point3<f32>,
    state: u8,
}

impl Iterator for BoundBoxIterator {
    type Item = Point3<f32>;

    fn next(&mut self) -> Option<Self::Item> {
        let p = match self.state {
            0 => self.lower,
            1 => point![self.upper.x, self.lower.y, self.lower.z],
            2 => point![self.upper.x, self.upper.y, self.lower.z],
            3 => point![self.lower.x, self.upper.y, self.lower.z],
            4 => point![self.lower.x, self.lower.y, self.upper.z],
            5 => point![self.upper.x, self.lower.y, self.upper.z],
            6 => self.upper,
            7 => point![self.lower.x, self.upper.y, self.upper.z],
            _ => return None,
        };
        self.state += 1;
        Some(p)
    }
}

impl IntoIterator for BoundBox {
    type Item = Point3<f32>;

    type IntoIter = BoundBoxIterator;

    fn into_iter(self) -> Self::IntoIter {
        BoundBoxIterator {
            lower: self.lower,
            upper: self.upper,
            state: 0,
        }
    }
}

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
