mod compositor_worker;
mod messages;
mod master_thread;
mod render_worker;

pub use master_thread::ParalelRenderer;

mod workers {
    pub use super::compositor_worker::CompositorWorker;
    pub use super::render_worker::RenderWorker;
}
