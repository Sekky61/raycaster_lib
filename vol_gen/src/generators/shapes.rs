use std::ops::RangeBounds;

use nalgebra::{vector, Vector3};

pub struct ShapesGenerator {
    dims: Vector3<usize>,
    shapes: Vec<ShapeInfo>,
}

impl ShapesGenerator {
    #[must_use]
    pub fn new(dims: Vector3<usize>, n_of_shapes: usize) -> Self {
        let random_shape_gen = ShapeInfoGenerator::new(dims, vector![5, 5, 5], vector![0, 0, 0]);
        let shapes = random_shape_gen.get_shapes(n_of_shapes);
        Self { dims, shapes }
    }
}

const N_OF_SHAPE_KINDS: u8 = 1;

pub enum ShapeType {
    Cuboid,
}

pub struct ShapeInfo {
    position: Vector3<usize>,
    size: Vector3<usize>,
    shape_type: ShapeType,
}

impl ShapeInfo {
    #[must_use]
    pub fn new(position: Vector3<usize>, size: Vector3<usize>, shape_type: ShapeType) -> Self {
        Self {
            position,
            size,
            shape_type,
        }
    }

    pub fn new_generator(
        vol_dims: Vector3<usize>,
        size: Vector3<usize>,
        size_variance: Vector3<usize>,
    ) -> ShapeInfoGenerator {
        ShapeInfoGenerator::new(vol_dims, size, size_variance)
    }
}

pub struct ShapeInfoGenerator {
    rng: fastrand::Rng,
    vol_dims: Vector3<usize>,
    size: Vector3<usize>,
    size_variance: Vector3<usize>,
}

impl ShapeInfoGenerator {
    #[must_use]
    pub fn new(
        vol_dims: Vector3<usize>,
        size: Vector3<usize>,
        size_variance: Vector3<usize>,
    ) -> Self {
        let rng = fastrand::Rng::new();
        Self {
            rng,
            vol_dims,
            size,
            size_variance,
        }
    }

    fn random_shape(&self) -> ShapeType {
        let ran = self.rng.u8(0..N_OF_SHAPE_KINDS);
        match ran {
            0 => ShapeType::Cuboid,
            _ => panic!("Random shape error"),
        }
    }

    fn random_vector<R>(&self, ranges: Vector3<R>) -> Vector3<usize>
    where
        R: RangeBounds<usize> + Clone,
    {
        let rand_x = self.rng.usize(ranges[0].clone()); // Using index, .x access not working
        let rand_y = self.rng.usize(ranges[1].clone());
        let rand_z = self.rng.usize(ranges[2].clone());
        vector![rand_x, rand_y, rand_z]
    }

    pub fn get_shapes(&self, n: usize) -> Vec<ShapeInfo> {
        (0..n).into_iter().map(|i| self.get_shape()).collect()
    }

    pub fn get_shape(&self) -> ShapeInfo {
        let size_min = self.size - self.size_variance;
        let size_max = self.size + self.size_variance;

        let size_range_x = size_min.x..size_max.x;
        let size_range_y = size_min.y..size_max.y;
        let size_range_z = size_min.z..size_max.z;

        let size_ranges = vector![size_range_x, size_range_y, size_range_z];
        let size = self.random_vector(size_ranges);

        // Spawn shape in positions it fits
        let pos_range_x = 0..(self.vol_dims.x - size.x);
        let pos_range_y = 0..(self.vol_dims.y - size.y);
        let pos_range_z = 0..(self.vol_dims.z - size.z);

        let pos_ranges = vector![pos_range_x, pos_range_y, pos_range_z];
        let position = self.random_vector(pos_ranges);

        let shape_type = self.random_shape();

        ShapeInfo {
            position,
            size,
            shape_type,
        }
    }
}
