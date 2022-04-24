//! # Default values
//!
//! Used as initial values.

use super::{
    common::{PrewrittenParser, PrewrittenTF},
    render_state::{PickedMemoryType, RenderQualitySettings},
};
use nalgebra::{point, vector, Point3, Vector2, Vector3};
use raycaster_lib::volumetric::MemoryType;

// Render resolution
pub const RENDER_WIDTH: u16 = 700; // todo use constants properly -- as defaults
pub const RENDER_HEIGHT: u16 = 700;
pub const RENDER_RESOLUTION: Vector2<u16> = vector![RENDER_WIDTH, RENDER_HEIGHT];

// Volume
// todo may not lead to a file
pub const VOLUME_PATH: &str = "volumes/Skull.vol";
// "volumes/Skull.vol" "volumes/a.vol" "volumes/800shapes_block16.vol" "volumes/solid_blocks_32.vol" "volumes/100_solid.vol"
pub const VOLUME_PARSER: PrewrittenParser = PrewrittenParser::SkullParser;

pub const MULTI_THREAD: bool = false;
pub const ERT: bool = true;
pub const EI: bool = true;

/// Block side used when constructing blocks in memory.
/// Does not affect volumes saved in files by blocks.
pub const BLOCK_SIDE: usize = 32;

pub const MEMORY_TYPE: PickedMemoryType = PickedMemoryType::Ram;
pub const TRANSFER_FUNCTION: PrewrittenTF = PrewrittenTF::Shapes;

pub const RENDER_QUALITY: RenderQualitySettings = RenderQualitySettings::FastOnMovement;

pub const RAY_STEP_FAST: f32 = 1.0;
pub const RAY_STEP_QUALITY: f32 = 0.2;

// Camera
// Ugly until https://github.com/rust-lang/rust/issues/57241 lands
pub const CAM_POS_X: f32 = 2600.0;
pub const CAM_POS_Y: f32 = 2600.0;
pub const CAM_POS_Z: f32 = 2600.0;

pub const CAM_POS: Point3<f32> = point![CAM_POS_X, CAM_POS_Y, CAM_POS_Z];
pub const CAM_DIR: Vector3<f32> = vector![0.0 - CAM_POS_X, 0.0 - CAM_POS_Y, 0.0 - CAM_POS_Z];
