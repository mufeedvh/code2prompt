//! Default text processor for standard file types.
//!
//! This processor handles all file types that don't require special processing.
//! It converts raw bytes to UTF-8 strings using lossy conversion to handle
//! invalid UTF-8 sequences gracefully.

use super::FileProcessor;
use anyhow::Result;
use chardetng::EncodingDetector;
use std::path::Path;

/// Default processor that converts bytes to UTF-8 string.
///
/// Uses lossy conversion to avoid crashes on invalid UTF-8 sequences.
/// This is the fallback processor for all file types without specific handlers.
pub struct DefaultTextProcessor;

impl FileProcessor for DefaultTextProcessor {
    fn process(&self, content: &[u8], _path: &Path) -> Result<String> {
        let mut detector = EncodingDetector::new();
        detector.feed(content, true);

        // Guess the encoding; if none is found, default to UTF-8
        let encoding = detector.guess(None, true);

        let (cow, _encoding_used, _had_errors) = encoding.decode(content);

        match cow {
            std::borrow::Cow::Owned(s) => Ok(s),
            std::borrow::Cow::Borrowed(s) => Ok(s.to_string()),
        }
    }
}
