use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};

use parking_lot::RwLock;

use raycaster_lib::{
    premade::parse::from_file,
    render::{ParalelRenderer, RenderOptions, RendererFront, RendererMessage, SerialRenderer},
    volumetric::{volumes::*, Blocked, BuildVolume, DataSource, Volume},
    ParserFn, PerspectiveCamera, TF,
};

use super::{
    common::{CameraMovement, PrewrittenParser, PrewrittenTF},
    defaults,
};

/// Queue of not yet applied camera movements
pub struct CameraBuffer {
    // todo instead of buffering, keep copy of camera, manipulate it and switch them
    buffer: VecDeque<CameraMovement>,
}

impl CameraBuffer {
    /// New empty queue
    pub fn new() -> Self {
        let buffer = VecDeque::new();
        Self { buffer }
    }

    /// Add new movement to queue
    /// Camera will be moved inbetween frames
    pub fn add_movement(&mut self, movement: CameraMovement) {
        self.buffer.push_back(movement);
    }

    /// Returns `true` if buffer is empty
    fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

/// Quality of rendered image
/// At the moment, the difference is sampling step
#[derive(Debug, Clone, Copy)]
pub enum RenderQuality {
    /// Better quality, shorter step
    Quality,
    /// Lower quality, longer step
    Fast,
}

/// Render quality preferences
pub enum RenderQualitySettings {
    /// Always render at the `RenderQuality::Quality` setting
    AlwaysQuality,
    /// Always render at the `RenderQuality::Fast` setting
    AlwaysFast,
    /// Use `Fast` when moving camera, `Quality` otherwise
    FastOnMovement,
}

/// State of rendering
///
/// Applies changes to camera, spawns and controls rendering
pub struct RenderState {
    pub renderer_front: RendererFront,
    pub render_options: RenderOptions,
    pub camera_buffer: CameraBuffer,
    pub render_quality_preference: RenderQualitySettings,
    pub current_tf: PrewrittenTF,
    pub render_time: Instant,
    pub is_rendering: bool,
    pub multi_thread: bool,
    pub stream_volume: bool,
    pub current_frame_quality: RenderQuality,
}

impl RenderState {
    /// Construct new `RenderState`
    ///
    /// Uses constants from `defaults`
    pub fn new() -> Self {
        let render_options = RenderOptions::builder()
            .early_ray_termination(defaults::ERT)
            .empty_space_skipping(defaults::EI)
            .resolution(defaults::RENDER_RESOLUTION)
            .build_unchecked();

        Self {
            renderer_front: RendererFront::new(),
            is_rendering: false,
            camera_buffer: CameraBuffer::new(),
            render_options,
            render_quality_preference: defaults::RENDER_QUALITY,
            multi_thread: defaults::MULTI_THREAD,
            stream_volume: defaults::STREAM,
            render_time: Instant::now(),
            current_tf: defaults::TRANSFER_FUNCTION,
            current_frame_quality: RenderQuality::Quality,
        }
    }

    /// Apply all buffered camera movements
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

    /// If conditions are met, start rendering new frame
    pub fn check_render_conditions(&mut self) {
        if self.is_rendering {
            return;
        }

        let quality = if self.camera_buffer.is_empty() {
            match self.render_quality_preference {
                RenderQualitySettings::FastOnMovement => match self.current_frame_quality {
                    RenderQuality::Quality => return,
                    RenderQuality::Fast => RenderQuality::Quality,
                },
                _ => return,
            }
        } else {
            match self.render_quality_preference {
                RenderQualitySettings::AlwaysQuality => RenderQuality::Quality,
                RenderQualitySettings::AlwaysFast => RenderQuality::Fast,
                RenderQualitySettings::FastOnMovement => RenderQuality::Fast,
            }
        };

        self.apply_cam_change();
        self.current_frame_quality = quality;
        self.start_render_frame(quality);
    }

