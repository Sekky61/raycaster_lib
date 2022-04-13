use nalgebra::{point, Point3, Vector3};

use super::Ray;

#[derive(Debug, Clone, Copy)]
pub struct BoundBox {
    pub lower: Point3<f32>,
    pub upper: Point3<f32>,
}

impl BoundBox {
    pub fn new(lower: Point3<f32>, upper: Point3<f32>) -> BoundBox {
        BoundBox { lower, upper }
    }

    /// Zero sized boundbox
    ///
    /// For testing purposes, where bound box is irrelevant
    pub fn empty() -> BoundBox {
        BoundBox {
            lower: point![0.0, 0.0, 0.0],
            upper: point![0.0, 0.0, 0.0],
        }
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

    pub fn intersect(&self, ray: &Ray) -> Option<(f32, f32)> {
        // Source: An Efficient and Robust Ray–Box Intersection Algorithm. Amy Williams et al. 2004.
        // http://citeseerx.ist.psu.edu/viewdoc/summary?doi=10.1.1.64.7663

        // t value of intersection with the 6 planes of a bounding box
        let t0 = (self.lower - ray.origin).component_div(&ray.direction);
        let t1 = (self.upper - ray.origin).component_div(&ray.direction);

        // [ (min,max) , (min,max) , (min,max) ]
        let t_minmax = t0.zip_map(&t1, |t0, t1| if t0 < t1 { (t0, t1) } else { (t1, t0) });

        let tmin = f32::max(f32::max(t_minmax.x.0, t_minmax.y.0), t_minmax.z.0);
        let tmax = f32::min(f32::min(t_minmax.x.1, t_minmax.y.1), t_minmax.z.1);

        // if tmax < 0, ray is intersecting AABB, but the whole AABB is behind us
        if tmax.is_sign_negative() {
            return None;
        }

        // if tmin > tmax, ray doesn't intersect AABB
        if tmin > tmax {
            return None;
        }

        Some((tmin, tmax))
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
