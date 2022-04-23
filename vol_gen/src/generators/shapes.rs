use std::ops::RangeBounds;

use nalgebra::{vector, Vector3};

use crate::config::{Config, GeneratorConfig};

use super::SampleGenerator;

const OBJECT_SIZE: Vector3<u32> = vector![100, 100, 100];

/// Generate volume with a number of randomly placed shapes
pub struct ShapesGenerator {
    shapes: Vec<ShapeInfo>,
}

impl ShapesGenerator {
    /// Constructor
    pub fn from_config(config: &Config) -> ShapesGenerator {
        let dims = config.dims;
        let (n_of_shapes, sample) = match config.generator {
            GeneratorConfig::Shapes {
                n_of_shapes,
                sample,
            } => (n_of_shapes, sample),
            //  Should not happen
            _ => panic!("Bad generator args"),
        };

        let seed = config.seed;

        // Generate n shapes
        let random_shape_gen =
            ShapeInfoGenerator::new(dims, OBJECT_SIZE, vector![10, 10, 10], sample, 10, seed);
        let shapes = random_shape_gen.get_shapes(n_of_shapes);
        ShapesGenerator { shapes }
    }
}

impl SampleGenerator for ShapesGenerator {
    fn sample_at(&self, coords: Vector3<u32>) -> u8 {
        for shape in &self.shapes {
            if coords.x >= shape.position_low.x
                && coords.y >= shape.position_low.y
                && coords.z >= shape.position_low.z
                && coords.x <= shape.position_high.x
                && coords.y <= shape.position_high.y
                && coords.z <= shape.position_high.z
            {
                let offset = coords - shape.position_low;
                return shape.render_at(offset);
            }
        }
        0 // todo background
    }

    fn construct(config: &Config) -> Self {
        ShapesGenerator::from_config(config)
    }
}

// # of enum ShapeType variants
const N_OF_SHAPE_KINDS: u8 = 2;

pub enum ShapeType {
    Cuboid,
    Sphere,
}

/// One shape in volume
pub struct ShapeInfo {
    pub position_low: Vector3<u32>,
    pub position_high: Vector3<u32>,
    pub shape_type: ShapeType,
    pub sample: u8,
}

impl ShapeInfo {
    #[must_use]
    pub fn new(
        position_low: Vector3<u32>,
        position_high: Vector3<u32>,
        shape_type: ShapeType,
        sample: u8,
    ) -> Self {
        Self {
            position_low,
            position_high,
            shape_type,
            sample, // todo
        }
    }

    fn render_at(&self, offset: Vector3<u32>) -> u8 {
        match self.shape_type {
            ShapeType::Cuboid => self.render_cuboid(offset),
            ShapeType::Sphere => self.render_sphere(offset),
        }
    }

    fn render_cuboid(&self, _offset: Vector3<u32>) -> u8 {
        self.sample
    }

    fn render_sphere(&self, offset: Vector3<u32>) -> u8 {
        let offset_f = offset.cast::<f32>();
        let pos_low_f = self.position_low.cast::<f32>();
        let pos_hi_f = self.position_high.cast::<f32>();

        let center = (pos_low_f + pos_hi_f) / 2.0 - pos_low_f;

        let r = (pos_hi_f.x - pos_low_f.x) / 2.0;
        let length = offset_f - center;

        //println!("Render sphere r {r} mag {} ", length.magnitude());
        if length.magnitude() <= r {
            self.sample
        } else {
            0
        }
    }
}

/// Generate shapes
/// Helper type
pub struct ShapeInfoGenerator {
    rng: fastrand::Rng,
    vol_dims: Vector3<u32>,
    size: Vector3<u32>,
    size_variance: Vector3<u32>,
    sample: u8,
    sample_variance: u8,
}

impl ShapeInfoGenerator {
    #[must_use]
    pub fn new(
        vol_dims: Vector3<u32>,
        size: Vector3<u32>,
        size_variance: Vector3<u32>,
        sample: u8,
        sample_variance: u8,
        seed: Option<u64>,
    ) -> Self {
        let rng = fastrand::Rng::new();
        if let Some(seed) = seed {
            rng.seed(seed);
        }

        Self {
            rng,
            vol_dims,
            size,
            size_variance,
            sample,
            sample_variance,
        }
    }

    fn random_shape(&self) -> ShapeType {
        let ran = self.rng.u8(0..N_OF_SHAPE_KINDS);
        match ran {
            0 => ShapeType::Cuboid,
            1 => ShapeType::Sphere,
            _ => panic!("Random shape error"),
        }
    }

    fn random_vector<R>(&self, ranges: Vector3<R>) -> Vector3<u32>
    where
        R: RangeBounds<u32> + Clone,
    {
        let rand_x = self.rng.u32(ranges[0].clone()); // Using index, .x access not working
        let rand_y = self.rng.u32(ranges[1].clone());
        let rand_z = self.rng.u32(ranges[2].clone());
        vector![rand_x, rand_y, rand_z]
    }

    pub fn get_shapes(&self, n: usize) -> Vec<ShapeInfo> {
        (0..n).into_iter().map(|_| self.get_shape()).collect()
    }

    pub fn get_shape(&self) -> ShapeInfo {
        let shape_type = self.random_shape();

        let size_min = self.size - self.size_variance;
        let size_max = self.size + self.size_variance;

        let size_range_x = size_min.x..=size_max.x;
        let size_range_y = size_min.y..=size_max.y;
        let size_range_z = size_min.z..=size_max.z;

        let size_ranges = vector![size_range_x, size_range_y, size_range_z];
        let size = self.random_vector(size_ranges);

        // Spawn shape in positions it fits
        let pos_range_x = 0..=(self.vol_dims.x - size.x);
        let pos_range_y = 0..=(self.vol_dims.y - size.y);
        let pos_range_z = 0..=(self.vol_dims.z - size.z);

        let pos_ranges = vector![pos_range_x, pos_range_y, pos_range_z];
        let position_low = self.random_vector(pos_ranges);

        let position_high = position_low + size;

        let sample = self.random_sample();

        ShapeInfo::new(position_low, position_high, shape_type, sample)
    }

    fn random_sample(&self) -> u8 {
        // Uses saturating intrinsics, so there is no overflow
        let low = self.sample.saturating_sub(self.sample_variance);
        let high = self.sample.saturating_add(self.sample_variance);
        self.rng.u8(low..=high)
    }
}
