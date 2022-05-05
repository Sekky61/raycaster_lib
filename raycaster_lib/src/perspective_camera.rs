/*
    raycaster_lib
    Author: Michal Majer
    Date: 2022-05-05
*/

use nalgebra::{vector, Point3, Rotation3, Vector2, Vector3};

use crate::common::{BoundBox, Ray, ViewportBox};

/// Ray-casting camera
#[derive(Clone)]
pub struct PerspectiveCamera {
    /// Position of the camera in world coordinates
    position: Point3<f32>,
    /// Up direction from the camera's perspective
    up: Vector3<f32>, // up vector = 0,1,0
    /// Right direction from the camera's perspective
    right: Vector3<f32>,
    /// Direction of camera
    direction: Vector3<f32>, // todo unit?
    /// Aspect ratio of image plane
    aspect: f32,
    /// Vertical Field of View in degrees
    fov_y: f32,
    /// Size of image plane
    img_plane_size: Vector2<f32>, // Calculated from fov_y
    /// Direction of ray passing through pixel \[0,0\]
    dir_00: Vector3<f32>, // Vector from camera point to pixel \[0,0\] | upper left corner, in line with buffer convention
    /// Vector offset between two horizontally neighbouring pixels (such as: \[0,0\] -> \[1,0\])
    du: Vector3<f32>,
    /// Vector offset between two vertically neighbouring pixels (such as: \[0,0\] -> \[0,1\])
    dv: Vector3<f32>,
}

impl PerspectiveCamera {
    /// Construct new camera
    ///
    /// # Arguments
    ///
    /// * `position` - Position of the camera in world coordinates
    /// * `direction` - Looking direction of the camera
    ///
    /// # Notes
    ///
    /// The up direction is assumed to be 'up' (positive y axis)
    ///
    /// Default fov is 60 degrees, default aspect ratio is 1. To change it,
    /// call [`change_aspect_from_resolution`](PerspectiveCamera::change_aspect_from_resolution),
    /// [`change_fov`](PerspectiveCamera::change_fov), [`change_aspect`](PerspectiveCamera::change_aspect)
    pub fn new(position: Point3<f32>, direction: Vector3<f32>) -> PerspectiveCamera {
        // todo init with resolution?
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

    /// Changes aspect ratio to match `(width, height)` resolution
    ///
    /// # Example
    ///
    /// ```
    /// use raycaster_lib::PerspectiveCamera;
    /// use nalgebra::{vector, point};
    ///
    /// let position = point![0.0,0.0,0.0];
    /// let direction = vector![1.0,0.0,0.0];
    /// let mut camera = PerspectiveCamera::new(position, direction);
    ///
    /// let width = 1280;
    /// let height = 720;
    ///
    /// camera.change_aspect_from_resolution(width, height);
    /// // has the same effect as
    /// let fov = (width as f32) / (height as f32);
    /// camera.change_aspect(fov);
    /// ```
    pub fn change_aspect_from_resolution(&mut self, width: u32, height: u32) {
        let fov = (width as f32) / (height as f32);
        self.change_aspect(fov);
    }

    /// Change vertical FoV of camera
    ///
    /// # Arguments
    ///
    /// * `vertical_fov_deg` - vertical FoV in degrees
    pub fn change_fov(&mut self, vertical_fov_deg: f32) {
        assert!(vertical_fov_deg > 0.0 && vertical_fov_deg < 180.0);
        self.fov_y = vertical_fov_deg;
        self.recalc_plane_size();
        self.recalc_dudv();
    }

    /// Change aspect ratio of camera
    ///
    /// For example 1.7777 for 16:9 ratio
    pub fn change_aspect(&mut self, aspect_ratio: f32) {
        self.aspect = aspect_ratio;
        self.recalc_plane_size();
        self.recalc_dudv();
    }

    /// Set new position of camera
    pub fn set_pos(&mut self, pos: Point3<f32>) {
        self.position = pos;
    }

    /// Set new direction of camera
    pub fn set_direction(&mut self, direction: Vector3<f32>) {
        self.direction = direction.normalize();
        self.recalc_plane();
    }

    /// Move camera by vector `delta`
    pub fn change_pos(&mut self, delta: Vector3<f32>) {
        self.position += delta;
    }

    /// Move camera on the plane defined by camera position and direction (as a normal to the plane)
    ///
    /// Can also be thought of as the plane defined by right and up camera vectors
    ///
    /// # Arguments
    ///
    /// * `delta` - move on the plane with base vectors being the right and up vector of the camera
    pub fn change_pos_plane(&mut self, delta: Vector2<f32>) {
        self.position += delta.x * self.right + delta.y * self.up;
    }

    /// Move camera on the line defined by camera position and direction (as a direction of the line)
    pub fn change_pos_view_dir(&mut self, delta: f32) {
        self.position += delta * self.direction;
    }

    /// Change direction of the camera
    ///
    /// Positive `delta.x` means look to the right.
    /// Positive `delta.y` means look up.
    pub fn look_around(&mut self, delta: Vector2<f32>) {
        self.direction += self.right * delta.x + self.up * delta.y;
        self.recalc_plane();
    }

    /// Apply rotation matrix to the camera
    /// This changes both position and direction
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

    /// Get ray originating in the camera position crossing view plane in coordinates `pixel_coord`
    ///
    /// # Arguments
    ///
    /// * pixel_coord - Coordinates in the range of `<0;1>x<0;1>`, point \[0,0\] being upper left corner
    pub fn get_ray(&self, pixel_coord: (f32, f32)) -> Ray {
        let dir = self.dir_00 + self.du * pixel_coord.0 + self.dv * pixel_coord.1;
        let dir = dir.normalize();
        Ray::new(self.position, dir)
    }

    /// Project bounding box of a volume to viewport
    ///
    /// Resulting viewport box is the minimal orthogonal rectangular projection
    pub fn project_box(&self, bound_box: BoundBox) -> ViewportBox {
        // Source: https://github.com/ospray/ospray, Intel corp., Apache 2.0 license
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

        viewbox // todo join with pixel mapping
    }

    /// Get the distance from camera origin to the middle of a bound box
    pub fn box_distance(&self, bound_box: &BoundBox) -> f32 {
        // Assuming blocks have the same size, lower corners can also be relatively compared
        let center = bound_box.lower + 0.5 * (bound_box.upper - bound_box.lower);
        (center - self.position).magnitude()
    }

    /// Direction getter
    pub fn get_dir(&self) -> Vector3<f32> {
        self.direction
    }

    /// Position getter
    pub fn get_pos(&self) -> Point3<f32> {
        self.position
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
