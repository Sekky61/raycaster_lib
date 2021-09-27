use nalgebra::{vector, Vector3};

use crate::vol_reader::RGBColor;

pub struct Camera {
    position: Vector3<f32>,
    target: Vector3<f32>,
    f: f32,
    resolution: (usize, usize),
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            position: vector![-1.5, 0.5, 0.5],
            target: vector![0.5, 0.5, 0.5],
            f: 1.0,
            resolution: (512, 512),
        }
    }

    pub fn change_pos(&mut self, change: Vector3<f32>) {
        self.position += change;
    }

    pub fn get_resolution(&self) -> (usize, usize) {
        self.resolution
    }

    pub fn cast_rays(&self) -> Vec<u32> {
        let mut buffer: Vec<u32> = vec![0; self.resolution.0 * self.resolution.1];

        let origin = self.position;
        let plane_x_offset = self.f;

        let width = 3.0;

        let bbox = BoundBox::new();

        let mut counter = (0, 0);

        let white_vec = vector![255.0, 255.0, 255.0];

        for y in 0..self.resolution.1 {
            for x in 0..self.resolution.0 {
                let offset_x_rel = x as f32 / self.resolution.0 as f32 - 0.5;
                let offset_y_rel = y as f32 / self.resolution.1 as f32 - 0.5;

                let view_point = vector![
                    origin.x + plane_x_offset,
                    origin.y + offset_x_rel * width,
                    origin.z + offset_y_rel * width
                ];

                let direction = view_point - origin;

                let ray = Ray::new(origin, direction);

                let int_res = bbox.intersect(&ray);

                // println!(
                //     "R {} | {} int: {}",
                //     ray.origin,
                //     ray.direction,
                //     int_res.is_some()
                // );

                match int_res {
                    Some((t1, t2)) => {
                        counter.0 += 1;
                        let color_v = (t2 - t1) * white_vec;
                        let color = RGBColor::from_vals(
                            color_v[0] as u8,
                            color_v[1] as u8,
                            color_v[2] as u8,
                        );
                        buffer[y * self.resolution.0 + x] = color.to_int();
                        //println!("i.s. ({} {})", ray.point_from_t(t1), ray.point_from_t(t2))
                    }
                    None => counter.1 += 1,
                }
            }
        }

        println!(
            "Ray hit rate: {} at cam ({} | {} | {}) window ({})",
            counter.0 as f32 / (counter.0 as f32 + counter.1 as f32),
            origin.x,
            origin.y,
            origin.z,
            0
        );
        buffer
    }
}

pub struct Ray {
    origin: Vector3<f32>,
    direction: Vector3<f32>,
}

impl Ray {
    pub fn new(origin: Vector3<f32>, direction: Vector3<f32>) -> Ray {
        Ray { origin, direction }
    }

    pub fn point_from_t(&self, t: f32) -> Vector3<f32> {
        self.origin + t * self.direction
    }
}

pub struct BoundBox {
    min: Vector3<f32>,
    max: Vector3<f32>,
}

impl BoundBox {
    pub fn new() -> BoundBox {
        BoundBox {
            min: vector![0.0, 0.0, 0.0],
            max: vector![1.0, 1.0, 1.0],
        }
    }

    pub fn intersect(&self, ray: &Ray) -> Option<(f32, f32)> {
        // t value of intersection with 6 planes of bounding box
        let t0x = (self.min.x - ray.origin.x) / ray.direction.x;
        let t1x = (self.max.x - ray.origin.x) / ray.direction.x;
        let t0y = (self.min.y - ray.origin.y) / ray.direction.y;
        let t1y = (self.max.y - ray.origin.y) / ray.direction.y;
        let t0z = (self.min.z - ray.origin.z) / ray.direction.z;
        let t1z = (self.max.z - ray.origin.z) / ray.direction.z;

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
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn intersect_works() {
        let bbox = BoundBox::new();
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
        let bbox = BoundBox::new();
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
        let bbox = BoundBox::new();
        let ray = Ray {
            origin: vector![2.0, 2.0, 2.0],
            direction: vector![1.0, 1.0, 8.0],
        };

        assert!(bbox.intersect(&ray).is_none());
    }
}
