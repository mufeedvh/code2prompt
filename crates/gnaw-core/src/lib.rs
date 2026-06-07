//! Core library for gnaw.
pub mod builtin_templates;
#[cfg(feature = "compression")]
pub mod compressor;
pub mod configuration;
pub mod file_processor;
pub mod filter;
pub mod git;
pub mod path;
pub mod selection;
pub mod session;
pub mod sort;
pub mod template;
pub mod tokenizer;
pub mod util;
