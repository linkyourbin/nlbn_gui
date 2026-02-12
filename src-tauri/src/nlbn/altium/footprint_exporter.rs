/// Altium Designer PcbLib (.PcbLib) file exporter
use std::fs::File;
use std::io::{Write, Result};
use std::path::Path;
use uuid::Uuid;
use super::footprint::*;

/// Exports PCB footprints to Altium Designer .PcbLib format
pub struct FootprintExporter;

impl FootprintExporter {
    pub fn new() -> Self {
        Self
    }

    /// Export a footprint to an Altium .PcbLib file
    ///
    /// # Arguments
    /// * `footprint` - The footprint data to export
    /// * `output_path` - Path where the .PcbLib file will be created
    pub fn export(&self, footprint: &AdFootprint, output_path: &Path) -> Result<()> {
        let mut content = String::new();

        // Write file header
        self.write_header(&mut content);

        // Write footprint definition
        self.write_footprint_def(&mut content, footprint);

        // Write pads
        self.write_pads(&mut content, &footprint.pads);

        // Write graphical elements
        self.write_lines(&mut content, &footprint.lines);
        self.write_arcs(&mut content, &footprint.arcs);
        self.write_texts(&mut content, &footprint.texts);

        // Write 3D model if present
        if let Some(model) = &footprint.model_3d {
            self.write_3d_model(&mut content, model);
        }

        // Create parent directories if they don't exist
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Write to file
        let mut file = File::create(output_path)?;
        file.write_all(content.as_bytes())?;

        Ok(())
    }

    /// Write PcbLib file header
    fn write_header(&self, content: &mut String) {
        content.push_str("|HEADER=Protel for Windows - PCB Library Binary File Version 5.0\n");
        content.push_str("|WEIGHT=748\n");
        content.push_str("\n");
    }

    /// Write footprint definition (RECORD=2)
    fn write_footprint_def(&self, content: &mut String, footprint: &AdFootprint) {
        content.push_str("|RECORD=2\n");
        content.push_str(&format!("|NAME={}\n", Self::escape_string(&footprint.name)));
        content.push_str(&format!("|DESCRIPTION={}\n", Self::escape_string(&footprint.description)));
        content.push_str("\n");
    }

    /// Write pad elements (RECORD=3)
    fn write_pads(&self, content: &mut String, pads: &[AdPad]) {
        for pad in pads {
            content.push_str("|RECORD=3\n");
            content.push_str("|OWNERINDEX=0\n");
            content.push_str(&format!("|LAYER={}\n", pad.layer.to_altium_name()));
            content.push_str(&format!("|X={}MIL\n", Self::format_coord(pad.x)));
            content.push_str(&format!("|Y={}MIL\n", Self::format_coord(pad.y)));
            content.push_str(&format!("|XSIZE={}MIL\n", Self::format_coord(pad.width)));
            content.push_str(&format!("|YSIZE={}MIL\n", Self::format_coord(pad.height)));
            content.push_str(&format!("|HOLESIZE={}MIL\n", Self::format_coord(pad.hole_size)));
            content.push_str(&format!("|SHAPE={}\n", pad.shape.to_altium_code()));
            content.push_str("|PADMODE=0\n");
            content.push_str(&format!("|PLATED={}\n", if pad.hole_size > 0.0 { "T" } else { "F" }));
            content.push_str(&format!("|NAME={}\n", Self::escape_string(&pad.name)));

            // Add rotation if not zero
            if pad.rotation != 0.0 {
                content.push_str(&format!("|ROTATION={}\n", pad.rotation));
            }

            content.push_str("\n");
        }
    }

    /// Write line elements (RECORD=6)
    fn write_lines(&self, content: &mut String, lines: &[AdLine]) {
        for line in lines {
            content.push_str("|RECORD=6\n");
            content.push_str(&format!("|LAYER={}\n", line.layer));
            content.push_str(&format!("|START.X={}MIL\n", Self::format_coord(line.start_x)));
            content.push_str(&format!("|START.Y={}MIL\n", Self::format_coord(line.start_y)));
            content.push_str(&format!("|END.X={}MIL\n", Self::format_coord(line.end_x)));
            content.push_str(&format!("|END.Y={}MIL\n", Self::format_coord(line.end_y)));
            content.push_str(&format!("|WIDTH={}MIL\n", Self::format_coord(line.width)));
            content.push_str("\n");
        }
    }

