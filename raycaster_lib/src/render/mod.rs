mod parallel_renderer;
mod render_front;
mod render_options;
mod renderer;
mod st_renderer;

pub use parallel_renderer::ParalelRenderer;
pub use render_front::{RenderThread, RendererFront, RendererMessage};
pub use render_options::RenderOptions;
pub use renderer::Renderer;
pub use st_renderer::SerialRenderer;

use crate::color::RGBA;
pub type TF = fn(f32) -> RGBA;
