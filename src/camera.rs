use nalgebra::{matrix, vector, Matrix4, Vector3, Vector4};

use crate::volume::Volume;

pub struct Camera {
    position: Vector3<f32>,
    target: Vector3<f32>,
    resolution: (usize, usize),
}

impl Camera {
    pub fn new(width: usize, height: usize) -> Camera {
        Camera {
            position: vector![-4.0, 108.0, -85.0],
            target: vector![34.0, 128.0, 128.0], //target: vector![0.0, 0.0, 0.0],
            resolution: (width, height),
        }
    }

    pub fn change_pos(&mut self, delta: Vector3<f32>) {
        self.position += delta;
    }

    pub fn set_pos(&mut self, pos: Vector3<f32>) {
        self.position = pos;
    }

    pub fn get_resolution(&self) -> (usize, usize) {
        self.resolution
    }

    pub fn get_look_at_matrix(&self) -> Matrix4<f32> {
        let camera_forward = (self.position - self.target).normalize();
        let up_vec = vector![0.0, 1.0, 0.0];
        let right = Vector3::cross(&up_vec, &camera_forward);
        let up = Vector3::cross(&camera_forward, &right);

        // cam to world
        matrix![right.x, up.x, camera_forward.x, self.position.x;
                                                            right.y, up.y, camera_forward.y, self.position.y;
                                                            right.z,up.z,camera_forward.z, self.position.z;
                                                            0.0,0.0,0.0, 1.0]
    }

    pub fn cast_rays_bytes(&self, bbox: &BoundBox, buffer: &mut [u8]) {
        let (image_width, image_height) = (self.resolution.0 as f32, self.resolution.1 as f32);

        let origin = self.position;
        let origin_4 = Vector4::new(origin.x, origin.y, origin.z, 1.0);

        let aspect_ratio = image_width / image_height;

        // cam to world
        let lookat_matrix = self.get_look_at_matrix();

        for y in 0..self.resolution.1 {
            for x in 0..self.resolution.0 {
                let pixel_ndc_x = (x as f32 + 0.5) / image_width;
                let pixel_ndc_y = (y as f32 + 0.5) / image_height;

                let pixel_screen_x = (pixel_ndc_x * 2.0 - 1.0) * aspect_ratio;
                let pixel_screen_y = 1.0 - pixel_ndc_y * 2.0; // v NDC Y roste dolu, obratime

                //todo FOV

                let pix_cam_space = vector![pixel_screen_x, pixel_screen_y, -1.0, 1.0];

                let dir_world = (lookat_matrix * pix_cam_space) - origin_4;
                let dir_world_3 = dir_world.xyz().normalize();

                //println!("{}", dir_world_3);

                let ray_world = Ray::from_3(origin, dir_world_3);

                let ray_color = bbox.collect_light(&ray_world);

                let index = (y * self.resolution.0 + x) * 3; // packed structs -/-

                buffer[index] = ray_color.0;
                buffer[index + 1] = ray_color.1;
                buffer[index + 2] = ray_color.2;
            }
        }
    }
}

pub struct Ray {
    origin: Vector3<f32>,
    direction: Vector3<f32>,
}

impl Ray {
    pub fn from_3(origin: Vector3<f32>, direction: Vector3<f32>) -> Ray {
        Ray { origin, direction }
    }

    pub fn point_from_t(&self, t: f32) -> Vector3<f32> {
        self.origin + t * self.direction
    }

    pub fn get_direction(&self) -> Vector3<f32> {
        self.direction
    }
}

// R G B A -- A <0;1>
pub fn transfer_function(sample: f32) -> (f32, f32, f32, f32) {
    if sample > 4.0 {
        (20.0, 220.0, 20.0, 0.5)
    } else if sample > 3.0 {
        (220.0, 10.0, 10.0, 0.2)
    } else if sample > 2.0 {
        (50.0, 50.0, 10.0, 0.2)
    } else if sample > 1.0 {
        (10.0, 10.0, 40.0, 0.1)
    } else {
        (0.0, 0.0, 0.0, 0.0)
    }
}

pub struct BoundBox {
    min: Vector3<f32>,
    max: Vector3<f32>,
    volume: Volume,
}

impl std::fmt::Debug for BoundBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoundBox")
            .field("min", &self.min)
            .field("max", &self.max)
            .field("volume", &self.volume)
            .finish()
    }
}

impl BoundBox {
    pub fn from_volume(volume: Volume) -> BoundBox {
        BoundBox {
            min: vector![0.0, 0.0, 0.0],
            max: volume.get_dims(),
            volume,
        }
    }

    pub fn collect_light(&self, ray: &Ray) -> (u8, u8, u8) {
        //let mut color = vector![0.0, 0.0, 0.0];
        let mut accum = (0.0, 0.0, 0.0, 0.0);

        match self.intersect(ray) {
            Some((t1, t2)) => {
                let begin = ray.point_from_t(t1);
                let direction = ray.get_direction();

                let step_size = 1.0;
                //let steps = 64;
                let step = direction * step_size; // normalized

                let mut pos = begin;

                //let mut steps_count = 0;

                loop {
                    let sample = self.volume.sample_at(pos);

                    let color = transfer_function(sample);

                    pos += step;

                    if color.3 == 0.0 {
                        if !self.volume.is_in(pos) {
                            break;
                        }
                        continue;
                    }

                    // pseudocode from https://scholarworks.rit.edu/cgi/viewcontent.cgi?article=6466&context=theses page 55, figure 5.6
                    //sum = (1 - sum.alpha) * volume.density * color + sum;

                    accum.0 += (1.0 - accum.3) * color.0;
                    accum.1 += (1.0 - accum.3) * color.1;
                    accum.2 += (1.0 - accum.3) * color.2;
                    accum.3 += (1.0 - accum.3) * color.3;

                    // accum.0 += color.0 * color.3;
                    // accum.1 += color.1 * color.3;
                    // accum.2 += color.2 * color.3;

                    // early ray termination
                    if (color.3 - 1.0).abs() < f32::EPSILON {
                        break;
                    }

                    //accum.1 = (accum.1 as u8).saturating_add(color.1 / color.3);
                    //accum.2 = (accum.2 as u8).saturating_add(color.2 / color.3);
                    //accum.3 = (accum.3 as u8).saturating_add(color.3);

                    if !self.volume.is_in(pos) {
                        break;
                    }
                }

                let accum_i_x = accum.0.min(255.0) as u8;
                let accum_i_y = accum.1.min(255.0) as u8;
                let accum_i_z = accum.2.min(255.0) as u8;

                (accum_i_x, accum_i_y, accum_i_z)
            }
            None => (0, 0, 0),
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

    fn cube_bound_box() -> BoundBox {
        BoundBox {
            min: vector![0.0, 0.0, 0.0],
            max: vector![1.0, 1.0, 1.0],
            volume: Volume::white_vol(),
        }
    }

    #[test]
    fn intersect_works() {
        let bbox = cube_bound_box();
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
        let bbox = cube_bound_box();
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
        let bbox = cube_bound_box();
        let ray = Ray {
            origin: vector![2.0, 2.0, 2.0],
            direction: vector![1.0, 1.0, 8.0],
        };

        assert!(bbox.intersect(&ray).is_none());
    }
}
