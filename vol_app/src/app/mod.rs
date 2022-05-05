/*
    vol_app
    Author: Michal Majer
    Date: 2022-05-05
*/

//! App state and gui callbacks

pub mod common;
pub mod defaults;
mod render_state;
mod state;
mod state_ref;
pub use render_state::RenderState;
pub use state::State;
pub use state_ref::StateRef;
