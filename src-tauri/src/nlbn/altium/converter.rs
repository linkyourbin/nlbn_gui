/// Converter from EasyEDA format to Altium Designer format
use crate::nlbn::easyeda::models::*;
use super::symbol::*;
use super::footprint::*;

/// Converts EasyEDA components to Altium Designer format
pub struct AltiumConverter;

impl AltiumConverter {
    pub fn new() -> Self {
        Self
    }

    /// Convert EasyEDA symbol to Altium symbol
    pub fn convert_symbol(&self, ee_symbol: &EeSymbol) -> AdSymbol {
        let mut ad_symbol = AdSymbol {
            libref: ee_symbol.name.clone(),
            description: format!("{} - Converted from EasyEDA", ee_symbol.name),
            pins: Vec::new(),
            rectangles: Vec::new(),
            lines: Vec::new(),
            texts: Vec::new(),
        };

        // Convert pins
        for ee_pin in &ee_symbol.pins {
            ad_symbol.pins.push(self.convert_pin(ee_pin));
        }

        // Convert rectangles (symbol body)
        for ee_rect in &ee_symbol.rectangles {
            ad_symbol.rectangles.push(self.convert_rectangle(ee_rect));
        }

        // Add default rectangle if no body is defined
        if ad_symbol.rectangles.is_empty() {
            ad_symbol.rectangles.push(self.create_default_body(&ad_symbol.pins));
        }

        ad_symbol
    }

    /// Convert EasyEDA footprint to Altium footprint
    pub fn convert_footprint(&self, ee_footprint: &EeFootprint, component_name: &str) -> AdFootprint {
        let mut ad_footprint = AdFootprint {
            name: ee_footprint.name.clone(),
            description: format!("{} - Converted from EasyEDA", ee_footprint.name),
            pads: Vec::new(),
            lines: Vec::new(),
            arcs: Vec::new(),
            texts: Vec::new(),
            model_3d: None,
        };

        // Convert pads
        for ee_pad in &ee_footprint.pads {
            ad_footprint.pads.push(self.convert_pad(ee_pad));
        }

        // Add 3D model reference if needed
        ad_footprint.model_3d = Some(Ad3DModel {
            filename: format!("{}.step", component_name),
            rotation_x: 0.0,
            rotation_y: 0.0,
            rotation_z: 0.0,
            offset_z: 0.0,
        });

        ad_footprint
    }

    /// Convert a pin from EasyEDA to Altium format
    fn convert_pin(&self, ee_pin: &EePin) -> AdPin {
        // EasyEDA uses 0.1 inch grid, Altium uses mil (1/1000 inch)
        // 1 inch = 1000 mil, 0.1 inch = 100 mil
        let x_mil = (ee_pin.x * 100.0) as i32;
        let y_mil = (ee_pin.y * 100.0) as i32;

        // Determine pin orientation based on EasyEDA rotation
        let orientation = self.rotation_to_orientation(ee_pin.rotation);

        // Guess electrical type from pin name (basic heuristic)
        let electrical = self.guess_pin_electrical(&ee_pin.name);

        AdPin {
            x: x_mil,
            y: y_mil,
            length: 100,  // Default pin length 100 mil
            name: ee_pin.name.clone(),
            designator: ee_pin.number.clone(),
            electrical,
            orientation,
        }
    }

    /// Convert a rectangle from EasyEDA to Altium format
    fn convert_rectangle(&self, ee_rect: &EeRectangle) -> AdRectangle {
        let x_mil = (ee_rect.x * 100.0) as i32;
        let y_mil = (ee_rect.y * 100.0) as i32;
        let width_mil = (ee_rect.width * 100.0) as i32;
        let height_mil = (ee_rect.height * 100.0) as i32;

        AdRectangle {
            x: x_mil,
            y: y_mil,
            width: width_mil,
            height: height_mil,
            color: 0x000000,  // Black color
            is_solid: false,
        }
    }

    /// Convert a pad from EasyEDA to Altium format
    fn convert_pad(&self, ee_pad: &EePad) -> AdPad {
        // EasyEDA uses mm, Altium uses mil
        // 1 mm = 39.3701 mil
        let x_mil = ee_pad.x * 39.3701;
        let y_mil = ee_pad.y * 39.3701;
        let width_mil = ee_pad.width * 39.3701;
        let height_mil = ee_pad.height * 39.3701;

        // EasyEDA stores hole_radius, convert to diameter in mil
        let hole_mil = ee_pad.hole_radius.map(|r| r * 2.0 * 39.3701).unwrap_or(0.0);

        // Determine pad shape
        let shape = match ee_pad.shape.as_str() {
            "OVAL" | "ELLIPSE" => PadShape::Round,
            "RECT" => PadShape::Rectangle,
            "OCTAGON" => PadShape::Octagonal,
            "ROUNDRECT" => PadShape::RoundRect,
            _ => PadShape::Round,  // Default to round
        };

        // Determine layer
        let layer = if hole_mil > 0.0 {
            PadLayer::MultiLayer  // Through-hole pad
        } else {
            PadLayer::Top  // SMD pad (assume top layer)
        };

        AdPad {
            x: x_mil,
            y: y_mil,
            width: width_mil,
            height: height_mil,
            hole_size: hole_mil,
            shape,
            name: ee_pad.number.clone(),
            layer,
            rotation: ee_pad.rotation,
        }
    }

