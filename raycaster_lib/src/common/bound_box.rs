use nalgebra::{point, Point3, Vector3};

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
