use std::{
    cell::RefCell,
    collections::VecDeque,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
    time::Instant,
};

use crossbeam_channel::Receiver;
use nalgebra::{point, vector, Point3, Vector2, Vector3};
use parking_lot::{Mutex, RwLock};
use raycaster_lib::{
    premade::{
        parse::{from_file, generator_parser, skull_parser},
        transfer_functions,
    },
    render::{ParalelRenderer, RenderOptions, RendererFront, RendererMessage, SerialRenderer},
    volumetric::{volumes::*, Blocked, BuildVolume, DataSource, Volume, VolumeMetadata},
    PerspectiveCamera, TF,
};
use slint::{
    re_exports::{PointerEvent, PointerEventButton, PointerEventKind},
    Weak,
};

use super::App;

// Default values

pub const RENDER_WIDTH: u16 = 700;
pub const RENDER_HEIGHT: u16 = 700;

// todo may not lead to a file
const DEFAULT_VOLUME_PATH: &str = "volumes/solid_blocks_32.vol"; // "volumes/Skull.vol" "volumes/a.vol" "volumes/solid_blocks_32.vol"
const DEFAULT_VOLUME_PARSER: PrewrittenParser = PrewrittenParser::MyVolParser;

const DEFAULT_MULTI_THREAD: bool = true;
const DEFAULT_BLOCK_SIDE: usize = 32;
const DEFAULT_STREAM: bool = true;
const DEFAULT_TF: PrewrittenTF = PrewrittenTF::Green;

const RAY_STEP_FAST: f32 = 1.0;
const RAY_STEP_QUALITY: f32 = 0.2;

// Ugly until https://github.com/rust-lang/rust/issues/57241 lands
const CAM_DEFAULT_POS_X: f32 = 300.0;
const CAM_DEFAULT_POS_Y: f32 = 300.0;
const CAM_DEFAULT_POS_Z: f32 = 300.0;

pub const CAM_DEFAULT_POS: Point3<f32> =
    point![CAM_DEFAULT_POS_X, CAM_DEFAULT_POS_Y, CAM_DEFAULT_POS_Z];
pub const CAM_DEFAULT_DIR: Vector3<f32> = vector![
    0.0 - CAM_DEFAULT_POS_X,
    0.0 - CAM_DEFAULT_POS_Y,
    0.0 - CAM_DEFAULT_POS_Z
];

/// Types of camera movement
pub enum CameraMovement {
    /// Move camera by `(x,y,z)`
    PositionOrtho(Vector3<f32>),
    /// Move camera on a plane orthogonal to viewing direction
    PositionPlane(Vector2<f32>),
    /// Rotate camera on `x` (right) and `y` (up) axis (from the camera's perspective)
    Direction(Vector2<f32>),
    /// Move camera in viewing direction
    PositionInDir(f32),
}

/// Queue of not yet applied camera movements
struct CameraBuffer {
    buffer: VecDeque<CameraMovement>,
}

impl CameraBuffer {
    /// New empty queue
    pub fn new() -> Self {
        let buffer = VecDeque::new();
        Self { buffer }
    }

    /// Add movement to queue
    pub fn add_movement(&mut self, movement: CameraMovement) {
        self.buffer.push_back(movement);
    }

    fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

/// List of parsers user can choose
enum PrewrittenParser {
    MyVolParser,
    SkullParser,
}

impl PrewrittenParser {
    /// Mapping from enum variant to the actual parser
    /// Returns function pointer
    pub fn get_parser_fn(&self) -> fn(DataSource<u8>) -> Result<VolumeMetadata<u8>, &'static str> {
        // todo typedef return type
        match self {
            PrewrittenParser::MyVolParser => generator_parser,
            PrewrittenParser::SkullParser => skull_parser,
        }
    }
}

#[derive(Clone, Copy)]
enum PrewrittenTF {
    Green,
    Gray,
    White,
}

impl PrewrittenTF {
    /// Mapping from enum variant to the actual transfer function
    /// Returns function pointer
    pub fn get_tf(&self) -> TF {
        match self {
            PrewrittenTF::Green => transfer_functions::skull_tf,
            PrewrittenTF::Gray => transfer_functions::anything_tf,
            PrewrittenTF::White => transfer_functions::white_tf,
        }
    }
}

#[derive(Debug)]
pub enum RenderQuality {
    Quality,
    Fast,
}

/// Render quality preferences
enum RenderQualitySettings {
    AlwaysQuality,
    AlwaysFast,
    FastOnMovement,
}

