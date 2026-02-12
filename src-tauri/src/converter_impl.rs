use std::path::Path;
use crate::nlbn::easyeda::{EasyedaApi, models::{ComponentData, EeSymbol, EeFootprint}};
use crate::nlbn::kicad::{
    symbol::{KiSymbol, KiPin, KiRectangle, KiCircle, KiArc, KiPolyline, PinType, PinStyle},
    footprint::{
        KiFootprint, KiPad, KiTrack, KiText, KiLine, Ki3dModel, Drill, PadType, PadShape,
        KiCircle as FootprintKiCircle, KiArc as FootprintKiArc
    },
    SymbolExporter, FootprintExporter, ModelExporter,
};
use crate::nlbn::{LibraryManager, KicadVersion, Converter};
use crate::nlbn::error::Result;

/// High-level converter that orchestrates the entire conversion process
pub struct ComponentConverter {
    api: EasyedaApi,
    library_manager: LibraryManager,
    kicad_version: KicadVersion,
}

impl ComponentConverter {
    pub fn new(output_path: &Path, kicad_v5: bool) -> Self {
        let kicad_version = if kicad_v5 {
            KicadVersion::V5
        } else {
            KicadVersion::V6
        };

        Self {
            api: EasyedaApi::new(),
            library_manager: LibraryManager::new(output_path),
            kicad_version,
        }
    }

    /// Convert a component from LCSC/EasyEDA to KiCad
    pub async fn convert(
        &self,
        lcsc_id: &str,
        convert_symbol: bool,
        convert_footprint: bool,
        convert_3d: bool,
        overwrite: bool,
    ) -> Result<ConversionResult> {
        log::info!("Starting conversion for {}", lcsc_id);

        // Create output directories
        self.library_manager.create_directories()?;

        // Fetch component data from API
        let component_data = self.api.get_component_data(lcsc_id).await?;
        log::info!("Fetched component data: {}", component_data.title);

        let mut files_created = Vec::new();
        let mut skipped_items = Vec::new();
        let component_name = sanitize_component_name(&component_data.title);

        // Convert symbol
        if convert_symbol && !component_data.data_str.is_empty() {
            log::info!("Converting symbol...");
            let (symbol_file, written) = self.convert_symbol(&component_data, &component_name, overwrite)?;
            if written {
                files_created.push(symbol_file.to_string_lossy().to_string());
            } else {
                skipped_items.push("Symbol (already exists)");
            }
        }

        // Convert footprint
        if convert_footprint && !component_data.package_detail.is_empty() {
            log::info!("Converting footprint...");
            let footprint_file = self.convert_footprint(&component_data, &component_name)?;
            files_created.push(footprint_file.to_string_lossy().to_string());
        }

        // Convert 3D model
        if convert_3d {
            if let Some(model_info) = &component_data.model_3d {
                log::info!("Converting 3D model...");
                match self.convert_3d_model(&model_info.uuid, &component_name).await {
                    Ok(model_files) => {
                        for file in model_files {
                            files_created.push(file.to_string_lossy().to_string());
                        }
                    }
                    Err(e) => {
                        log::warn!("Failed to convert 3D model: {}", e);
                    }
                }
            } else {
                log::info!("No 3D model available for this component");
            }
        }

        // Build result message
        let mut message = format!("Successfully converted {} to {}", lcsc_id, component_name);
        if !skipped_items.is_empty() {
            message.push_str(&format!("\nSkipped: {} (enable overwrite to update)", skipped_items.join(", ")));
        }

        Ok(ConversionResult {
            lcsc_id: lcsc_id.to_string(),
            component_name: Some(component_name.clone()),
            success: true,
            message,
            files_created,
        })
    }

    fn convert_symbol(
        &self,
        component_data: &ComponentData,
        component_name: &str,
        overwrite: bool,
    ) -> Result<(std::path::PathBuf, bool)> {
        use crate::nlbn::easyeda::SymbolImporter;

        // Parse EasyEDA symbol data
        let ee_symbol = SymbolImporter::parse(&component_data.data_str)?;

        // Convert to KiCad symbol
        let ki_symbol = self.convert_ee_symbol_to_ki(
            &ee_symbol,
            component_name,
            &component_data.manufacturer,
            &component_data.datasheet,
            &component_data.lcsc_id,
            &component_data.jlc_id,
        )?;

        // Export to KiCad format
        let exporter = SymbolExporter::new(self.kicad_version);
        let symbol_data = exporter.export(&ki_symbol)?;

        // Write to library file
        let lib_path = self.library_manager.get_symbol_lib_path(self.kicad_version == KicadVersion::V5);
        let written = self.library_manager.add_or_update_component(&lib_path, component_name, &symbol_data, overwrite)?;

        if written {
            log::info!("Symbol written to: {}", lib_path.display());
        } else {
            log::info!("Symbol already exists, skipped: {}", lib_path.display());
        }

        Ok((lib_path, written))
    }

