//! File processor module for handling different file types intelligently.
//!
//! This module provides a strategy pattern for processing file contents based on their extension.
//! Instead of naively reading all files as raw text, it applies format-specific logic to extract
//! relevant information (e.g., schema + sample for CSV, code cells for Jupyter notebooks).

use anyhow::Result;
use std::path::Path;

mod csv;
mod default;
mod ipynb;
mod jsonl;
mod tsv;

pub use csv::CsvProcessor;
pub use default::DefaultTextProcessor;
pub use ipynb::JupyterNotebookProcessor;
pub use jsonl::JsonLinesProcessor;
pub use tsv::TsvProcessor;

/// Trait for processing file contents into LLM-optimized string representations.
///
/// Each processor takes raw bytes and produces a formatted string suitable for
/// inclusion in an LLM prompt. Processors may extract schemas, truncate content,
/// or apply other transformations to reduce token usage while preserving semantic value.
pub trait FileProcessor: Send + Sync {
    /// Process file content and return a formatted string.
    ///
    /// # Arguments
    ///
    /// * `content` - Raw file bytes
    /// * `path` - File path for context and error messages
    ///
    /// # Returns
    ///
    /// * `Result<String>` - Processed content or error
    fn process(&self, content: &[u8], path: &Path) -> Result<String>;
}

/// Factory function to get the appropriate processor for a file extension.
///
/// # Arguments
///
/// * `extension` - File extension (without dot)
///
/// # Returns
///
/// * `Box<dyn FileProcessor>` - Processor instance for the given extension
///
/// # Examples
///
/// ```ignore
/// let processor = get_processor_for_extension("csv");
/// let result = processor.process(&bytes, path)?;
/// ```
pub fn get_processor_for_extension(extension: &str) -> Box<dyn FileProcessor> {
    match extension.to_lowercase().as_str() {
        "csv" => Box::new(CsvProcessor),
        "tsv" => Box::new(TsvProcessor),
        "jsonl" | "ndjson" => Box::new(JsonLinesProcessor),
        "ipynb" => Box::new(JupyterNotebookProcessor),
        // Future processors can be added here:
        // "parquet" => Box::new(ParquetProcessor),
        // "xml" => Box::new(XmlProcessor),
        _ => Box::new(DefaultTextProcessor),
    }
}
