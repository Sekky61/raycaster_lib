use nalgebra::{Vector2, Vector3};
use raycaster_lib::{
    premade::{
        parse::{generator_parser, skull_parser},
        transfer_functions,
    },
    volumetric::{DataSource, VolumeMetadata},
    TF,
};

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

/// List of parsers user can choose
#[derive(Clone, Copy)]
pub enum PrewrittenParser {
    /// Parser for the custom volume format generated by vol_gen app
    MyVolParser,
    /// Parser for the Skull example volume
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

    /// Get index in GUI
    pub fn get_gui_index(&self) -> i32 {
        // todo typedef return type
        match self {
            PrewrittenParser::MyVolParser => 0,
            PrewrittenParser::SkullParser => 1,
        }
    }
}

/// List of transfer functions user can choose
#[derive(Clone, Copy)]
pub enum PrewrittenTF {
    Skull,
    Gray,
    White,
    Shapes,
}

impl PrewrittenTF {
    /// Mapping from enum variant to the actual transfer function
    /// Returns function pointer
    pub fn get_tf(&self) -> TF {
        match self {
            PrewrittenTF::Skull => transfer_functions::skull_tf,
            PrewrittenTF::Gray => transfer_functions::anything_tf,
            PrewrittenTF::White => transfer_functions::white_tf,
            PrewrittenTF::Shapes => transfer_functions::shapes_tf,
        }
    }

    /// Mapping from enum variant to string with name.
    /// To update GUI while initializing.
    pub fn get_name(&self) -> &'static str {
        match self {
            PrewrittenTF::Skull => "Skull",
            PrewrittenTF::Gray => "Gray",
            PrewrittenTF::White => "White",
            PrewrittenTF::Shapes => "Shapes",
        }
    }
}
