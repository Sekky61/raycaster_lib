use nalgebra::{vector, Point3, Rotation3, Vector2, Vector3};

use crate::common::{BoundBox, Ray, ViewportBox};

pub struct PerspectiveCamera {
    position: Point3<f32>,
    up: Vector3<f32>, // up vector = 0,1,0
    right: Vector3<f32>,
    direction: Vector3<f32>,
    aspect: f32,
    fov_y: f32,                   // Vertical field of view, in degrees
    img_plane_size: Vector2<f32>, // Calculated from fov_y
    // ray
    dir_00: Vector3<f32>, // Vector from camera point to pixel [0,0] | upper left corner, in line with buffer convention
    du: Vector3<f32>, // Vector between two horizontally neighbouring pixels (example: [0,0] -> [1,0])
    dv: Vector3<f32>, // Vector between two vertically neighbouring pixels (example: [0,0] -> [0,1])
}

impl PerspectiveCamera {
    pub fn new(position: Point3<f32>, direction: Vector3<f32>) -> PerspectiveCamera {
        let up = vector![0.0, 1.0, 0.0];
        let direction = direction.normalize();

        let right = direction.cross(&up); // todo normalize and unit?
        let up = right.cross(&direction);

        let fov_y = 60.0;
        let mut img_plane_size = vector![0.0, 2.0 * f32::tan(f32::to_radians(0.5 * fov_y))];
        img_plane_size.x = img_plane_size.y; // * aspect, but aspect is 1.0 right now

        let du: Vector3<f32> = img_plane_size.x * direction.cross(&up).normalize();
        let dv: Vector3<f32> = -img_plane_size.y * du.cross(&direction).normalize(); // negative, pointing downwards
        let dir_00 = direction - 0.5 * du - 0.5 * dv;
        PerspectiveCamera {
            position,
            up,
            right,
            direction,
            aspect: 1.0,
            fov_y,
            img_plane_size,
            dir_00,
            du,
            dv,
        }
    }

    pub fn get_dir(&self) -> Vector3<f32> {
        self.direction
    }

    // In degrees
    pub fn change_fov(&mut self, vertical_fov: f32) {
        self.fov_y = vertical_fov;
        self.recalc_plane_size();
        self.recalc_dudv();
    }

    // W/H ... for example 1.7777 for 16:9
    pub fn change_aspect(&mut self, aspect_ratio: f32) {
        self.aspect = aspect_ratio;
        self.recalc_plane_size();
        self.recalc_dudv();
    }

    pub fn set_pos(&mut self, pos: Point3<f32>) {
        self.position = pos;
    }

    pub fn set_direction(&mut self, direction: Vector3<f32>) {
        self.direction = direction.normalize();
        self.recalc_plane();
    }

    pub fn change_pos(&mut self, delta: Vector3<f32>) {
        self.position += delta;
    }

    pub fn change_pos_plane(&mut self, delta: Vector2<f32>) {
        self.position += delta.x * self.right + delta.y * self.up;
    }

    pub fn change_pos_view_dir(&mut self, delta: f32) {
        self.position += delta * self.direction;
    }

    pub fn look_around(&mut self, delta: Vector2<f32>) {
        self.direction += self.right * delta.x + self.up * delta.y;
        self.recalc_plane();
    }

    pub fn change_pos_matrix(&mut self, matrix: Rotation3<f32>) {
        self.position = matrix * self.position;
        self.direction = matrix * self.direction;

        self.recalc_plane();
    }

    // Call when camera direction changed
    fn recalc_plane(&mut self) {
        self.direction = self.direction.normalize();
        self.recalc_up_right();
        self.recalc_dudv();
    }

    // Call when camera direction changed
    fn recalc_up_right(&mut self) {
        let up = vector![0.0, 1.0, 0.0];
        self.right = self.direction.cross(&up); // todo normalize and unit?
        self.up = self.right.cross(&self.direction);
    }

    // Call when fov or aspect ratio changed
    fn recalc_plane_size(&mut self) {
        self.img_plane_size = vector![0.0, 2.0 * f32::tan(f32::to_radians(0.5 * self.fov_y))];
        self.img_plane_size.x = self.img_plane_size.y * self.aspect;
    }

    // Call when direction changed
    fn recalc_dudv(&mut self) {
        self.du = self.img_plane_size.x * self.direction.cross(&self.up).normalize();
        self.dv = -self.img_plane_size.y * self.du.cross(&self.direction).normalize(); // Notice '-' sign
        self.dir_00 = self.direction - 0.5 * self.du - 0.5 * self.dv;
    }

