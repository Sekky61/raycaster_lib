use nalgebra::{vector, Vector3};

use super::{
    vol_builder::{color, BuildVolume, RGBA},
    Volume, VolumeBuilder,
};

pub struct LinearVolume {
    size: Vector3<usize>,
    border: u32,
    scale: Vector3<f32>,    // shape of voxels
    vol_dims: Vector3<f32>, // size * scale = resulting size of bounding box ; max of bounding box
    data: Vec<RGBA>,
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

    fn get_3d_data(&self, x: usize, y: usize, z: usize) -> RGBA {
        //println!("Getting {} {} {}", x, y, z);
        let val = self.data.get(self.get_3d_index(x, y, z));
        match val {
            Some(&v) => v,
            None => color::zero(),
        }
    }
}

impl Volume for LinearVolume {
    fn get_dims(&self) -> Vector3<f32> {
        self.vol_dims
    }

    fn sample_at(&self, pos: Vector3<f32>) -> RGBA {
        // todo taky zkusit rozseknout
        let x_low = pos.x as usize;
        let y_low = pos.y as usize;
        let z_low = pos.z as usize;

        let x_high = x_low + 1;
        let y_high = y_low + 1;
        let z_high = z_low + 1;

        let x_t = pos.x.fract();
        let y_t = pos.y.fract();
        let z_t = pos.z.fract();

        let c000 = self.get_3d_data(x_low, y_low, z_low);
        let c001 = self.get_3d_data(x_low, y_low, z_high);
        let c010 = self.get_3d_data(x_low, y_high, z_low);
        let c011 = self.get_3d_data(x_low, y_high, z_high);
        let c100 = self.get_3d_data(x_high, y_low, z_low);
        let c101 = self.get_3d_data(x_high, y_low, z_high);
        let c110 = self.get_3d_data(x_high, y_high, z_low);
        let c111 = self.get_3d_data(x_high, y_high, z_high);

        let inv_x_t = 1.0 - x_t;
        let c00 = c000 * inv_x_t + c100 * x_t;
        let c01 = c001 * inv_x_t + c101 * x_t;
        let c10 = c010 * inv_x_t + c110 * x_t;
        let c11 = c011 * inv_x_t + c111 * x_t;

        let inv_y_t = 1.0 - y_t;
        let c0 = c00 * inv_y_t + c10 * y_t;
        let c1 = c01 * inv_y_t + c11 * y_t;

        c0 * (1.0 - z_t) + c1 * z_t
    }

    fn is_in(&self, pos: &Vector3<f32>) -> bool {
        self.vol_dims.x > pos.x
            && self.vol_dims.y > pos.y
            && self.vol_dims.z > pos.z
            && pos.x > 0.0
            && pos.y > 0.0
            && pos.z > 0.0
    }

    fn get_data(&self, x: usize, y: usize, z: usize) -> RGBA {
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

    use crate::ray::Ray;

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
