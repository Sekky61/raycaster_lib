use nalgebra::{vector, Point3, Vector3};

use super::{
    vol_builder::{BuildVolume, VolumeMetadata},
    Volume, TF,
};

pub struct LinearVolume {
    position: Vector3<f32>,
    size: Vector3<usize>,
    scale: Vector3<f32>,    // shape of voxels
    vol_dims: Vector3<f32>, // size * scale = resulting size of bounding box ; max of bounding box
    data: Vec<f32>,
    tf: TF,
}

impl std::fmt::Debug for LinearVolume {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Volume")
            .field("size", &self.size)
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

    fn get_block_data_half(&self, base: usize) -> [f32; 4] {
        [
            self.data[base],
            self.data[base + 1],
            self.data[base + self.size.y],
            self.data[base + self.size.y + 1],
        ]
    }
}

impl Volume for LinearVolume {
    fn get_dims(&self) -> Vector3<f32> {
        self.vol_dims
    }

    fn sample_at(&self, pos: Point3<f32>) -> f32 {
        // todo taky zkusit rozseknout
        let x_low = pos.x as usize;
        let y_low = pos.y as usize;
        let z_low = pos.z as usize;

        let x_t = pos.x.fract();
        let y_t = pos.y.fract();
        let z_t = pos.z.fract();

        let base = self.get_3d_index(x_low, y_low, z_low);

        let first_index = base;
        let second_index = base + self.size.z * self.size.y;

        let first_data = self.get_block_data_half(first_index);

        //let first_data = self.get_block_data_half(first_index);
        let [c000, c001, c010, c011] = first_data;

        let inv_z_t = 1.0 - z_t;
        let inv_y_t = 1.0 - y_t;

        // first plane

        let c00 = c000 * inv_z_t + c001 * z_t; // z low
        let c01 = c010 * inv_z_t + c011 * z_t; // z high
        let c0 = c00 * inv_y_t + c01 * y_t; // point on yz plane

        // second plane

        let second_data = self.get_block_data_half(second_index);

        //let second_data = self.get_block_data_half(second_index);
        let [c100, c101, c110, c111] = second_data;

        let c10 = c100 * inv_z_t + c101 * z_t; // z low
        let c11 = c110 * inv_z_t + c111 * z_t; // z high
        let c1 = c10 * inv_y_t + c11 * y_t; // point on yz plane

        c0 * (1.0 - x_t) + c1 * x_t
    }

    fn is_in(&self, pos: &Point3<f32>) -> bool {
        self.vol_dims.x > pos.x
            && self.vol_dims.y > pos.y
            && self.vol_dims.z > pos.z
            && pos.x > 0.0
            && pos.y > 0.0
            && pos.z > 0.0
    }

    fn get_data(&self, x: usize, y: usize, z: usize) -> f32 {
        self.get_3d_data(x, y, z)
    }

    fn get_size(&self) -> Vector3<usize> {
        self.size
    }

    fn get_pos(&self) -> Vector3<f32> {
        self.position
    }

    fn get_tf(&self) -> TF {
        self.tf
    }
}

impl BuildVolume<u8> for LinearVolume {
    fn build(metadata: VolumeMetadata<u8>) -> Result<LinearVolume, &'static str> {
        println!("Build started");

        let data = metadata.data.ok_or("No volumetric data passed")?;
        let slice = data.get_slice().ok_or("No data inside datasource")?;
        let offset = metadata.data_offset.unwrap_or(0);

        let data: Vec<f32> = slice[offset..].iter().map(|&val| val.into()).collect();

        // let data_range_max = data.iter().fold(-10000.0, |cum, &v| f32::max(v, cum));
        // let data_range_min = data.iter().fold(100000.0, |cum, &v| f32::min(v, cum));

        // println!("Build data range: {data_range_min} to {data_range_max}");

        let size = metadata.size.ok_or("No size")?;
        let scale = metadata.scale.unwrap_or_else(|| vector![1.0, 1.0, 1.0]);
        let tf = metadata.tf.ok_or("No transfer function")?;

        let vol_dims = size.map(|v| (v - 1) as f32).component_mul(&scale);

        let position = metadata.position.unwrap_or_else(Vector3::zeros);

        Ok(LinearVolume {
            position,
            size,
            scale,
            vol_dims,
            data,
            tf,
        })
    }
}

#[cfg(test)]
mod test {

    use nalgebra::{point, vector};

    use crate::ray::Ray;

    use super::*;
    use crate::test_helpers::*;

    #[test]
    fn intersect_works() {
        let bbox: LinearVolume = white_volume();
        let ray = Ray {
            origin: point![-1.0, -1.0, 0.0],
            direction: vector![1.0, 1.0, 1.0],
        };
        let inter = bbox.intersect(&ray);
        println!("intersection: {:?}", inter);
        assert!(inter.is_some());
    }

    #[test]
    fn intersect_works2() {
        let vol: LinearVolume = white_volume();
        let ray = Ray {
            origin: point![-0.4, 0.73, 0.0],
            direction: vector![1.0, 0.0, 1.0],
        };
        let inter = vol.intersect(&ray);
        println!("intersection: {:?}", inter);
        assert!(inter.is_some());
    }

    #[test]
    fn not_intersecting() {
        let vol: LinearVolume = white_volume();
        let ray = Ray {
            origin: point![200.0, 200.0, 200.0],
            direction: vector![1.0, 0.0, 0.0],
        };

        assert!(vol.intersect(&ray).is_none());
    }
}