struct RenderState {
    pub renderer_front: RendererFront,
    pub render_options: RenderOptions,
    pub camera_buffer: CameraBuffer,
    pub render_quality_preference: RenderQualitySettings,
    pub current_tf: PrewrittenTF,
    pub render_time: Instant,
    pub is_rendering: bool,
    pub multi_thread: bool,
    pub stream_volume: bool,
}

impl RenderState {
    pub fn new() -> Self {
        let render_options = RenderOptions::builder()
            .early_ray_termination(true)
            .empty_space_skipping(true)
            .resolution(vector![RENDER_WIDTH, RENDER_HEIGHT])
            .build_unchecked();

        Self {
            renderer_front: RendererFront::new(),
            is_rendering: false,
            camera_buffer: CameraBuffer::new(),
            render_options,
            render_quality_preference: RenderQualitySettings::FastOnMovement,
            multi_thread: DEFAULT_MULTI_THREAD,
            stream_volume: DEFAULT_STREAM,
            render_time: Instant::now(),
            current_tf: DEFAULT_TF,
        }
    }

    pub fn apply_cam_change(&mut self) {
        /* todo chybí mousewheel
        // y        ... vertical scroll
                        // +1 unit  ... 1 step of wheel down (negative -> scroll up)

                        cam.change_pos_view_dir((*y as f32) * 5.0);
        */
        let camera = self.renderer_front.get_camera_handle().unwrap();
        {
            let mut camera = camera.write();
            while let Some(movement) = self.camera_buffer.buffer.pop_front() {
                match movement {
                    CameraMovement::PositionOrtho(d) => camera.change_pos(d),
                    CameraMovement::PositionPlane(d) => camera.change_pos_plane(d),
                    CameraMovement::Direction(d) => camera.look_around(d * 0.3),
                    CameraMovement::PositionInDir(d) => camera.change_pos_view_dir(d),
                }
            }
            // Drop Write camera guard
        }
    }

    pub fn check_render_conditions(&mut self) {
        if self.is_rendering || self.camera_buffer.is_empty() {
            return;
        }

        let rq = match self.render_quality_preference {
            RenderQualitySettings::AlwaysQuality => RenderQuality::Quality,
            RenderQualitySettings::AlwaysFast => RenderQuality::Fast,
            RenderQualitySettings::FastOnMovement => RenderQuality::Fast,
        };

        self.apply_cam_change();
        self.start_render(rq);
    }

    // Instruct renderer to start rendering next frame
    // todo rename
    fn start_render(&mut self, quality: RenderQuality) {
        self.is_rendering = true;
        let msg = match quality {
            RenderQuality::Quality => RendererMessage::StartRendering,
            RenderQuality::Fast => RendererMessage::StartRenderingFast,
        };

        println!("Starting render - quality {quality:?}");
        self.renderer_front.send_message(msg);
        self.render_time = Instant::now();
    }

    pub fn start_renderer(&mut self, path: PathBuf, parser: PrewrittenParser) {
        match (self.stream_volume, self.multi_thread) {
            (true, true) => {
                let renderer =
                    volume_setup_paralel::<StreamBlockVolume>(&path, parser, self.current_tf); // todo tf redundant
                self.renderer_front.start_rendering(renderer);
            }
            (false, true) => {
                let renderer = volume_setup_paralel::<BlockVolume>(&path, parser, self.current_tf);
                self.renderer_front.start_rendering(renderer);
            }
            (true, false) => {
                let renderer = volume_setup_linear::<StreamVolume>(&path, parser, self.current_tf);
                self.renderer_front.start_rendering(renderer);
            }
            (false, false) => {
                let renderer = volume_setup_linear::<LinearVolume>(&path, parser, self.current_tf);
                self.renderer_front.start_rendering(renderer);
            }
        }

        self.renderer_front
            .send_message(RendererMessage::StartRendering);
        println!(
            "Started renderer: {} | {path:#?}",
            if self.multi_thread { "MT" } else { "ST" }
        );
    }

    fn register_movement(&mut self, movement: CameraMovement) {
        self.camera_buffer.add_movement(movement);
        self.check_render_conditions();
    }
}

/// Application state
/// All fields are directly accessible
pub struct State {
    rendering: RenderState,
    // GUI
    app: Weak<App>,
    slider: Vector3<f32>,
    left_mouse_held: bool,
    right_mouse_held: bool,
    mouse: Option<Vector2<f32>>,
    // Vol picker
    file_picked: Option<PathBuf>,
    parser_picked: Option<PrewrittenParser>, // todo current parser to fix switching TF
    current_tf: PrewrittenTF,
}

