mod compositor_worker;
mod messages;
mod paralel_renderer;
mod render_worker;

pub use paralel_renderer::ParalelRenderer;

mod workers {
    pub use super::compositor_worker::CompositorWorker;
    pub use super::render_worker::RenderWorker;
}
