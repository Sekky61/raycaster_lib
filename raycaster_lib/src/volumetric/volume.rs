use crate::common::{BoundBox, Ray};

use crate::TF;
use nalgebra::{point, vector, Matrix4, Point3, Vector3};

/// Interface for blocked volume types
///
/// Used by multithreaded renderer
pub trait Blocked: Send + Sync {
    /// Type of block
    type BlockType: Volume;

    /// Getter for all blocks in a volume
    fn get_blocks(&self) -> &[Self::BlockType];

    /// Getter for block visibility information
    fn get_empty_blocks(&self) -> &[bool];
}

/// Interface for all volume types
///
/// Getters and sampling functions to be used by renderers
/// Returned samples are always `f32`
pub trait Volume: Send {
    /// Returns volume's grid size
    fn get_size(&self) -> Vector3<usize>; // todo u32

    /// Transform ray from world coordinates into volume coordinates
    fn transform_ray(&self, ray: &Ray) -> Option<(Ray, f32)> {
        let bbox = self.get_bound_box();

        let (t0, t1) = bbox.intersect(ray)?;

        let scale_inv = vector![1.0, 1.0, 1.0].component_div(&self.get_scale());
        let lower_vec = bbox.lower - point![0.0, 0.0, 0.0];

        let transform = Matrix4::identity()
            .append_translation(&-lower_vec)
            .append_nonuniform_scaling(&scale_inv);

        let obj_origin = ray.point_from_t(t0);

        let origin = transform.transform_point(&obj_origin);

        let direction = ray.direction.component_mul(&scale_inv);
        let direction = direction.normalize();

        let obj_ray = Ray::new(origin, direction);

        Some((obj_ray, t1 - t0))
    }

    /// Returns volume position
    /// Volume position is the lower corner of `BoundBox`
    fn get_pos(&self) -> Point3<f32> {
        self.get_bound_box().lower
    }

    /// Returns transfer function
    fn get_tf(&self) -> TF;

    /// Sets new transfer function
    /// If volume uses indexing, the indexes are rebuilt
    fn set_tf(&mut self, tf: TF); // todo check that indexes are rebuilt

    /// Returns `true` if volume is indexed as empty in `pos`
    fn is_empty(&self, pos: Point3<f32>) -> bool;

    /// Sample the volume at `pos`.
    /// Panics if `pos` is outside the volume.
    ///
    /// # Arguments
    /// * `pos` - Sampling point in volume coordinates (grid coordinates)
    fn sample_at(&self, pos: Point3<f32>) -> f32;

    /// Sample the volume at `pos` and sample surroundings to get gradient.
    fn sample_at_gradient(&self, pos: Point3<f32>) -> (f32, Vector3<f32>) {
        // Default implementation, can be replaced with a more effective one for concrete volume types
        let sample = self.sample_at(pos);
        let grad_dir = 0.4; // todo take into account voxel shape

        // Samples are taken on higher coordinates, cap safe sample coord range
        let safe_size = self.get_size().map(|v| v as f32 - 1.01);
        // todo change from sampling to gradient compute and optimise

        let sample_x = if pos.x + grad_dir > safe_size.x {
            -self.sample_at(pos - vector![grad_dir, 0.0, 0.0])
        } else {
            self.sample_at(pos + vector![grad_dir, 0.0, 0.0])
        };

        let sample_y = if pos.y + grad_dir > safe_size.y {
            -self.sample_at(pos - vector![0.0, grad_dir, 0.0])
        } else {
            self.sample_at(pos + vector![0.0, grad_dir, 0.0])
        };

        let sample_z = if pos.z + grad_dir > safe_size.z {
            -self.sample_at(pos - vector![0.0, 0.0, grad_dir])
        } else {
            self.sample_at(pos + vector![0.0, 0.0, grad_dir])
        };

        let grad_samples = vector![sample_x, sample_y, sample_z];

        (sample, grad_samples)
    }

    /// Returns bounding box of volume
    fn get_bound_box(&self) -> BoundBox;

    /// Returns shape of voxels/cells
    fn get_scale(&self) -> Vector3<f32>;

    /// Less efficient sampling
    /// Checks bounds and returns `None` if position is outside the volume.
    /// Used for building indexes.
    fn get_data(&self, x: usize, y: usize, z: usize) -> Option<f32>; // todo pos: Point3

    /// Returns iterator of values in block specified by `side` and `base`
    ///
    /// # Params
    /// * `side` - length of the side of the block
    /// * `base` - lowest point of the block
    fn get_block(&self, side: usize, base: Point3<usize>) -> VolumeBlockIter<Self> {
        VolumeBlockIter {
            volume: self,
            base,
            side,
            iter_progress: 0,
        }
    }

    /// Returns the name of the volume
    fn get_name() -> &'static str;

    fn build_empty_index(&mut self);
}

// pub struct VolumeHit {
//     color: Vector4<f32>,
//     gradient: Vector3<f32>, // not normalized
// }

// impl VolumeHit {
//     pub fn new(color: Vector4<f32>, gradient: Vector3<f32>) -> Self {
//         Self { color, gradient }
//     }
// }

// Warning: Returns None if block element is outside the volume
// In other words, returned None option does not necesarily mean the iterator is exhausted
pub struct VolumeBlockIter<'a, V>
where
    V: 'a + Volume + ?Sized,
{
    volume: &'a V,
    base: Point3<usize>,
    side: usize,
    iter_progress: usize,
}

impl<'a, V: Volume> Iterator for VolumeBlockIter<'a, V> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let block_len = self.side * self.side * self.side;

        if self.iter_progress < block_len {
            let offset_x = self.iter_progress / self.side / self.side;
            let offset_y = self.iter_progress / self.side;
            let offset_z = self.iter_progress % self.side;

            let sample = self.volume.get_data(
                self.base.x + offset_x,
                self.base.y + offset_y,
                self.base.z + offset_z,
            );

            self.iter_progress += 1;

            sample
        } else {
            None
        }
    }
}