impl State {
    /// Default values for state
    ///
    /// # Params
    ///
    /// * app - reference to GUI, see `App::as_weak`
    pub fn new(app: Weak<App>) -> State {
        State {
            app,
            left_mouse_held: false,
            right_mouse_held: false,
            mouse: None,
            slider: Default::default(),
            file_picked: None,
            parser_picked: None,
            current_tf: PrewrittenTF::Green,
            rendering: RenderState::new(),
        }
    }

    pub fn new_shared(app: Weak<App>) -> Rc<RefCell<State>> {
        let state = State::new(app);
        Rc::new(RefCell::new(state))
    }

    /// Get GUI handle
    ///
    /// Panics if called from a thread different than main thread
    pub fn get_app(&self) -> App {
        self.app.upgrade().unwrap()
    }

    pub fn set_file_picked(&mut self, file: PathBuf) {
        self.file_picked = Some(file);
    }

    pub fn get_buffer_handle(&self) -> &Arc<Mutex<Vec<u8>>> {
        self.rendering
            .renderer_front
            .get_buffer_handle_borrow()
            .unwrap()
    }

    pub fn slider_event(&mut self, slider_id: u8, slider: f32) {
        let delta = match slider_id {
            0 => {
                let res = vector![slider - self.slider.x, 0.0, 0.0];
                self.slider.x = slider;
                res
            }
            1 => {
                let res = vector![0.0, slider - self.slider.y, 0.0];
                self.slider.y = slider;
                res
            }
            2 => {
                let res = vector![0.0, 0.0, slider - self.slider.z];
                self.slider.z = slider;
                res
            }
            _ => panic!("Bad slider id, todo enum"),
        };
        self.rendering
            .register_movement(CameraMovement::PositionOrtho(delta));
    }

    pub fn handle_mouse_pos(&mut self, action: Vector2<f32>) {
        // rust-analyzer struggles here because m is of generated type
        // The type is (f32, f32)

        let drag_diff = if let Some(base) = self.mouse {
            action - base
        } else {
            self.mouse = Some(vector![action.x, action.y]);
            return;
        };

        self.mouse = Some(action);

        match (self.left_mouse_held, self.right_mouse_held) {
            (false, false) => (),
            (true, false) => {
                // move on the plane described by camera position and normal
                let delta = vector![drag_diff.x * -0.2, drag_diff.y * 0.2];
                self.rendering
                    .register_movement(CameraMovement::PositionPlane(delta));
            }
            (false, true) => {
                // change camera direction
                let delta = vector![drag_diff.x * 0.01, drag_diff.y * -0.01];
                self.rendering
                    .register_movement(CameraMovement::Direction(delta));
            }
            (true, true) => {
                // rotate around origin
                // TODO
                // let axisangle = Vector3::y() * (std::f32::consts::FRAC_PI_8 * drag_diff.0);
                // let rot = nalgebra::Rotation3::new(axisangle);

                // cam.change_pos_matrix(rot);
            }
        }
    }

    // todo pointer style move
    pub fn handle_pointer_event(&mut self, pe: PointerEvent) {
        self.mouse = None;
        match pe {
            PointerEvent {
                button: PointerEventButton::left,
                kind: PointerEventKind::up,
            } => self.left_mouse_held = false,
            PointerEvent {
                button: PointerEventButton::left,
                kind: PointerEventKind::down,
            } => self.left_mouse_held = true,
            PointerEvent {
                button: PointerEventButton::right,
                kind: PointerEventKind::up,
            } => self.right_mouse_held = false,
            PointerEvent {
                button: PointerEventButton::right,
                kind: PointerEventKind::down,
            } => self.right_mouse_held = true,
            _ => (),
        }
    }

    pub fn handle_key_press(&mut self, ch: char) {
        match ch {
            '+' => self
                .rendering
                .register_movement(CameraMovement::PositionInDir(5.0)),
            '-' => self
                .rendering
                .register_movement(CameraMovement::PositionInDir(-5.0)),
            _ => (),
        }
    }

    pub fn initial_render_call(&mut self) {
        self.rendering
            .start_renderer(DEFAULT_VOLUME_PATH.into(), DEFAULT_VOLUME_PARSER);
    }

