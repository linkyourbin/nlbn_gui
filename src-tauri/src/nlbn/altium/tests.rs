/// Test Altium export functionality
#[cfg(test)]
mod tests {
    use crate::nlbn::altium::{
        symbol::{AdSymbol, AdPin, AdRectangle, PinElectrical, PinOrientation},
        footprint::{AdFootprint, AdPad, AdLine, AdArc, AdText, Ad3DModel, PadShape, PadLayer},
        SymbolExporter, FootprintExporter,
    };
    use std::path::PathBuf;

    #[test]
    fn test_simple_symbol_export() {
        // Create a simple test symbol
        let symbol = AdSymbol {
            libref: "TEST_COMPONENT".to_string(),
            description: "Test Component Description".to_string(),
            pins: vec![
                AdPin {
                    x: 0,
                    y: 100,
                    length: 100,
                    name: "VCC".to_string(),
                    designator: "1".to_string(),
                    electrical: PinElectrical::Power,
                    orientation: PinOrientation::Right,
                },
                AdPin {
                    x: 0,
                    y: 0,
                    length: 100,
                    name: "GND".to_string(),
                    designator: "2".to_string(),
                    electrical: PinElectrical::Power,
                    orientation: PinOrientation::Right,
                },
            ],
            rectangles: vec![
                AdRectangle {
                    x: 100,
                    y: -100,
                    width: 200,
                    height: 300,
                    color: 0x000000,
                    is_solid: false,
                }
            ],
            lines: vec![],
            texts: vec![],
        };

        // Export to file
        let exporter = SymbolExporter::new();
        let temp_path = PathBuf::from("test_symbol.SchLib");

        let result = exporter.export(&symbol, &temp_path);
        assert!(result.is_ok(), "Export failed: {:?}", result.err());

        // Read file and verify content
        let content = std::fs::read_to_string(&temp_path).expect("Failed to read file");

        println!("Generated SchLib content:\n{}", content);

        // Verify header
        assert!(content.contains("|HEADER=Protel for Windows"));

        // Verify component
        assert!(content.contains("|RECORD=1"));
        assert!(content.contains("|LIBREF=TEST_COMPONENT"));

        // Verify pins
        assert!(content.contains("|RECORD=41"));
        assert!(content.contains("|NAME=VCC"));
        assert!(content.contains("|DESIGNATOR=1"));

        // Verify rectangle
        assert!(content.contains("|RECORD=2"));

        // Cleanup
        let _ = std::fs::remove_file(&temp_path);
    }

    #[test]
    fn test_simple_footprint_export() {
        // Create a simple test footprint
        let footprint = AdFootprint {
            name: "TEST_QFN48".to_string(),
            description: "Test QFN-48 Package".to_string(),
            pads: vec![
                AdPad {
                    x: 100.0,
                    y: 100.0,
                    width: 15.0,
                    height: 60.0,
                    hole_size: 0.0,
                    shape: PadShape::Rectangle,
                    name: "1".to_string(),
                    layer: PadLayer::Top,
                    rotation: 0.0,
                },
                AdPad {
                    x: 150.0,
                    y: 100.0,
                    width: 15.0,
                    height: 60.0,
                    hole_size: 0.0,
                    shape: PadShape::Rectangle,
                    name: "2".to_string(),
                    layer: PadLayer::Top,
                    rotation: 0.0,
                },
            ],
            lines: vec![
                AdLine {
                    start_x: 0.0,
                    start_y: 0.0,
                    end_x: 200.0,
                    end_y: 0.0,
                    width: 5.0,
                    layer: "TOPOVERLAY".to_string(),
                }
            ],
            arcs: vec![],
            texts: vec![],
            model_3d: Some(Ad3DModel {
                filename: "test.step".to_string(),
                rotation_x: 0.0,
                rotation_y: 0.0,
                rotation_z: 0.0,
                offset_z: 0.0,
            }),
        };

        // Export to file
        let exporter = FootprintExporter::new();
        let temp_path = PathBuf::from("test_footprint.PcbLib");

        let result = exporter.export(&footprint, &temp_path);
        assert!(result.is_ok(), "Export failed: {:?}", result.err());

        // Read file and verify content
        let content = std::fs::read_to_string(&temp_path).expect("Failed to read file");

        println!("Generated PcbLib content:\n{}", content);

        // Verify header
        assert!(content.contains("|HEADER=Protel for Windows"));

        // Verify footprint
        assert!(content.contains("|RECORD=2"));
        assert!(content.contains("|NAME=TEST_QFN48"));

        // Verify pads
        assert!(content.contains("|RECORD=3"));
        assert!(content.contains("|NAME=1"));
        assert!(content.contains("|LAYER=TOP"));

        // Verify line
        assert!(content.contains("|RECORD=6"));

        // Verify 3D model
        assert!(content.contains("|RECORD=16"));
        assert!(content.contains("|MODELNAME=test.step"));

        // Cleanup
        let _ = std::fs::remove_file(&temp_path);
    }
}
