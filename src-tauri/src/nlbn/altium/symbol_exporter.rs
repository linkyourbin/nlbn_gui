/// Altium Designer SchLib (.SchLib) file exporter
use std::fs::File;
use std::io::{Write, Result};
use std::path::Path;
use super::symbol::*;

/// Exports schematic symbols to Altium Designer .SchLib format
pub struct SymbolExporter;

impl SymbolExporter {
    pub fn new() -> Self {
        Self
    }

    /// Export a symbol to an Altium .SchLib file
    ///
    /// # Arguments
    /// * `symbol` - The symbol data to export
    /// * `output_path` - Path where the .SchLib file will be created
    pub fn export(&self, symbol: &AdSymbol, output_path: &Path) -> Result<()> {
        let mut content = String::new();

        // Write file header
        self.write_header(&mut content);

        // Write component definition
        self.write_component(&mut content, symbol);

        // Write graphical elements
        self.write_rectangles(&mut content, &symbol.rectangles);
        self.write_lines(&mut content, &symbol.lines);
        self.write_texts(&mut content, &symbol.texts);

        // Write pins
        self.write_pins(&mut content, &symbol.pins);

        // Create parent directories if they don't exist
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Write to file
        let mut file = File::create(output_path)?;
        file.write_all(content.as_bytes())?;

        Ok(())
    }

    /// Write SchLib file header
    fn write_header(&self, content: &mut String) {
        content.push_str("|HEADER=Protel for Windows - Schematic Library Editor Binary File Version 5.0\n");
        content.push_str("|WEIGHT=748\n");
        content.push_str("|MINORVERSION=2\n");
        content.push_str("|USEMBCS=T\n");
        content.push_str("\n");
    }

    /// Write component definition (RECORD=1)
    fn write_component(&self, content: &mut String, symbol: &AdSymbol) {
        content.push_str("|RECORD=1\n");
        content.push_str(&format!("|LIBREF={}\n", Self::escape_string(&symbol.libref)));
        content.push_str(&format!("|COMPONENTDESCRIPTION={}\n", Self::escape_string(&symbol.description)));
        content.push_str("|PARTCOUNT=1\n");
        content.push_str("|DISPLAYMODECOUNT=1\n");
        content.push_str("|INDEXINSHEET=-1\n");
        content.push_str("|OWNERPARTID=-1\n");
        content.push_str("|LOCATION.X=0\n");
        content.push_str("|LOCATION.Y=0\n");
        content.push_str("|LIBRARYPATH=*\n");
        content.push_str("|SOURCELIBRARYNAME=*\n");
        content.push_str("|TARGETFILENAME=*\n");
        content.push_str("\n");
    }

    /// Write rectangle elements (RECORD=2)
    fn write_rectangles(&self, content: &mut String, rectangles: &[AdRectangle]) {
        for rect in rectangles {
            content.push_str("|RECORD=2\n");
            content.push_str("|OWNERINDEX=1\n");
            content.push_str("|OWNERPARTID=-1\n");
            content.push_str(&format!("|LOCATION.X={}\n", rect.x));
            content.push_str(&format!("|LOCATION.Y={}\n", rect.y));
            content.push_str(&format!("|CORNER.X={}\n", rect.x + rect.width));
            content.push_str(&format!("|CORNER.Y={}\n", rect.y + rect.height));
            content.push_str(&format!("|COLOR={}\n", rect.color));
            content.push_str("|AREACOLOR=16777215\n");
            content.push_str(&format!("|ISSOLID={}\n", if rect.is_solid { "T" } else { "F" }));
            content.push_str("|LINEWIDTH=1\n");
            content.push_str("\n");
        }
    }

    /// Write line elements (RECORD=13)
    fn write_lines(&self, content: &mut String, lines: &[AdLine]) {
        for line in lines {
            content.push_str("|RECORD=13\n");
            content.push_str("|OWNERINDEX=1\n");
            content.push_str("|OWNERPARTID=-1\n");
            content.push_str(&format!("|LINEWIDTH={}\n", line.width));
            content.push_str(&format!("|COLOR={}\n", line.color));
            content.push_str("|LOCATIONCOUNT=2\n");
            content.push_str(&format!("|X1={}\n", line.start_x));
            content.push_str(&format!("|Y1={}\n", line.start_y));
            content.push_str(&format!("|X2={}\n", line.end_x));
            content.push_str(&format!("|Y2={}\n", line.end_y));
            content.push_str("\n");
        }
    }

    /// Write text elements (RECORD=4)
    fn write_texts(&self, content: &mut String, texts: &[AdText]) {
        for text in texts {
            content.push_str("|RECORD=4\n");
            content.push_str("|OWNERINDEX=1\n");
            content.push_str("|OWNERPARTID=-1\n");
            content.push_str(&format!("|LOCATION.X={}\n", text.x));
            content.push_str(&format!("|LOCATION.Y={}\n", text.y));
            content.push_str(&format!("|TEXT={}\n", Self::escape_string(&text.text)));
            content.push_str("|FONTID=1\n");
            content.push_str("|COLOR=0\n");
            content.push_str(&format!("|ORIENTATION={}\n", (text.rotation / 90.0) as i32));
            content.push_str("\n");
        }
    }

    /// Write pin elements (RECORD=41)
    fn write_pins(&self, content: &mut String, pins: &[AdPin]) {
        for pin in pins {
            content.push_str("|RECORD=41\n");
            content.push_str("|OWNERINDEX=1\n");
            content.push_str("|OWNERPARTID=-1\n");
            content.push_str(&format!("|LOCATION.X={}\n", pin.x));
            content.push_str(&format!("|LOCATION.Y={}\n", pin.y));
            content.push_str(&format!("|PINLENGTH={}\n", pin.length));
            content.push_str(&format!("|ELECTRICAL={}\n", pin.electrical.to_altium_code()));
            content.push_str(&format!("|PINCONGLOMERATE={}\n", pin.orientation.to_altium_code()));
            content.push_str(&format!("|NAME={}\n", Self::escape_string(&pin.name)));
            content.push_str(&format!("|DESIGNATOR={}\n", Self::escape_string(&pin.designator)));
            content.push_str("|SWAPIDPIN=\n");
            content.push_str("|SWAPIDPART=\n");
            content.push_str("|COLOR=0\n");
            content.push_str("|PINNAME_POSITIONCONGLOMERATE=11\n");
            content.push_str("\n");
        }
    }

    /// Escape special characters in strings for Altium format
    fn escape_string(s: &str) -> String {
        // Altium uses | as delimiter, so we need to escape it
        s.replace('|', "\\|")
            .replace('\n', " ")
            .replace('\r', "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_string() {
        assert_eq!(SymbolExporter::escape_string("test|string"), "test\\|string");
        assert_eq!(SymbolExporter::escape_string("test\nstring"), "test string");
    }

    #[test]
    fn test_pin_electrical_codes() {
        assert_eq!(PinElectrical::Input.to_altium_code(), 0);
        assert_eq!(PinElectrical::IO.to_altium_code(), 1);
        assert_eq!(PinElectrical::Output.to_altium_code(), 2);
        assert_eq!(PinElectrical::Power.to_altium_code(), 7);
    }

    #[test]
    fn test_pin_orientation_codes() {
        assert_eq!(PinOrientation::Right.to_altium_code(), 0);
        assert_eq!(PinOrientation::Up.to_altium_code(), 1);
        assert_eq!(PinOrientation::Left.to_altium_code(), 2);
        assert_eq!(PinOrientation::Down.to_altium_code(), 3);
    }
}
