// CLI layer module
// Handles communication with glazewm via command line interface

pub mod client;
pub mod errors;
pub mod parser;

pub use client::{GlazewmClient, RealGlazewmClient};
pub use errors::CliError;
pub use parser::GlazewmParser;