    /// Create a default symbol body based on pin positions
    fn create_default_body(&self, pins: &[AdPin]) -> AdRectangle {
        if pins.is_empty() {
            // Default size if no pins
            return AdRectangle {
                x: -100,
                y: -200,
                width: 200,
                height: 400,
                color: 0x000000,
                is_solid: false,
            };
        }

        // Calculate bounding box from pins
        let min_x = pins.iter().map(|p| p.x).min().unwrap_or(-100);
        let max_x = pins.iter().map(|p| p.x).max().unwrap_or(100);
        let min_y = pins.iter().map(|p| p.y).min().unwrap_or(-200);
        let max_y = pins.iter().map(|p| p.y).max().unwrap_or(200);

        // Add padding
        let padding = 50;
        let x = min_x - padding;
        let y = min_y - padding;
        let width = (max_x - min_x) + padding * 2;
        let height = (max_y - min_y) + padding * 2;

        AdRectangle {
            x,
            y,
            width,
            height,
            color: 0x000000,
            is_solid: false,
        }
    }

    /// Convert EasyEDA rotation to Altium pin orientation
    fn rotation_to_orientation(&self, rotation: i32) -> PinOrientation {
        match rotation {
            0 => PinOrientation::Right,
            90 => PinOrientation::Up,
            180 => PinOrientation::Left,
            270 => PinOrientation::Down,
            _ => PinOrientation::Right,  // Default
        }
    }

    /// Guess pin electrical type from pin name (basic heuristic)
    fn guess_pin_electrical(&self, name: &str) -> PinElectrical {
        let name_upper = name.to_uppercase();

        if name_upper.contains("VCC") || name_upper.contains("VDD") ||
           name_upper.contains("VSS") || name_upper.contains("GND") ||
           name_upper.contains("VBAT") || name_upper.contains("POWER") {
            PinElectrical::Power
        } else if name_upper.starts_with("IN") || name_upper.contains("_IN") {
            PinElectrical::Input
        } else if name_upper.starts_with("OUT") || name_upper.contains("_OUT") {
            PinElectrical::Output
        } else if name_upper.contains("IO") || name_upper.contains("GPIO") ||
                  name_upper.starts_with("P") {  // PA0, PB1, etc.
            PinElectrical::IO
        } else {
            PinElectrical::Passive  // Default to passive
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotation_to_orientation() {
        let converter = AltiumConverter::new();
        assert!(matches!(converter.rotation_to_orientation(0), PinOrientation::Right));
        assert!(matches!(converter.rotation_to_orientation(90), PinOrientation::Up));
        assert!(matches!(converter.rotation_to_orientation(180), PinOrientation::Left));
        assert!(matches!(converter.rotation_to_orientation(270), PinOrientation::Down));
    }

    #[test]
    fn test_guess_pin_electrical() {
        let converter = AltiumConverter::new();
        assert!(matches!(converter.guess_pin_electrical("VCC"), PinElectrical::Power));
        assert!(matches!(converter.guess_pin_electrical("GND"), PinElectrical::Power));
        assert!(matches!(converter.guess_pin_electrical("IN1"), PinElectrical::Input));
        assert!(matches!(converter.guess_pin_electrical("OUT1"), PinElectrical::Output));
        assert!(matches!(converter.guess_pin_electrical("PA0"), PinElectrical::IO));
        assert!(matches!(converter.guess_pin_electrical("GPIO1"), PinElectrical::IO));
    }

    #[test]
    fn test_unit_conversion_schematic() {
        // EasyEDA: 0.1 inch grid -> Altium: mil (1/1000 inch)
        // 1 unit in EasyEDA = 0.1 inch = 100 mil
        assert_eq!(1.0 * 100.0, 100.0);  // 1 EasyEDA unit = 100 mil
        assert_eq!(10.0 * 100.0, 1000.0);  // 10 EasyEDA units = 1000 mil = 1 inch
    }

    #[test]
    fn test_unit_conversion_pcb() {
        // EasyEDA: mm -> Altium: mil
        // 1 mm = 39.3701 mil
        let mm_to_mil = 39.3701;
        assert_eq!((1.0 * mm_to_mil * 100.0).round() / 100.0, 39.37);  // 1 mm ≈ 39.37 mil
        assert_eq!((25.4 * mm_to_mil * 10.0).round() / 10.0, 1000.0);  // 25.4 mm = 1 inch = 1000 mil
    }
}
