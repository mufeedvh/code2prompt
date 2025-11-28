//! CSV file processor with schema extraction.
//!
//! This processor uses the `csv` crate to robustly parse CSV files and extract:
//! - Column headers
//! - One sample data row
//!
//! This provides sufficient context for LLMs to understand the data structure
//! without wasting tokens on thousands of rows.

use super::{DefaultTextProcessor, FileProcessor};
use anyhow::{Context, Result};
use std::path::Path;

/// CSV processor that extracts headers and one sample row.
///
/// Uses streaming to avoid loading large files into memory.
/// Falls back to raw text if parsing fails.
pub struct CsvProcessor;

impl CsvProcessor {
    /// Internal processing with specific delimiter.
    ///
    /// # Arguments
    ///
    /// * `content` - Raw CSV bytes
    /// * `delimiter` - Field delimiter (b',' for CSV, b'\t' for TSV)
    /// * `path` - File path for error messages
    pub(crate) fn process_with_delimiter(
        &self,
        content: &[u8],
        delimiter: u8,
        _path: &Path,
    ) -> Result<String> {
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(delimiter)
            .flexible(true) // Allow variable number of fields
            .from_reader(content);

        // Extract headers
        let headers = reader
            .headers()
            .context("Failed to read CSV headers")?
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        if headers.is_empty() {
            anyhow::bail!("CSV file has no headers");
        }

        // Read first data row
        let mut records = reader.records();
        let first_row = records
            .next()
            .transpose()
            .context("Failed to read first data row")?;

        let mut output = String::new();
        output.push_str("CSV Schema (1 sample row):\n");
        output.push_str(&format!("Headers: {}\n", headers.join(", ")));

        if let Some(row) = first_row {
            let values: Vec<String> = row.iter().map(|field| format!("\"{}\"", field)).collect();
            output.push_str(&format!("Sample: {}\n", values.join(", ")));

            // Count remaining rows for truncation message
            let remaining_rows = records.count();
            if remaining_rows > 0 {
                output.push_str(&format!("... [{} more rows omitted]\n", remaining_rows));
            }
        } else {
            output.push_str("(No data rows found)\n");
        }

        Ok(output)
    }
}

impl FileProcessor for CsvProcessor {
    fn process(&self, content: &[u8], path: &Path) -> Result<String> {
        match self.process_with_delimiter(content, b',', path) {
            Ok(result) => Ok(result),
            Err(e) => {
                log::warn!(
                    "CSV parsing failed for {:?}: {}. Using raw text fallback.",
                    path,
                    e
                );
                // Fallback to raw text
                let fallback = DefaultTextProcessor;
                fallback.process(content, path)
            }
        }
    }
}
