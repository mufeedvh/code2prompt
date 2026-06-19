//! Cross-stage data types. These ARE the wire schema (CLI --json, REST, MCP).
//! All are serde-serializable with stable field names.

use super::*;

/// Raw content as it leaves a source, before chunking.
///
/// `Text` is a plain file body (working-tree source). `Changed` is a file
/// changed between two refs — modeled as its own kind because a changed file
/// is genuinely a different thing than a file: it carries a *diff*, which is
/// a property of a ref-pair, not of the file. `Omitted` is binary/over-size.
///
/// Which of `before`/`patch` a `Changed` carries depends on the diff-shas
/// content mode — they are `Option` precisely because the producer
/// (`get_changed_files_with_contents`) populates a different subset per mode.
/// `after` is non-optional: a changed file always has a current state to show
/// (a deletion is represented by the source omitting the item or via status).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum RawContent {
    Text {
        text: String,
    },
    Changed {
        after: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        before: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        patch: Option<String>,
    },
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
    /// Rename source, repo-relative. `Some` only for renamed files (changed-
    /// files source); `None` for working-tree files and non-renames.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_path: Option<String>,
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
    /// Measured token cost of `text`. 0 until the budget/count stage fills it
    /// (the chunker emits chunks uncounted). Carried so split can pack files
    /// without re-tokenizing.
    #[serde(default)]
    pub tokens: usize,
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
    /// Secret-scan findings collected by the Scrubber stage, independent of
    /// what the budgeter kept — a finding in a budget-dropped file is still
    /// reported (it was still read off disk). Empty when scanning is Off.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub findings: Vec<FindingDto>,
    /// Kept chunks, surfaced so a frontend can re-assemble subsets (split)
    /// without re-extracting. Same `Chunk`s the budgeter produced — token
    /// counts already filled. Empty for consumers that only want `body`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub chunks: Vec<Chunk>,
    /// Full source tree, surfaced so split reuses it across all parts (the
    /// tree is constant per part; only file contents vary).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub source_tree: String,
}

/// A secret-scan finding, wire form. Distinct from `secret_scan::Finding`
/// (which holds a `&'static str` rule id for the scanner's sake and can't
/// round-trip through serde). Owned fields so REST/MCP can both emit and
/// accept it. The preview is already redacted by the scanner — never the
/// full secret.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FindingDto {
    pub path: String,
    pub rule_id: String,
    pub line: usize,
    pub entropy: f32,
    pub preview: String,
}

impl FindingDto {
    /// Build from a path-tagged core finding.
    pub fn from_core(path: String, f: &crate::secret_scan::Finding) -> Self {
        Self {
            path,
            rule_id: f.rule_id.to_string(),
            line: f.line,
            entropy: f.entropy,
            preview: f.preview.clone(),
        }
    }
}