    /// Instruct renderer to start rendering next frame
    ///
    /// # Params
    /// * `quality` - quality of render
    fn start_render_frame(&mut self, quality: RenderQuality) {
        self.is_rendering = true;
        let msg = match quality {
            RenderQuality::Quality => RendererMessage::StartRendering,
            RenderQuality::Fast => RendererMessage::StartRenderingFast,
        };

        println!("Starting render - quality {quality:?}");
        self.renderer_front.send_message(msg);
        self.render_time = Instant::now();
    }

    /// Initialize renderer
    ///
    /// # Params
    /// * `path` - path of file to parse
    /// * `parser` - parser to use
    pub fn start_renderer(&mut self, path: &Path, parser: PrewrittenParser) {
        print!(
            "GUI: starting renderer: MT {} | ERT {} | EI {} | ",
            self.multi_thread,
            self.render_options.early_ray_termination,
            self.render_options.empty_space_skipping
        );
        match (self.stream_volume, self.multi_thread) {
            (true, true) => {
                println!("StreamBlockVolume");
                let renderer = volume_setup_paralel::<StreamBlockVolume>(
                    &path,
                    parser,
                    self.render_options,
                    self.current_tf,
                ); // todo tf redundant
                self.renderer_front.start_rendering(renderer);
            }
            (false, true) => {
                println!("BlockVolume");
                let renderer = volume_setup_paralel::<BlockVolume>(
                    &path,
                    parser,
                    self.render_options,
                    self.current_tf,
                );
                self.renderer_front.start_rendering(renderer);
            }
            (true, false) => {
                println!("StreamVolume");
                let renderer = volume_setup_linear::<StreamVolume>(
                    &path,
                    parser,
                    self.render_options,
                    self.current_tf,
                );
                self.renderer_front.start_rendering(renderer);
            }
            (false, false) => {
                println!("LinearVolume");
                let renderer = volume_setup_linear::<LinearVolume>(
                    &path,
                    parser,
                    self.render_options,
                    self.current_tf,
                );
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

    /// Put `movement` to buffer
    pub fn register_movement(&mut self, movement: CameraMovement) {
        self.camera_buffer.add_movement(movement);
        self.check_render_conditions();
    }
}

/// Setup parallel renderer
fn volume_setup_paralel<V>(
    path: &Path,
    parser: PrewrittenParser,
    render_options: RenderOptions,
    tf: PrewrittenTF,
) -> ParalelRenderer<V>
where
    V: Volume + Blocked + BuildVolume<u8> + 'static,
{
    let (camera, parser_fn, tf_fn) = construct_common(parser, tf);

    // Example of custom parsing on client side
    // If volume is not blocked, build blocks in memory
    let parser_add_block_side = move |src: DataSource<u8>| {
        let mut res = parser_fn(src);
        match &mut res {
            Ok(ref mut m) => {
                if m.block_side.is_none() {
                    m.block_side = Some(defaults::BLOCK_SIDE);
                }
            }
            Err(_) => (),
        }
        res
    };

    let volume: V = from_file(path, parser_add_block_side, tf_fn).unwrap();

    ParalelRenderer::new(volume, camera, render_options)
}

/// Setup linear renderer
fn volume_setup_linear<V>(
    path: &Path,
    parser: PrewrittenParser,
    render_options: RenderOptions,
    tf: PrewrittenTF,
) -> SerialRenderer<V>
where
    V: Volume + BuildVolume<u8>,
{
    let (camera, parser_fn, tf_fn) = construct_common(parser, tf);

    let volume = from_file(path, parser_fn, tf_fn).unwrap();

    SerialRenderer::new(volume, camera, render_options)
}

fn construct_common(
    parser: PrewrittenParser,
    tf: PrewrittenTF,
) -> (Arc<RwLock<PerspectiveCamera>>, ParserFn, TF) {
    let position = defaults::CAM_POS;
    let direction = defaults::CAM_DIR;

    let parser_fn = parser.get_parser_fn();
    let tf_fn = tf.get_tf();

    let camera = PerspectiveCamera::new(position, direction);
    let camera = Arc::new(RwLock::new(camera));

    (camera, parser_fn, tf_fn)
}
