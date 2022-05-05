/*
    raycaster_lib
    Author: Michal Majer
    Date: 2022-05-05
*/

//! Structures and functions used throughout the library

mod bound_box;
mod ray;
mod value_range;
mod viewport_box;

pub use bound_box::{BoundBox, BoundBoxIterator};
use nalgebra::{vector, Vector3};
pub use ray::Ray;
pub use value_range::ValueRange;
pub use viewport_box::{PixelBox, ViewportBox};

use crate::TF;

/// Divides volume into blocks.
/// Rounds up.
///
/// # Params
/// * `size` - resolution of volume in voxels
/// * `side` - side of block in voxels
/// * `overlap` - overlap of blocks (usually 1 voxel)
///
/// # Example
/// ```ignore
/// # use nalgebra::vector;
/// # use crate::common::blockify;
/// let size = vector![19, 20, 21];
/// assert_eq!(blockify(size, 10, 1), vector![2, 3, 3]);
/// ```
pub fn blockify(size: Vector3<usize>, side: usize, overlap: usize) -> Vector3<usize> {
    let cells = side - overlap; // cells per block
                                // (size-1) -- number of cells
                                // (+cells-1 / cells) -- division with rounding up
    let x = size + vector![cells, cells, cells];
    let y = x - vector![2, 2, 2];
    y / cells
}

/// Calculates ranges of samples yielding opaque colors, given `tf`.
/// Assumes `tf` (transfer function) operates on values `<0.0;255.0>`.
///
/// Returns vector of `ValueRange`.
///
/// # Example
/// ```ignore
/// # use nalgebra::vector;
/// # use crate::common::ValueRange;
/// let tf = |x: f32| {
/// if x > 10.5 && x < 20.5 {
///     return vector![1.0, 1.0, 1.0, 1.0];
/// } else if x > 80.1 && x < 85.8 {
///     return vector![1.0, 1.0, 1.0, 0.1];
/// } else {
///     return vector![1.0, 1.0, 1.0, 0.0];
/// }
/// };
///
/// assert_eq!(
/// tf_visible_range(tf),
/// vec![
///     (11.0..21.0).into(),
///     (81.0..86.0).into()
/// ]
/// );
/// ```
pub fn tf_visible_range(tf: TF) -> Vec<ValueRange> {
    let mut ranges = vec![];
    let mut range: Option<ValueRange> = None;

    for v in 0..=255 {
        let v = v as f32;
        let sample = tf(v);
        if sample.w == 0.0 {
            if let Some(mut r) = range.take() {
                r.high = v;
                ranges.push(r);
            }
        } else if range.is_none() {
            range = Some(ValueRange::seed(v));
        }
    }

    if let Some(mut range) = range {
        range.high = 255.0;
        ranges.push(range);
    }
    ranges
}

#[cfg(test)]
mod test {

    use nalgebra::vector;

    use super::*;

    #[test]
    fn tf_range() {
        let tf = |x: f32| {
            if x > 10.5 && x < 20.5 {
                return vector![1.0, 1.0, 1.0, 1.0];
            } else {
                return vector![1.0, 1.0, 1.0, 0.0];
            }
        };

        let ranges = tf_visible_range(tf);

        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0], (11.0..21.0).into());
    }

    #[test]
    fn tf_range_split() {
        let tf = |x: f32| {
            if x > 10.5 && x < 20.5 {
                return vector![1.0, 1.0, 1.0, 1.0];
            } else if x > 80.1 && x < 85.8 {
                return vector![1.0, 1.0, 1.0, 0.1];
            } else {
                return vector![1.0, 1.0, 1.0, 0.0];
            }
        };

        assert_eq!(
            tf_visible_range(tf),
            vec![(11.0..21.0).into(), (81.0..86.0).into()]
        );
    }

    #[test]
    fn blockify_3() {
        let side = 3;
        let overlap = 1;

        let size = vector![5, 5, 5];
        assert_eq!(blockify(size, side, overlap), vector![2, 2, 2]);

        let size = vector![6, 6, 6];
        assert_eq!(blockify(size, side, overlap), vector![3, 3, 3]);

        let size = vector![6, 6, 7];
        assert_eq!(blockify(size, side, overlap), vector![3, 3, 3]);

        let size = vector![6, 6, 8];
        assert_eq!(blockify(size, side, overlap), vector![3, 3, 4]);
    }

    #[test]
    fn blockify_10() {
        let side = 10;
        let overlap = 1;

        let size = vector![5, 5, 5];
        assert_eq!(blockify(size, side, overlap), vector![1, 1, 1]);

        let size = vector![11, 10, 12];
        assert_eq!(blockify(size, side, overlap), vector![2, 1, 2]);

        let size = vector![19, 20, 21];
        assert_eq!(blockify(size, side, overlap), vector![2, 3, 3]);

        let size = vector![105, 57, 67];
        assert_eq!(blockify(size, side, overlap), vector![12, 7, 8]);
    }

    #[test]
    fn blockify_11() {
        let side = 11;
        let overlap = 1;

        let size = vector![5, 5, 5];
        assert_eq!(blockify(size, side, overlap), vector![1, 1, 1]);

        let size = vector![10, 11, 12];
        assert_eq!(blockify(size, side, overlap), vector![1, 1, 2]);

        let size = vector![19, 20, 21];
        assert_eq!(blockify(size, side, overlap), vector![2, 2, 2]);

        let size = vector![105, 57, 67];
        assert_eq!(blockify(size, side, overlap), vector![11, 6, 7]);
    }
}
