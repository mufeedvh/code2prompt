//! Cross-stage data types. These ARE the wire schema (CLI --json, REST, MCP).
//! All are serde-serializable with stable field names.

use super::*;

/// Raw content as it leaves a source, before chunking. Binary is a
/// first-class state so a changed-files response can honestly report a
/// binary file rather than emitting lossy garbage or dropping it silently.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", tag = "kind", content = "data")]
pub enum RawContent {
    Text(String),
    /// Binary or over-size: content deliberately omitted.
    Omitted,
}

/// One item yielded by a `ContextSource` — a working-tree file, a file
/// changed between two refs, etc. `path` is repo-relative and the wire
/// identity of the item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawItem {
    pub path: String,
    pub extension: String,
    pub content: RawContent,
    /// Source-specific provenance (e.g. "modified", "added") — optional so
    /// the working-tree source can leave it `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// A chunk of one item. The identity (`source_path`) is denormalized onto
/// the chunk so wire consumers can group by file without re-deriving it,
/// and so `Rendered` is reconstructable from a `Selection` alone.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Chunk {
    pub source_path: String,
    pub extension: String,
    pub text: String,
    /// Ordinal within the source item (0 for whole-file identity chunks).
    pub index: usize,
}

/// A chunk with a relevance score. Generic at the trait boundary (see
/// `Ranker`), but the serializable form is monomorphized — a generic
/// `Scored<T>` does not produce a clean, stable JSON schema.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScoredChunk {
    pub chunk: Chunk,
    pub score: f32,
}

/// Per-selection token accounting. The single source of truth for the
/// reported count — computed from what the budgeter actually kept, once.
/// This is what makes the changed-files token bug structurally impossible:
/// there is no parallel "sum over all loaded files" to drift from.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TokenTally {
    pub total: usize,
    /// path -> tokens, for the per-file breakdown CLI/REST expose.
    pub by_path: std::collections::BTreeMap<String, usize>,
    pub encoding: String,
}

/// What the budgeter hands the renderer: the kept chunks plus the tally.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Selection {
    pub chunks: Vec<Chunk>,
    pub tally: TokenTally,
    /// Items the budgeter dropped to fit the budget (paths only), so a
    /// response can report what was omitted.
    pub omitted: Vec<String>,
}

/// Final rendered output. `format` names the template family so a consumer
/// knows how to treat `body` (markdown vs xml vs json).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Rendered {
    pub body: String,
    pub format: String,
    pub tally: TokenTally,
}
