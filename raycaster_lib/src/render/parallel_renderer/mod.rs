/*
    raycaster_lib
    Author: Michal Majer
    Date: 2022-05-05
*/

//! Parallel renderer module

mod communication;
mod composition;
mod master_thread;
mod messages;
mod render_worker;

pub use master_thread::ParalelRenderer;

mod workers {
    pub use super::composition::CompWorker;
    pub use super::render_worker::RenderWorker;
}
