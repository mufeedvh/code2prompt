//! TSV (Tab-Separated Values) file processor.
//!
//! This processor is a thin wrapper around the CSV processor with tab delimiter.
//! It extracts headers and one sample row from TSV files.

use super::{CsvProcessor, FileProcessor};
use anyhow::Result;
use std::path::Path;

/// TSV processor that reuses CSV logic with tab delimiter.
pub struct TsvProcessor;

impl FileProcessor for TsvProcessor {
    fn process(&self, content: &[u8], path: &Path) -> Result<String> {
        let csv_processor = CsvProcessor;
        match csv_processor.process_with_delimiter(content, b'\t', path) {
            Ok(mut result) => {
                // Replace "CSV" with "TSV" in the output
                result = result.replace("CSV Schema", "TSV Schema");
                Ok(result)
            }
            Err(e) => {
                log::warn!(
                    "TSV parsing failed for {:?}: {}. Using raw text fallback.",
                    path,
                    e
                );
                // Fallback to raw text
                let fallback = super::DefaultTextProcessor;
                fallback.process(content, path)
            }
        }
    }
}