    pub fn handle_open_vol(&mut self, parser_index: i32) {
        // Is file and parser picked?
        let path_picked = self.file_picked.is_some();
        let parser_picked = parser_index != -1;

        // Is parser selected?
        if !path_picked || !parser_picked {
            return;
        }

        // Take both
        let path = match self.file_picked.take() {
            Some(path) => path,
            None => return, // todo error
        };
        self.parser_picked = None;

        // Display new
        let parser = match parser_index {
            0 => PrewrittenParser::MyVolParser,
            1 => PrewrittenParser::SkullParser,
            _ => panic!("Unexpected parser"),
        };

        self.rendering.start_renderer(path, parser);
    }

    pub fn handle_tf_changed(&mut self, tf_name: &str) {
        let tf = match tf_name {
            "Green" => PrewrittenTF::Green,
            "Gray" => PrewrittenTF::Gray,
            "White" => PrewrittenTF::White,
            _ => panic!("Unknown transfer function '{tf_name}'"),
        };
        self.current_tf = tf;
        self.rendering
            .start_renderer(DEFAULT_VOLUME_PATH.into(), PrewrittenParser::SkullParser);
    }

    pub fn get_renderer_receiver(&self) -> Receiver<()> {
        self.rendering.renderer_front.get_receiver()
    }

    pub fn get_resolution(&self) -> Vector2<u16> {
        self.rendering.render_options.resolution
    }

    pub fn handle_rendering_finished(&mut self) {
        self.rendering.is_rendering = false;

        // Frame time counter
        let elapsed = self.rendering.render_time.elapsed();
        self.app.upgrade_in_event_loop(move |app| {
            app.set_frame_time(elapsed.as_millis().try_into().unwrap())
        });

        // Check missed inputs
        self.rendering.check_render_conditions();
    }

    pub fn set_mt(&mut self, multi_thread: bool) {
        self.rendering.multi_thread = multi_thread;
    }

    pub fn shutdown_renderer(&mut self) {
        self.rendering
            .renderer_front
            .send_message(RendererMessage::ShutDown);

        self.rendering.renderer_front.finish();
    }
}

fn volume_setup_paralel<V>(
    path: &Path,
    parser: PrewrittenParser,
    tf: PrewrittenTF,
) -> ParalelRenderer<V>
where
    V: Volume + Blocked + BuildVolume<u8> + 'static,
{
    let position = CAM_DEFAULT_POS;
    let direction = CAM_DEFAULT_DIR;

    let parser_fn = parser.get_parser_fn();

    let tf_fn = tf.get_tf();

    // Factor out
    let parser_add_block_side = move |src: DataSource<u8>| {
        let mut res = parser_fn(src);
        match &mut res {
            Ok(ref mut m) => {
                if m.block_side.is_none() {
                    m.block_side = Some(DEFAULT_BLOCK_SIDE);
                }
            }
            Err(_) => (),
        }
        res
    };

    let volume: V = from_file(path, parser_add_block_side, tf_fn).unwrap();

    let camera = PerspectiveCamera::new(position, direction);
    let camera = Arc::new(RwLock::new(camera));

    let render_options = RenderOptions::builder()
        .resolution(vector![RENDER_WIDTH, RENDER_HEIGHT])
        .early_ray_termination(true)
        .empty_space_skipping(true)
        .ray_step_quality(RAY_STEP_QUALITY)
        .ray_step_fast(RAY_STEP_FAST)
        .build_unchecked();

    ParalelRenderer::new(volume, camera, render_options)
}

fn volume_setup_linear<V>(
    path: &Path,
    parser: PrewrittenParser,
    tf: PrewrittenTF,
) -> SerialRenderer<V>
where
    V: Volume + BuildVolume<u8>,
{
    let position = point![300.0, 300.0, 300.0];
    let direction = point![34.0, 128.0, 128.0] - position; // vector![-0.8053911, -0.357536, -0.47277182]

    let parser_fn = parser.get_parser_fn();

    let tf_fn = tf.get_tf();

    let volume = from_file(path, parser_fn, tf_fn).unwrap();
    //let volume = from_file("volumes/a.vol", generator_parser, anything_tf).unwrap();

    let camera = PerspectiveCamera::new(position, direction);
    let camera = Arc::new(RwLock::new(camera));

    let render_options = RenderOptions::builder()
        .resolution(vector![RENDER_WIDTH, RENDER_HEIGHT])
        .early_ray_termination(true)
        .empty_space_skipping(true)
        .ray_step_quality(RAY_STEP_QUALITY)
        .ray_step_fast(RAY_STEP_FAST)
        .build_unchecked();

    SerialRenderer::new(volume, camera, render_options)
}
