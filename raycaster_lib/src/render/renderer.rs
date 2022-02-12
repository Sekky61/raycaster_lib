use nalgebra::{point, vector, Point3, Vector3, Vector4};

use crate::{
    camera::Camera,
    ray::Ray,
    transfer_functions::skull_tf,
    volumetric::{BlockType, EmptyIndex, EmptyIndexes, Volume},
};

#[derive(Default)]
pub struct RenderOptions {
    pub ray_termination: bool,
    pub empty_index: bool,
    pub multi_thread: bool,
}

impl RenderOptions {
    pub fn new(ray_termination: bool, empty_index: bool, multi_thread: bool) -> Self {
        Self {
            ray_termination,
            empty_index,
            multi_thread,
        }
    }
}

pub struct Position<'a> {
    pos: Point3<f32>,
    pos_int: Point3<usize>,
    level: usize,
    index_pos: Point3<usize>,
    index: &'a EmptyIndex,
}

impl<'a> Position<'a> {
    pub fn new(pos: Point3<f32>, level: usize, index: &'a EmptyIndex) -> Self {
        let mut position = Position {
            pos,
            pos_int: point![0, 0, 0],
            level,
            index_pos: point![0, 0, 0],
            index,
        };
        position.sync_pos();
        position
    }

    pub fn sync_pos(&mut self) {
        self.pos_int = self.pos.map(|f| f as usize);
        self.index_pos = EmptyIndexes::get_block_coords_int(self.level, &self.pos_int);
    }

    pub fn lower_level(&mut self, index_ref: &'a EmptyIndexes) {
        self.level -= 1;
        self.index = index_ref.get_index_ref(self.level);
        self.index_pos = EmptyIndexes::get_block_coords_int(self.level, &self.pos_int);
    }

    pub fn get_block_type(&self) -> BlockType {
        self.index.get_block_vec(&self.index_pos)
    }

    pub fn change_pos(&mut self, delta: &Vector3<f32>) {
        self.pos += delta;
        self.sync_pos();
    }
}

pub struct Renderer<V, C>
where
    V: Volume,
    C: Camera,
{
    pub volume: V,
    pub camera: C,
    pub empty_index: EmptyIndexes,
    render_options: RenderOptions,
}