    /// Write arc elements (RECORD=7)
    fn write_arcs(&self, content: &mut String, arcs: &[AdArc]) {
        for arc in arcs {
            content.push_str("|RECORD=7\n");
            content.push_str(&format!("|LAYER={}\n", arc.layer));
            content.push_str(&format!("|LOCATION.X={}MIL\n", Self::format_coord(arc.center_x)));
            content.push_str(&format!("|LOCATION.Y={}MIL\n", Self::format_coord(arc.center_y)));
            content.push_str(&format!("|RADIUS={}MIL\n", Self::format_coord(arc.radius)));
            content.push_str(&format!("|STARTANGLE={}\n", arc.start_angle));
            content.push_str(&format!("|ENDANGLE={}\n", arc.end_angle));
            content.push_str(&format!("|WIDTH={}MIL\n", Self::format_coord(arc.width)));
            content.push_str("\n");
        }
    }

    /// Write text elements (RECORD=8)
    fn write_texts(&self, content: &mut String, texts: &[AdText]) {
        for text in texts {
            content.push_str("|RECORD=8\n");
            content.push_str(&format!("|LAYER={}\n", text.layer));
            content.push_str(&format!("|X={}MIL\n", Self::format_coord(text.x)));
            content.push_str(&format!("|Y={}MIL\n", Self::format_coord(text.y)));
            content.push_str(&format!("|TEXT={}\n", Self::escape_string(&text.text)));
            content.push_str(&format!("|HEIGHT={}MIL\n", Self::format_coord(text.height)));
            content.push_str(&format!("|WIDTH={}MIL\n", Self::format_coord(text.width)));
            content.push_str(&format!("|ROTATION={}\n", text.rotation));
            content.push_str("|FONTID=1\n");
            content.push_str("\n");
        }
    }

    /// Write 3D model reference (RECORD=16)
    fn write_3d_model(&self, content: &mut String, model: &Ad3DModel) {
        let model_id = Uuid::new_v4();

        content.push_str("|RECORD=16\n");
        content.push_str("|OWNERINDEX=0\n");
        content.push_str(&format!("|MODELNAME={}\n", Self::escape_string(&model.filename)));
        content.push_str(&format!("|MODELID={{{}}}\n", model_id));
        content.push_str("|MODELDESCRIPTION=\n");
        content.push_str(&format!("|ROTATION.X={}\n", model.rotation_x));
        content.push_str(&format!("|ROTATION.Y={}\n", model.rotation_y));
        content.push_str(&format!("|ROTATION.Z={}\n", model.rotation_z));
        content.push_str(&format!("|Z={}MIL\n", Self::format_coord(model.offset_z)));
        content.push_str("|CHECKSUM=\n");
        content.push_str("|EMBEDSTEP=F\n");
        content.push_str("\n");
    }

    /// Format coordinate value (remove unnecessary decimals)
    fn format_coord(value: f64) -> String {
        if value.fract() == 0.0 {
            format!("{}", value as i32)
        } else {
            format!("{:.4}", value).trim_end_matches('0').trim_end_matches('.').to_string()
        }
    }

    /// Escape special characters in strings for Altium format
    fn escape_string(s: &str) -> String {
        s.replace('|', "\\|")
            .replace('\n', " ")
            .replace('\r', "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_coord() {
        assert_eq!(FootprintExporter::format_coord(100.0), "100");
        assert_eq!(FootprintExporter::format_coord(100.5), "100.5");
        assert_eq!(FootprintExporter::format_coord(100.5000), "100.5");
        assert_eq!(FootprintExporter::format_coord(100.1234), "100.1234");
    }

    #[test]
    fn test_pad_shape_codes() {
        assert_eq!(PadShape::Round.to_altium_code(), 0);
        assert_eq!(PadShape::Rectangle.to_altium_code(), 1);
        assert_eq!(PadShape::Octagonal.to_altium_code(), 2);
        assert_eq!(PadShape::RoundRect.to_altium_code(), 3);
    }

    #[test]
    fn test_pad_layer_names() {
        assert_eq!(PadLayer::Top.to_altium_name(), "TOP");
        assert_eq!(PadLayer::Bottom.to_altium_name(), "BOTTOM");
        assert_eq!(PadLayer::MultiLayer.to_altium_name(), "MULTILAYER");
    }
}
