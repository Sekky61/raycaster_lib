use super::*;

impl Renderer<LinearVolume> {
    pub fn render_settings(&mut self, options: RendererOptions) {
        self.render = match options {
            RendererOptions {
                ray_termination: true,
                empty_index: false,
                multi_thread: false,
            } => Self::render_linear_term_st,
            RendererOptions {
                ray_termination: false,
                empty_index: false,
                multi_thread: false,
            } => Self::render_linear_st,
            _ => panic!("Not implemented"),
        };
    }

    fn render_linear_st(&mut self) {}

    fn render_linear_term_st(&mut self) {
        let buffer = self.buffer.as_mut_slice();
        println!("RENDER BUM st term");
        let (image_width, image_height) = (
            self.camera.resolution.0 as f32,
            self.camera.resolution.1 as f32,
        );

        let origin_4 = Vector4::new(
            self.camera.position.x,
            self.camera.position.y,
            self.camera.position.z,
            1.0,
        );

        let aspect_ratio = image_width / image_height;

        // cam to world
        let lookat_matrix = self.camera.get_look_at_matrix();

        for y in 0..self.camera.resolution.1 {
            for x in 0..self.camera.resolution.0 {
                let pixel_ndc_x = (x as f32 + 0.5) / image_width;
                let pixel_ndc_y = (y as f32 + 0.5) / image_height;

                let pixel_screen_x = (pixel_ndc_x * 2.0 - 1.0) * aspect_ratio;
                let pixel_screen_y = 1.0 - pixel_ndc_y * 2.0; // v NDC Y roste dolu, obratime

                //todo FOV

                let pix_cam_space = vector![pixel_screen_x, pixel_screen_y, -1.0, 1.0];

                let dir_world = (lookat_matrix * pix_cam_space) - origin_4;
                let mut dir_world_3 = dir_world.xyz();
                dir_world_3.normalize_mut();

                //println!("{}", dir_world_3);

                let ray_world = Ray::from_3(self.camera.position, dir_world_3);

                let ray_color = self.volume.collect_light_term(&ray_world);

                let index = (y * self.camera.resolution.0 + x) * 3; // packed structs -/-

                buffer[index] = ray_color.0;
                buffer[index + 1] = ray_color.1;
                buffer[index + 2] = ray_color.2;
            }
        }
    }
}
