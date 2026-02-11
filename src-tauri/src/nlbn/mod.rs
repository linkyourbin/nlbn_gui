// NLBN core module (migrated from CLI)
pub mod easyeda;
pub mod kicad;
pub mod converter;
pub mod library;
pub mod error;

// Re-export commonly used types
pub use error::{AppError, Result};
pub use easyeda::{EasyedaApi, SymbolImporter, FootprintImporter};
pub use kicad::{SymbolExporter, FootprintExporter, ModelExporter};
pub use converter::Converter;
pub use library::LibraryManager;

// Enum types (from cli.rs)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KicadVersion {
    V5,
    V6,
}