impl<V, C> Renderer<V, C>
where
    V: Volume,
    C: Camera,
{
    pub fn new(volume: V, camera: C) -> Renderer<V, C> {
        let empty_index = EmptyIndexes::from_volume(&volume);
        Renderer {
            volume,
            camera,
            empty_index,
            render_options: RenderOptions {
                ray_termination: true,
                empty_index: false,
                multi_thread: false,
            },
        }
    }

    pub fn set_render_options(&mut self, opts: RenderOptions) {
        self.render_options = opts;
    }

    pub fn render_to_buffer(&mut self, buffer: &mut [u8]) {
        self.render(buffer);
    }

    fn render(&mut self, buffer: &mut [u8]) {
        // println!("THE RENDER");
        // println!("Vol: {:?}", self.volume.get_dims());
        // println!("Index: {:?}", self.empty_index);

        let (img_w, img_h) = self.camera.get_resolution();

        let (image_width, image_height) = (img_w as f32, img_h as f32);

        let origin_4 = self.camera.get_position().to_homogeneous();

        let aspect_ratio = image_width / image_height;

        // cam to world
        let lookat_matrix = self.camera.view_matrix();

        let mut buffer_index = 0;

        for y in 0..img_h {
            for x in 0..img_w {
                let pixel_ndc_x = (x as f32 + 0.5) / image_width;
                let pixel_ndc_y = (y as f32 + 0.5) / image_height;

                let pixel_screen_x = (pixel_ndc_x * 2.0 - 1.0) * aspect_ratio;
                let pixel_screen_y = 1.0 - pixel_ndc_y * 2.0; // v NDC Y roste dolu, obratime

                //todo FOV

                let pix_cam_space = vector![pixel_screen_x, pixel_screen_y, -1.0, 1.0];

                let dir_world = (lookat_matrix * pix_cam_space) - origin_4;
                let dir_world_3 = dir_world.xyz().normalize();

                let ray_world = Ray::from_3(self.camera.get_position(), dir_world_3);

                // performance: branch gets almost optimized away since it is predictable
                let ray_color = if self.render_options.empty_index {
                    self.collect_light_index(&ray_world)
                } else {
                    self.collect_light(&ray_world)
                };

                let opacity = ray_color.w;

                // expects black background
                buffer[buffer_index] = (ray_color.x * opacity) as u8;
                buffer[buffer_index + 1] = (ray_color.y * opacity) as u8;
                buffer[buffer_index + 2] = (ray_color.z * opacity) as u8;

                buffer_index += 3;
            }
        }
    }

    pub fn collect_light(&self, ray: &Ray) -> Vector4<f32> {
        let mut accum = vector![0.0, 0.0, 0.0, 0.0];

        let (t1, _) = match self.volume.intersect(ray) {
            Some(tup) => tup,
            None => return accum,
        };

        let begin = ray.point_from_t(t1);
        let direction = ray.get_direction();

        let step_size = 1.0;

        let step = direction * step_size; // normalized

        let mut pos = begin;

        loop {
            let sample = self.volume.sample_at(pos);

            let color = skull_tf(sample as u8);

            pos += step;

            if color.w == 0.0 {
                if !self.volume.is_in(&pos) {
                    break;
                }
                continue;
            }

            // pseudocode from https://scholarworks.rit.edu/cgi/viewcontent.cgi?article=6466&context=theses page 55, figure 5.6
            //sum = (1 - sum.alpha) * volume.density * color + sum;

            accum += (1.0 - accum.w) * color;

            // relying on branch predictor to "eliminate" branch
            if self.render_options.ray_termination {
                // early ray termination
                if (color.w - 0.99) > 0.0 {
                    break;
                }
            }

            if !self.volume.is_in(&pos) {
                break;
            }
        }

        accum
    }

    pub fn collect_light_index(&self, ray: &Ray) -> Vector4<f32> {
        let mut accum = vector![0.0, 0.0, 0.0, 0.0];

        let (t1, _) = match self.volume.intersect(ray) {
            Some(tup) => tup,
            None => return accum,
        };

        let begin = ray.point_from_t(t1);
        let direction = ray.get_direction();

        let step_size = 1.0;

        let step = direction * step_size; // normalized
        let ray_dirs = step.map(|v| if v.is_sign_positive() { 1usize } else { 0 });

        let m_max = self.empty_index.len() - 1;
        let starting_m = m_max - 1;

        let begin_index = self.empty_index.get_index_ref(starting_m);

        let mut position = Position::new(begin, starting_m, begin_index);

        let mut index = position.index.get_block_vec(&position.index_pos);

        // index edge
        let mut index_edge = EmptyIndexes::get_index_size(position.level);
        let mut index_edge_fl = index_edge as f32;

        loop {
            //println!("> m {} index {:?}", m, index);

            if index == BlockType::NonEmpty {
                if position.level > 0 {
                    // go down a level
                    position.lower_level(&self.empty_index);
                    index = position.get_block_type();

                    index_edge = EmptyIndexes::get_index_size(position.level);
                    index_edge_fl = index_edge as f32;

                    //println!("#5 pos set level {} us {}", m, index_coords);
                } else {
                    // m == 0
                    // sample
                    let sample = self.volume.sample_at(position.pos);

                    let color = skull_tf(sample as u8);

                    accum += (1.0 - accum.w) * color;

                    if self.render_options.ray_termination {
                        // early ray termination
                        if (color.w - 0.99) > 0.0 {
                            break;
                        }
                    }

                    position.change_pos(&step);

                    if !self.volume.is_in(&position.pos) {
                        break;
                    }

                    index = position.get_block_type();
                }
                continue;
            }

            // empty

            // step to next on same level

            let index_edge = EmptyIndexes::get_index_size(position.level); // todo recalculate only on level change
            let index_low_coords = position.index_pos * index_edge;
            let index_low_coords = index_low_coords.map(|v| v as f32);

            let delta_i = ray_dirs.map(|d| if d != 0 { index_edge_fl } else { 0.0 });

            let delta_i = (index_low_coords + delta_i - position.pos).component_div(&step);

            let delta_i = delta_i.map(|f| f.ceil());

            let n_of_steps = delta_i.min().max(1.0);
            let change = step * n_of_steps;

            position.change_pos(&change);

            if !self.volume.is_in(&position.pos) {
                break;
            }

            index = position.get_block_type();

            /*let new_pos = position.pos + step * (n_of_steps as f32);
            let new_index_coords = EmptyIndexes::get_block_coords(m, &new_pos);
            // println!("#2 new pos level {} us {}", m, new_index_coords);

            if !self.volume.is_in(new_pos) {
                break;
            }
            //println!("{} is in", new_pos);

            // parents
            if m < m_max - 1 {
                let parent_index = self.empty_index.get_parent_index(m, &new_index_coords);
                if parent_index == BlockType::Empty {
                    m += 1;

                    pos = new_pos;
                    index = parent_index;
                    index_coords = EmptyIndexes::get_block_coords(m, &pos);
                    // println!("#3 pos set level {} us {}", m, index_coords);
                    continue;
                }
            }

            pos = new_pos;
            //println!("#4 pos set level {} us {}", m, new_index_coords);
            index_coords = new_index_coords;
            index = self.empty_index.get_index_from_usize(m, &index_coords);*/
        }

        // let accum_i_x = accum.x as u8;
        // let accum_i_y = accum.y as u8;
        // let accum_i_z = accum.z as u8;

        accum
    }
}
