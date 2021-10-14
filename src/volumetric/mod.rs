pub mod vol_builder;
pub mod vol_reader;
pub mod volume;

mod block_volume;
mod linear_volume;

pub use linear_volume::LinearVolume;
pub use vol_builder::VolumeBuilder;
pub use volume::Volume;

#[cfg(test)]
mod test {

    use crate::ray::Ray;
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
        let bbox = cube_volume();
        let ray = Ray {
            origin: vector![-0.4, 0.73, 0.0],
            direction: vector![1.0, 0.0, 1.0],
        };
        let inter = bbox.intersect(&ray);
        println!("intersection: {:?}", inter);
        assert!(inter.is_some());
    }

    #[test]
    fn not_intersecting() {
        let bbox = cube_volume();
        let ray = Ray {
            origin: vector![2.0, 2.0, 2.0],
            direction: vector![1.0, 1.0, 8.0],
        };

        assert!(bbox.intersect(&ray).is_none());
    }
}
