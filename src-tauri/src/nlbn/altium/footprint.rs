/// Altium Designer PCB footprint data structures
use serde::{Deserialize, Serialize};

/// Represents a complete PCB footprint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdFootprint {
    /// Footprint name (e.g., "QFN-48")
    pub name: String,
    /// Footprint description
    pub description: String,
    /// List of pads
    pub pads: Vec<AdPad>,
    /// List of lines (silkscreen, etc.)
    pub lines: Vec<AdLine>,
    /// List of arcs
    pub arcs: Vec<AdArc>,
    /// List of text elements
    pub texts: Vec<AdText>,
    /// Optional 3D model reference
    pub model_3d: Option<Ad3DModel>,
}

/// Represents a pad/land in the footprint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdPad {
    /// X coordinate in mils
    pub x: f64,
    /// Y coordinate in mils
    pub y: f64,
    /// Pad width in mils
    pub width: f64,
    /// Pad height in mils
    pub height: f64,
    /// Drill/hole size in mils (0 for SMD pads)
    pub hole_size: f64,
    /// Pad shape
    pub shape: PadShape,
    /// Pad number/designator (e.g., "1", "A1")
    pub name: String,
    /// Layer the pad is on
    pub layer: PadLayer,
    /// Rotation angle in degrees
    pub rotation: f64,
}

/// Pad shape types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PadShape {
    Round,      // Circular pad
    Rectangle,  // Rectangular pad
    Octagonal,  // Octagonal pad (8-sided)
    RoundRect,  // Rounded rectangle
}

impl PadShape {
    /// Convert to Altium shape code
    pub fn to_altium_code(&self) -> i32 {
        match self {
            PadShape::Round => 0,
            PadShape::Rectangle => 1,
            PadShape::Octagonal => 2,
            PadShape::RoundRect => 3,
        }
    }
}

/// Pad layer specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PadLayer {
    Top,        // Top copper layer only (SMD)
    Bottom,     // Bottom copper layer only (SMD)
    MultiLayer, // All copper layers (Through-hole)
}

impl PadLayer {
    /// Convert to Altium layer name
    pub fn to_altium_name(&self) -> &'static str {
        match self {
            PadLayer::Top => "TOP",
            PadLayer::Bottom => "BOTTOM",
            PadLayer::MultiLayer => "MULTILAYER",
        }
    }
}

/// Line element for silkscreen, assembly, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdLine {
    /// Start X coordinate in mils
    pub start_x: f64,
    /// Start Y coordinate in mils
    pub start_y: f64,
    /// End X coordinate in mils
    pub end_x: f64,
    /// End Y coordinate in mils
    pub end_y: f64,
    /// Line width in mils
    pub width: f64,
    /// Layer name (e.g., "TOPOVERLAY", "BOTTOMOVERLAY")
    pub layer: String,
}

/// Arc element for curved shapes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdArc {
    /// Center X coordinate in mils
    pub center_x: f64,
    /// Center Y coordinate in mils
    pub center_y: f64,
    /// Arc radius in mils
    pub radius: f64,
    /// Start angle in degrees (0-360)
    pub start_angle: f64,
    /// End angle in degrees (0-360)
    pub end_angle: f64,
    /// Line width in mils
    pub width: f64,
    /// Layer name
    pub layer: String,
}

/// Text element for reference designator, values, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdText {
    /// X coordinate in mils
    pub x: f64,
    /// Y coordinate in mils
    pub y: f64,
    /// Text content
    pub text: String,
    /// Font height in mils
    pub height: f64,
    /// Font width in mils
    pub width: f64,
    /// Rotation angle in degrees
    pub rotation: f64,
    /// Layer name
    pub layer: String,
}

/// 3D model reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ad3DModel {
    /// Model filename (e.g., "component.step")
    pub filename: String,
    /// Rotation around X axis in degrees
    pub rotation_x: f64,
    /// Rotation around Y axis in degrees
    pub rotation_y: f64,
    /// Rotation around Z axis in degrees
    pub rotation_z: f64,
    /// Offset along Z axis in mils
    pub offset_z: f64,
}
