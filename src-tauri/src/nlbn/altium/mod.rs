/// Altium Designer file format support module
///
/// This module provides functionality to export EasyEDA components
/// to Altium Designer format (.SchLib and .PcbLib files).

pub mod symbol;
pub mod footprint;
pub mod symbol_exporter;
pub mod footprint_exporter;
pub mod converter;

// Re-export main types for convenience
pub use symbol::*;
pub use footprint::*;
pub use symbol_exporter::SymbolExporter;
pub use footprint_exporter::FootprintExporter;
pub use converter::AltiumConverter;
