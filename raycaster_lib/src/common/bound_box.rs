use nalgebra::{point, Point3, Vector3};

use super::Ray;

/// Bounding box of object in world space.
/// Defined by two points, `lower` and `upper`.
/// Volume is defined as the space between these points.
/// Box is axis-aligned.
///
/// `BoundBox` implements [`IntoIterator`].
/// This way, corner points can be itarated over.
/// ```ignore
/// # use crate::common::BoundBox;
/// # use nalgebra::point;
/// let bbox = BoundBox::new(point![0.0, 0.0, 0.0], point![1.0, 1.0, 1.0]);
/// for point in bbox {
///     println!{"Point: {point:?}"};
/// }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct BoundBox {
    /// Lowest point of volume
    pub lower: Point3<f32>,
    /// Highest point of volume
    pub upper: Point3<f32>,
}

impl BoundBox {
    /// Construct new `BoundBox` from their defining points.
    pub fn new(lower: Point3<f32>, upper: Point3<f32>) -> BoundBox {
        BoundBox { lower, upper }
    }

    /// Alternative construction method.
    /// Uses `position` as `lower` point and calculates `upper` as `position + dimensions`.
    pub fn from_position_dims(position: Point3<f32>, dimensions: Vector3<f32>) -> BoundBox {
        BoundBox {
            lower: position,
            upper: position + dimensions,
        }
    }

    /// Zero sized boundbox.
    /// Used for testing purposes, where `BoundBox` is irrelevant.
    pub fn empty() -> BoundBox {
        BoundBox {
            lower: point![0.0, 0.0, 0.0],
            upper: point![0.0, 0.0, 0.0],
        }
    }

    /// Returns size of the volume in units
    pub fn dims(&self) -> Vector3<f32> {
        self.upper - self.lower
    }

    /// Tests if `pos` is inside bounding box.
    pub fn is_in(&self, pos: &Point3<f32>) -> bool {
        self.upper.x > pos.x
            && self.upper.y > pos.y
            && self.upper.z > pos.z
            && pos.x > self.lower.x
            && pos.y > self.lower.y
            && pos.z > self.lower.z
    }

    /// Returns whether intersection between `BoundBox` and `Ray` exists.
    /// If intersection exists, function also returns the segment of ray.
    ///
    /// # Returns
    ///
    /// * `None` - if intersection does not exist
    /// * `Some(t1, t2)` - if objects intersect. `t1` and `t2` are points of intersection on the ray.
    ///
    /// To get a point from `t1`, see [`Ray::point_from_t`]
    pub fn intersect(&self, ray: &Ray) -> Option<(f32, f32)> {
        // Source: An Efficient and Robust Ray–Box Intersection Algorithm. Amy Williams et al. 2004.
        // http://citeseerx.ist.psu.edu/viewdoc/summary?doi=10.1.1.64.7663

        // t value of intersection with the 6 planes of a bounding box
        let t0 = (self.lower - ray.origin).component_div(&ray.direction);
        let t1 = (self.upper - ray.origin).component_div(&ray.direction);

        // [ (min,max) , (min,max) , (min,max) ]
        let t_minmax = t0.zip_map(&t1, |t0, t1| if t0 < t1 { (t0, t1) } else { (t1, t0) });

        let tmin = f32::max(f32::max(t_minmax.x.0, t_minmax.y.0), t_minmax.z.0) + 0.0001;
        let tmax = f32::min(f32::min(t_minmax.x.1, t_minmax.y.1), t_minmax.z.1) - 0.0001;

        // the whole box is behind us
        if tmax.is_sign_negative() {
            return None;
        }

        if tmin > tmax {
            return None;
        }

        Some((tmin, tmax))
    }
}

/// Iteration structure, iterates over corners of a `BoundBox`.
pub struct BoundBoxIterator {
    lower: Point3<f32>,
    upper: Point3<f32>,
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

#[cfg(test)]
mod test {

    use nalgebra::point;

    use super::*;

    #[test]
    fn bbox_iter() {
        let bbox = BoundBox::new(point![0.0, 0.0, 0.0], point![1.0, 1.0, 1.0]);

        let points: Vec<_> = bbox.into_iter().collect();

        assert_eq!(
            points,
            vec![
                point![0.0, 0.0, 0.0],
                point![1.0, 0.0, 0.0],
                point![1.0, 1.0, 0.0],
                point![0.0, 1.0, 0.0],
                point![0.0, 0.0, 1.0],
                point![1.0, 0.0, 1.0],
                point![1.0, 1.0, 1.0],
                point![0.0, 1.0, 1.0]
            ]
        );
    }
}
