use serde::{Deserialize, Serialize};

/// Conversion options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionOptions {
    pub output_dir: String,
    pub convert_symbol: bool,
    pub convert_footprint: bool,
    pub convert_3d: bool,
    pub kicad_v5: bool,
    pub project_relative: bool,
    pub overwrite: bool,
    // Target format selection
    pub target_kicad: bool,
    pub target_altium: bool,
}

impl Default for ConversionOptions {
    fn default() -> Self {
        Self {
            output_dir: String::from("./output"),
            convert_symbol: true,
            convert_footprint: true,
            convert_3d: true,
            kicad_v5: false,
            project_relative: false,
            overwrite: false,
            target_kicad: true,
            target_altium: false,
        }
    }
}

/// Single conversion result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionResult {
    pub lcsc_id: String,
    pub success: bool,
    pub message: String,
    pub component_name: Option<String>,
    pub files_created: Vec<String>,
}

/// Batch conversion result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    pub total: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub results: Vec<ConversionResult>,
}

/// Conversion progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    pub current: usize,
    pub total: usize,
    pub lcsc_id: String,
    pub status: String,
}

/// History entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: i64,
    pub lcsc_id: String,
    pub component_name: Option<String>,
    pub success: bool,
    pub timestamp: String,
    pub output_dir: String,
}