    pub fn get_ray(&self, pixel_coord: (f32, f32)) -> Ray {
        let dir = self.dir_00 + self.du * pixel_coord.0 + self.dv * pixel_coord.1;
        let dir = dir.normalize();
        Ray::from_3(self.position, dir)
    }

    pub fn project_box(&self, bound_box: BoundBox) -> ViewportBox {
        let mut viewbox = ViewportBox::new();

        let dun = self.du.normalize() / self.img_plane_size.x;
        let dvn = self.dv.normalize() / self.img_plane_size.y;
        let neg_dir = -self.direction;

        for point in bound_box {
            let v = point - self.position;
            let n = v.normalize();
            let neg_n = -n;

            let den = neg_n.dot(&neg_dir);
            if den != 0.0 {
                let t = 1.0 / den;
                let screen_dir = n * t - self.dir_00;
                let x = screen_dir.dot(&dun);
                let y = screen_dir.dot(&dvn);
                viewbox.add_point(x, y);
            }
        }

        viewbox
    }

    // TODO is lower corner enough for relative distances? Assuming blocks have the same size
    pub fn box_distance(&self, bound_box: &BoundBox) -> f32 {
        let center = bound_box.lower + 0.5 * (bound_box.upper - bound_box.lower);
        (center - self.position).magnitude()
    }
}

#[cfg(test)]
mod test {

    use nalgebra::point;

    use super::*;

    fn compare_float(actual: f32, expected: f32, error: f32) {
        let err = f32::abs(actual - expected);
        assert!(err < error * f32::EPSILON);
    }

    #[test]
    fn camera_du_dv() {
        let cam_pos = point![0.0, 0.0, 0.0];
        let cam_target = point![1.0, 0.0, 0.0];
        let cam = PerspectiveCamera::new(cam_pos, cam_target - cam_pos);

        assert_eq!(cam.right, vector![0.0, 0.0, 1.0]);
        assert_eq!(cam.up, vector![0.0, 1.0, 0.0]);

        assert_eq!(cam.du.normalize(), vector![0.0, 0.0, 1.0]);
        assert_eq!(cam.dv.normalize(), vector![0.0, -1.0, 0.0]);

        assert_eq!(cam.du.z, cam.img_plane_size.x);
        assert_eq!(cam.dv.y, -cam.img_plane_size.y); // notice '-' sign, dv points down
    }

    #[test]
    fn project_origin() {
        let origin = point![0.0, 0.0, 0.0];

        let cam_pos = point![-10.0, 7.7, -9.6];
        let cam_target = origin;
        let cam_dir = cam_target - cam_pos;
        let cam = PerspectiveCamera::new(cam_pos, cam_dir);

        assert_eq!(cam.direction, cam_dir.normalize());

        let origin_bbox = BoundBox::new(origin, origin);

        assert_eq!(origin_bbox.lower, point![0.0, 0.0, 0.0]);
        assert_eq!(origin_bbox.upper, point![0.0, 0.0, 0.0]);

        let projection = cam.project_box(origin_bbox);

        compare_float(projection.lower.x, 0.5, 4.0);
        compare_float(projection.lower.y, 0.5, 4.0);

        compare_float(projection.upper.x, 0.5, 4.0);
        compare_float(projection.upper.y, 0.5, 4.0);
    }

    #[test]
    fn project_corner() {
        let cam_pos = point![-10.0, 0.0, 0.0];
        let cam_target = point![0.0, 0.0, 0.0];
        let cam = PerspectiveCamera::new(cam_pos, cam_target - cam_pos);

        assert_eq!(cam.direction.normalize(), vector![1.0, 0.0, 0.0]);

        // Viewing angle of 60deg, 30deg from the center
        let top = 10.0 * f32::sqrt(3.0) / 3.0;

        let point = point![0.0, top, top];

        let bbox = BoundBox::new(point, point);

        let projection = cam.project_box(bbox);

        compare_float(projection.lower.x, 1.0, 4.0);
        compare_float(projection.lower.y, 0.0, 4.0);

        compare_float(projection.upper.x, 1.0, 4.0);
        compare_float(projection.upper.y, 0.0, 4.0);
    }

    #[test]
    fn box_distance() {
        let cam_pos = point![-1.0, 0.5, 0.5];
        let cam_target = point![0.0, 0.0, 0.0];
        let cam = PerspectiveCamera::new(cam_pos, cam_target - cam_pos);

        let lower = point![0.0, 0.0, 0.0];
        let upper = point![1.0, 1.0, 1.0];

        let origin_bbox = BoundBox::new(lower, upper);

        let distance = cam.box_distance(&origin_bbox);

        dbg!(distance);

        assert!((distance - 1.5).abs() < f32::EPSILON);
    }
}
