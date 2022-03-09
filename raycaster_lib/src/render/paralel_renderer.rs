use crossbeam_channel::{Receiver, Sender};
use rayon::ThreadPool;

use crate::{camera::PerspectiveCamera, volumetric::BlockVolume};

use super::RenderOptions;

pub struct RenderJob {
    start_pixel: (usize, usize),
    resolution: (usize, usize),
    visibility: Vec<u8>,
}

pub struct SubRenderResult {
    start_index: usize,
    width: usize,
    data: Vec<u8>,
}

pub struct ParalelRenderer {
    volume: BlockVolume,
    camera: PerspectiveCamera,
    render_options: RenderOptions,
    compositors: ThreadPool,
    renderers: ThreadPool,
    channel: (Sender<SubRenderResult>, Receiver<SubRenderResult>),
}

impl ParalelRenderer {
    pub fn new(
        volume: BlockVolume,
        camera: PerspectiveCamera,
        render_options: RenderOptions,
    ) -> ParalelRenderer {
        let compositors = rayon::ThreadPoolBuilder::new()
            .num_threads(8)
            .build()
            .unwrap();
        let renderers = rayon::ThreadPoolBuilder::new()
            .num_threads(8)
            .build()
            .unwrap();
        let channel = crossbeam_channel::unbounded();
        ParalelRenderer {
            volume,
            camera,
            render_options,
            compositors,
            renderers,
            channel,
        }
    }

    pub fn render(&mut self, buffer: &mut [u8]) {

        // Start compositors

        // Start renderers
    }
}
