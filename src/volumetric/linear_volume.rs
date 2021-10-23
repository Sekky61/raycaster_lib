use nalgebra::{vector, Vector3};

use crate::ray::Ray;

use super::{vol_builder::BuildVolume, Volume, VolumeBuilder};

pub struct LinearVolume {
    size: Vector3<usize>,
    border: u32,
    scale: Vector3<f32>,    // shape of voxels
    vol_dims: Vector3<f32>, // size * scale = resulting size of bounding box ; max of bounding box
    data: Vec<f32>,
}

impl std::fmt::Debug for LinearVolume {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Volume")
            .field("size", &self.size)
            .field("border", &self.border)
            .field("scale", &self.scale)
            .field("vol_dims", &self.vol_dims)
            .field("data len ", &self.data.len())
            .finish()
    }
}

impl LinearVolume {
    fn get_3d_index(&self, x: usize, y: usize, z: usize) -> usize {
        z + y * self.size.z + x * self.size.y * self.size.z
    }

    fn get_3d_data(&self, x: usize, y: usize, z: usize) -> f32 {
        //println!("Getting {} {} {}", x, y, z);
        let val = self.data.get(self.get_3d_index(x, y, z));
        match val {
            Some(&v) => v,
            None => 0.0,
        }
    }
}

impl Volume for LinearVolume {
    fn get_dims(&self) -> Vector3<f32> {
        self.vol_dims
    }

    fn sample_at(&self, pos: Vector3<f32>) -> f32 {
        let x_low = pos[0].floor() as usize;
        let y_low = pos[1].floor() as usize;
        let z_low = pos[2].floor() as usize;

        let x_high = x_low + 1;
        let y_high = y_low + 1;
        let z_high = z_low + 1;

        let x_t = pos.x.fract();
        let y_t = pos.y.fract();
        let z_t = pos.z.fract();

        let c000 = self.get_3d_data(x_low, y_low, z_low) as f32;
        let c001 = self.get_3d_data(x_low, y_low, z_high) as f32;
        let c010 = self.get_3d_data(x_low, y_high, z_low) as f32;
        let c011 = self.get_3d_data(x_low, y_high, z_high) as f32;
        let c100 = self.get_3d_data(x_high, y_low, z_low) as f32;
        let c101 = self.get_3d_data(x_high, y_low, z_high) as f32;
        let c110 = self.get_3d_data(x_high, y_high, z_low) as f32;
        let c111 = self.get_3d_data(x_high, y_high, z_high) as f32;

        let c00 = c000 * (1.0 - x_t) + c100 * x_t;
        let c01 = c001 * (1.0 - x_t) + c101 * x_t;
        let c10 = c010 * (1.0 - x_t) + c110 * x_t;
        let c11 = c011 * (1.0 - x_t) + c111 * x_t;

        let c0 = c00 * (1.0 - y_t) + c10 * y_t;
        let c1 = c01 * (1.0 - y_t) + c11 * y_t;

        c0 * (1.0 - z_t) + c1 * z_t
    }

    fn is_in(&self, pos: Vector3<f32>) -> bool {
        self.vol_dims.x > pos.x
            && self.vol_dims.y > pos.y
            && self.vol_dims.z > pos.z
            && pos.x > 0.0
            && pos.y > 0.0
            && pos.z > 0.0
    }

    fn intersect(&self, ray: &Ray) -> Option<(f32, f32)> {
        // t value of intersection with 6 planes of bounding box
        let t0x = (0.0 - ray.origin.x) / ray.direction.x;
        let t1x = (self.vol_dims.x - ray.origin.x) / ray.direction.x;
        let t0y = (0.0 - ray.origin.y) / ray.direction.y;
        let t1y = (self.vol_dims.y - ray.origin.y) / ray.direction.y;
        let t0z = (0.0 - ray.origin.z) / ray.direction.z;
        let t1z = (self.vol_dims.z - ray.origin.z) / ray.direction.z;

        let tmin = f32::max(
            f32::max(f32::min(t0x, t1x), f32::min(t0y, t1y)),
            f32::min(t0z, t1z),
        );
        let tmax = f32::min(
            f32::min(f32::max(t0x, t1x), f32::max(t0y, t1y)),
            f32::max(t0z, t1z),
        );

        // if tmax < 0, ray (line) is intersecting AABB, but the whole AABB is behind us
        if tmax.is_sign_negative() {
            return None;
        }

        // if tmin > tmax, ray doesn't intersect AABB
        if tmin > tmax {
            return None;
        }

        Some((tmin, tmax))
    }

    fn get_data(&self, x: usize, y: usize, z: usize) -> f32 {
        self.get_3d_data(x, y, z)
    }

    fn get_size(&self) -> Vector3<usize> {
        self.size
    }
}

impl BuildVolume for LinearVolume {
    fn build(builder: VolumeBuilder) -> Self {
        let vol_dims = (builder.size - vector![1, 1, 1]) // side length is n-1 times the point
            .cast::<f32>()
            .component_mul(&builder.scale);
        LinearVolume {
            size: builder.size,
            border: builder.border,
            scale: builder.scale,
            vol_dims,
            data: builder.data,
        }
    }
}

#[cfg(test)]
mod test {

    use nalgebra::vector;

    use super::*;

    fn cube_volume() -> LinearVolume {
        VolumeBuilder::white_vol().build()
    }

    #[test]
    fn intersect_works() {
        let bbox = cube_volume();
        let ray = Ray {
            origin: vector![-1.0, -1.0, 0.0],
            direction: vector![1.0, 1.0, 1.0],
        };
        let inter = bbox.intersect(&ray);
        println!("intersection: {:?}", inter);
        assert!(inter.is_some());
    }

    #[test]
    fn intersect_works2() {
        let vol = cube_volume();
        let ray = Ray {
            origin: vector![-0.4, 0.73, 0.0],
            direction: vector![1.0, 0.0, 1.0],
        };
        let inter = vol.intersect(&ray);
        println!("intersection: {:?}", inter);
        assert!(inter.is_some());
    }

    #[test]
    fn not_intersecting() {
        let vol = cube_volume();
        let ray = Ray {
            origin: vector![200.0, 200.0, 200.0],
            direction: vector![1.0, 0.0, 0.0],
        };

        assert!(vol.intersect(&ray).is_none());
    }
}
