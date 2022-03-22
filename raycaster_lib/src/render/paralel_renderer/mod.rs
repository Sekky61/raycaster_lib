mod communication;
mod compositor_worker;
mod master_thread;
mod messages;
mod render_worker;

pub use master_thread::ParalelRenderer;

mod workers {
    pub use super::compositor_worker::CompositorWorker;
    pub use super::render_worker::RenderWorker;
}
