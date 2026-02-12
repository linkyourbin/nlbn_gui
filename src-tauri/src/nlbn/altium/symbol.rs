/// Altium Designer schematic symbol data structures
use serde::{Deserialize, Serialize};

/// Represents a complete schematic symbol component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdSymbol {
    /// Component reference name (e.g., "STM32G431CBU6")
    pub libref: String,
    /// Component description
    pub description: String,
    /// List of component pins
    pub pins: Vec<AdPin>,
    /// List of rectangles (symbol body)
    pub rectangles: Vec<AdRectangle>,
    /// List of lines (symbol graphics)
    pub lines: Vec<AdLine>,
    /// List of text elements
    pub texts: Vec<AdText>,
}

/// Represents a pin in the schematic symbol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdPin {
    /// X coordinate in mils (1/1000 inch)
    pub x: i32,
    /// Y coordinate in mils
    pub y: i32,
    /// Pin length in mils
    pub length: i32,
    /// Pin name (e.g., "PA0", "VDD")
    pub name: String,
    /// Pin designator/number (e.g., "1", "A1")
    pub designator: String,
    /// Electrical type of the pin
    pub electrical: PinElectrical,
    /// Pin orientation/direction
    pub orientation: PinOrientation,
}

/// Pin electrical types supported by Altium Designer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PinElectrical {
    Input,          // Input pin
    IO,             // Input/Output pin
    Output,         // Output pin
    OpenCollector,  // Open collector
    Passive,        // Passive (no direction)
    HiZ,            // High impedance
    OpenEmitter,    // Open emitter
    Power,          // Power pin
}

impl PinElectrical {
    /// Convert to Altium electrical type code
    pub fn to_altium_code(&self) -> i32 {
        match self {
            PinElectrical::Input => 0,
            PinElectrical::IO => 1,
            PinElectrical::Output => 2,
            PinElectrical::OpenCollector => 3,
            PinElectrical::Passive => 4,
            PinElectrical::HiZ => 5,
            PinElectrical::OpenEmitter => 6,
            PinElectrical::Power => 7,
        }
    }
}

/// Pin orientation/direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PinOrientation {
    Right,  // Pin points to the right
    Left,   // Pin points to the left
    Up,     // Pin points upward
    Down,   // Pin points downward
}

impl PinOrientation {
    /// Convert to Altium pin conglomerate code
    pub fn to_altium_code(&self) -> i32 {
        match self {
            PinOrientation::Right => 0,
            PinOrientation::Up => 1,
            PinOrientation::Left => 2,
            PinOrientation::Down => 3,
        }
    }
}

/// Rectangle shape for symbol body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdRectangle {
    /// X coordinate in mils (bottom-left corner)
    pub x: i32,
    /// Y coordinate in mils (bottom-left corner)
    pub y: i32,
    /// Width in mils
    pub width: i32,
    /// Height in mils
    pub height: i32,
    /// Line color (RGB format: 0xRRGGBB)
    pub color: u32,
    /// Whether the rectangle is filled
    pub is_solid: bool,
}

/// Line shape for symbol graphics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdLine {
    /// Start X coordinate in mils
    pub start_x: i32,
    /// Start Y coordinate in mils
    pub start_y: i32,
    /// End X coordinate in mils
    pub end_x: i32,
    /// End Y coordinate in mils
    pub end_y: i32,
    /// Line width in mils
    pub width: i32,
    /// Line color (RGB format: 0xRRGGBB)
    pub color: u32,
}

/// Text element for labels and designators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdText {
    /// X coordinate in mils
    pub x: i32,
    /// Y coordinate in mils
    pub y: i32,
    /// Text content
    pub text: String,
    /// Font height in mils
    pub height: i32,
    /// Rotation angle in degrees (0, 90, 180, 270)
    pub rotation: f64,
}
