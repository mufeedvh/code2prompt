//! CLI-side rendered-prompt intermediate.
//!
//! The pipeline produces `gnaw_core::pipeline::Rendered`; the CLI wraps it with
//! the presentation fields its output + split stages read. Lives here, not in
//! core, because it's a frontend concern — this type left `gnaw_core::session`
//! when the eager session was deleted. A REST/MCP frontend shapes its own
//! response from `Rendered` and doesn't use this.

use gnaw_core::pipeline::FindingDto;

pub struct RenderedPrompt {
    pub prompt: String,
    pub token_count: usize,
    pub model_info: &'static str,
    pub secret_findings: Vec<FindingDto>,
}
