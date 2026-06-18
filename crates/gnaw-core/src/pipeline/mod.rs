//! Pipeline ports (trait contracts) and DTOs for the staged extraction
//! architecture: Source → Filter → Chunk → Count → Rank/Budget → Render.
//!
//! STEP 1 of the migration: definitions only. Nothing in the legacy
//! `session` path references this module yet; adapters (step 2) and the
//! runner (step 3) come later. This file must compile and test without
//! touching git2, ignore, or the filesystem — ports are pure contracts.
//!
//! DTOs are the wire schema. Field names and shapes are the contract shared
//! by CLI `--json`, the planned REST surface, and the planned MCP server.
//! Treat renames here as breaking changes.

use serde::{Deserialize, Serialize};

pub mod adapters;
pub mod dto;
pub mod ports;
pub mod runner;

pub use dto::*;
pub use ports::*;
pub use runner::{PipelineSpec, run};

/// Error type for pipeline stages. Library-level, so `thiserror`, not
/// `anyhow` — the binary maps these at the edge. Variants are coarse on
/// purpose for step 1; widen as adapters surface real failure modes.
#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("source failed: {0}")]
    Source(String),
    #[error("chunk failed: {0}")]
    Chunk(String),
    #[error("render failed: {0}")]
    Render(String),
    #[error("budget exceeded: {0}")]
    Budget(String),
}