    fn convert_footprint(
        &self,
        component_data: &ComponentData,
        component_name: &str,
    ) -> Result<std::path::PathBuf> {
        use crate::nlbn::easyeda::FootprintImporter;

        // Parse EasyEDA footprint data
        let ee_footprint = FootprintImporter::parse(&component_data.package_detail)?;

        // Convert to KiCad footprint
        let ki_footprint = self.convert_ee_footprint_to_ki(&ee_footprint, component_name)?;

        // Export to KiCad format
        let exporter = FootprintExporter::new();
        let footprint_data = exporter.export(&ki_footprint)?;

        // Write footprint file
        let footprint_path = self.library_manager.write_footprint(component_name, &footprint_data)?;

        Ok(footprint_path)
    }

    async fn convert_3d_model(
        &self,
        uuid: &str,
        component_name: &str,
    ) -> Result<Vec<std::path::PathBuf>> {
        let mut files = Vec::new();

        // Download OBJ model
        let obj_data = self.api.download_3d_obj(uuid).await?;

        // Convert OBJ to VRML
        let model_exporter = ModelExporter::new();
        let wrl_data = model_exporter.obj_to_wrl(&obj_data)?;

        // Write VRML model
        let wrl_path = self.library_manager.write_wrl_model(component_name, &wrl_data)?;
        files.push(wrl_path);

        // Try to download STEP model (may fail)
        match self.api.download_3d_step(uuid).await {
            Ok(step_data) => {
                let step_path = self.library_manager.write_step_model(component_name, &step_data)?;
                files.push(step_path);
            }
            Err(e) => {
                log::warn!("STEP model not available: {}", e);
            }
        }

        Ok(files)
    }

    fn convert_ee_symbol_to_ki(
        &self,
        ee_symbol: &EeSymbol,
        component_name: &str,
        manufacturer: &str,
        datasheet: &str,
        lcsc_id: &str,
        jlc_id: &str,
    ) -> Result<KiSymbol> {
        let converter = Converter::new(self.kicad_version);

        // Convert pins
        let pins: Vec<KiPin> = ee_symbol.pins.iter().map(|ee_pin| {
            let pin_type = PinType::from_easyeda(&ee_pin.electric_type);
            let style = if ee_pin.dot {
                PinStyle::Inverted
            } else if ee_pin.clock {
                PinStyle::Clock
            } else {
                PinStyle::Line
            };

            KiPin {
                number: ee_pin.number.clone(),
                name: ee_pin.name.clone(),
                pin_type,
                style,
                pos_x: ee_pin.x,
                pos_y: converter.flip_y(ee_pin.y),
                rotation: ee_pin.rotation,
                length: ee_pin.length,
            }
        }).collect();

        // Convert rectangles
        let rectangles: Vec<KiRectangle> = ee_symbol.rectangles.iter().map(|rect| {
            KiRectangle {
                x1: rect.x,
                y1: converter.flip_y(rect.y),
                x2: rect.x + rect.width,
                y2: converter.flip_y(rect.y + rect.height),
                stroke_width: rect.stroke_width,
                fill: rect.fill,
            }
        }).collect();

        // Convert circles
        let circles: Vec<KiCircle> = ee_symbol.circles.iter().map(|circle| {
            KiCircle {
                cx: circle.cx,
                cy: converter.flip_y(circle.cy),
                radius: circle.radius,
                stroke_width: circle.stroke_width,
                fill: circle.fill,
            }
        }).collect();

        // Convert polylines
        let polylines: Vec<KiPolyline> = ee_symbol.polylines.iter().map(|polyline| {
            let points: Vec<(f64, f64)> = polyline.points.iter()
                .map(|(x, y)| (*x, converter.flip_y(*y)))
                .collect();

            KiPolyline {
                points,
                stroke_width: polyline.stroke_width,
                fill: false,
            }
        }).collect();

        // For now, skip arcs and other complex shapes
        let arcs = Vec::new();

        Ok(KiSymbol {
            name: component_name.to_string(),
            reference: ee_symbol.prefix.clone(),
            value: component_name.to_string(),
            footprint: String::new(),
            datasheet: datasheet.to_string(),
            manufacturer: manufacturer.to_string(),
            lcsc_id: lcsc_id.to_string(),
            jlc_id: jlc_id.to_string(),
            pins,
            rectangles,
            circles,
            arcs,
            polylines,
        })
    }

