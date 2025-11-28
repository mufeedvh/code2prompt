//! JSON Lines (JSONL) file processor with schema extraction.
//!
//! This processor parses JSONL/NDJSON files and extracts:
//! - Field names from the first JSON object
//! - One sample JSON object
//!
//! This provides sufficient context for LLMs without including thousands of lines.

use super::{DefaultTextProcessor, FileProcessor};
use anyhow::{Context, Result};
use serde_json::Value;
use std::path::Path;

/// JSONL processor that extracts schema and one sample line.
pub struct JsonLinesProcessor;

impl FileProcessor for JsonLinesProcessor {
    fn process(&self, content: &[u8], _path: &Path) -> Result<String> {
        let text = String::from_utf8_lossy(content);
        let mut lines = text.lines();

        // Get first line
        let first_line = match lines.next() {
            Some(line) if !line.trim().is_empty() => line,
            _ => {
                anyhow::bail!("JSONL file is empty or has no valid lines");
            }
        };

        // Parse first line as JSON
        let json_obj: Value = serde_json::from_str(first_line)
            .with_context(|| format!("Failed to parse first line as JSON: {}", first_line))?;

        // Extract field names
        let fields = if let Value::Object(map) = &json_obj {
            map.keys().cloned().collect::<Vec<_>>()
        } else {
            anyhow::bail!("First line is not a JSON object");
        };

        if fields.is_empty() {
            anyhow::bail!("JSON object has no fields");
        }

        // Count remaining lines
        let remaining_lines = lines.filter(|line| !line.trim().is_empty()).count();

        // Format output
        let mut output = String::new();
        output.push_str("JSONL Schema (1 sample line):\n");
        output.push_str(&format!("Fields: {}\n", fields.join(", ")));
        output.push_str(&format!("Sample: {}\n", first_line));

        if remaining_lines > 0 {
            output.push_str(&format!("... [{} more lines omitted]\n", remaining_lines));
        }

        Ok(output)
    }
}

impl JsonLinesProcessor {
    /// Process with fallback to raw text on error.
    pub fn process_with_fallback(&self, content: &[u8], path: &Path) -> Result<String> {
        match self.process(content, path) {
            Ok(result) => Ok(result),
            Err(e) => {
                log::warn!(
                    "JSONL parsing failed for {:?}: {}. Using raw text fallback.",
                    path,
                    e
                );
                let fallback = DefaultTextProcessor;
                fallback.process(content, path)
            }
        }
    }
}