    fn convert_ee_footprint_to_ki(
        &self,
        ee_footprint: &EeFootprint,
        component_name: &str,
    ) -> Result<KiFootprint> {
        let converter = Converter::new(self.kicad_version);

        // Convert pads
        let pads: Vec<KiPad> = ee_footprint.pads.iter().map(|ee_pad| {
            let pad_type = if ee_pad.hole_radius.is_some() {
                PadType::ThroughHole
            } else {
                PadType::Smd
            };

            let shape = PadShape::from_easyeda(&ee_pad.shape);

            let drill = ee_pad.hole_radius.map(|radius| {
                Drill {
                    diameter: radius * 2.0,  // Convert radius to diameter
                    width: None,  // No oval drills for now
                    offset_x: 0.0,
                    offset_y: 0.0,
                }
            });

            KiPad {
                number: ee_pad.number.clone(),
                pad_type,
                shape,
                pos_x: converter.px_to_mm(ee_pad.x),
                pos_y: converter.px_to_mm(converter.flip_y(ee_pad.y)),
                size_x: converter.px_to_mm(ee_pad.width),
                size_y: converter.px_to_mm(ee_pad.height),
                rotation: ee_pad.rotation,
                drill,
                layers: vec!["F.Cu".to_string(), "F.Paste".to_string(), "F.Mask".to_string()],
                polygon: None,  // No custom polygons for now
            }
        }).collect();

        // Convert tracks to lines
        let lines: Vec<KiLine> = ee_footprint.tracks.iter().filter_map(|track| {
            let coords: Vec<f64> = track.points
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();

            if coords.len() >= 4 {
                Some(KiLine {
                    start_x: converter.px_to_mm(coords[0]),
                    start_y: converter.px_to_mm(converter.flip_y(coords[1])),
                    end_x: converter.px_to_mm(coords[2]),
                    end_y: converter.px_to_mm(converter.flip_y(coords[3])),
                    width: converter.px_to_mm(track.stroke_width),
                    layer: "F.SilkS".to_string(),
                })
            } else {
                None
            }
        }).collect();

        // Convert circles
        let circles: Vec<FootprintKiCircle> = ee_footprint.circles.iter().map(|circle| {
            let center_x = converter.px_to_mm(circle.cx);
            let center_y = converter.px_to_mm(converter.flip_y(circle.cy));
            let radius_mm = converter.px_to_mm(circle.radius);

            // KiCad represents circles with center and end point (on the circle)
            FootprintKiCircle {
                center_x,
                center_y,
                end_x: center_x + radius_mm,  // Point on circle (radius to the right)
                end_y: center_y,
                width: converter.px_to_mm(circle.stroke_width),
                layer: "F.SilkS".to_string(),
                fill: circle.fill,
            }
        }).collect();

        // Reference and value texts
        let texts = vec![
            KiText {
                text: "REF**".to_string(),
                pos_x: 0.0,
                pos_y: -3.0,
                rotation: 0.0,
                layer: "F.SilkS".to_string(),
                size: 1.0,
                thickness: 0.15,
            },
            KiText {
                text: component_name.to_string(),
                pos_x: 0.0,
                pos_y: 3.0,
                rotation: 0.0,
                layer: "F.Fab".to_string(),
                size: 1.0,
                thickness: 0.15,
            },
        ];

        // 3D model reference (if exists)
        let model_3d = Some(Ki3dModel {
            path: format!("${{KIPRJMOD}}/nlbn.3dshapes/{}.wrl", component_name),
            offset: (0.0, 0.0, 0.0),
            scale: (1.0, 1.0, 1.0),
            rotate: (0.0, 0.0, 0.0),
        });

        Ok(KiFootprint {
            name: component_name.to_string(),
            pads,
            tracks: Vec::new(),
            lines,
            circles,
            arcs: Vec::new(),
            texts,
            model_3d,
        })
    }
}

/// Result of component conversion
#[derive(Debug, Clone)]
pub struct ConversionResult {
    pub lcsc_id: String,
    pub component_name: Option<String>,
    pub success: bool,
    pub message: String,
    pub files_created: Vec<String>,
}

/// Sanitize component name for file system
fn sanitize_component_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}
