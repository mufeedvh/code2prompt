//! Entity-level code map via [sem-core](https://github.com/Ataraxy-Labs/sem).
//!
//! When the `entity-map` feature is enabled, code2prompt extracts the structural
//! entities (functions, classes, methods, ...) from each source file using
//! sem-core's tree-sitter parsers. The result is exposed to templates both
//! per-file (`FileEntry.entities`) and as a top-level `code_map` aggregate, so a
//! prompt can include a compact outline of the codebase instead of, or alongside,
//! full file contents.
//!
//! sem-core is offline and emits no telemetry, so enabling this does not change
//! code2prompt's privacy posture.

use serde::{Deserialize, Serialize};

/// A single structural entity (function, class, method, ...) within a file.
///
/// This is a deliberately small projection of sem-core's internal entity type:
/// it carries only what a prompt template needs (name, kind, line range,
/// signature, parent), not source bodies or content hashes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EntitySummary {
    /// Entity name, e.g. `process_single_file`.
    pub name: String,
    /// Entity kind as reported by sem-core, e.g. `function`, `class`, `method`.
    pub kind: String,
    /// 1-based first line of the entity.
    pub start_line: usize,
    /// 1-based last line of the entity.
    pub end_line: usize,
    /// First line of the entity's source (its signature/declaration), trimmed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    /// Name of the enclosing entity (e.g. the class a method belongs to), if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
}

/// A file paired with its entity outline, used for the top-level `code_map`
/// template variable (an aggregate view alongside the per-file `entities`).
#[derive(Debug, Clone, Serialize)]
pub struct FileCodeMap {
    pub path: String,
    pub entities: Vec<EntitySummary>,
}

/// Extract the entity outline for one file's contents.
///
/// `file_path` is used by sem-core to pick the right language parser (by
/// extension). Returns an empty vector for files in languages sem-core does not
/// parse, so it is safe to call on every file.
#[cfg(feature = "entity-map")]
pub fn extract_entities(file_path: &str, content: &str) -> Vec<EntitySummary> {
    use sem_core::parser::plugins::create_default_registry;
    use sem_core::parser::registry::ParserRegistry;
    use std::cell::RefCell;
    use std::collections::HashMap;

    // One registry per worker thread: building it registers every language
    // plugin, so we amortize that across files rather than paying it per file,
    // while staying thread-safe inside code2prompt's rayon file pipeline.
    // NOTE: `ParserRegistry::new()` is empty; `create_default_registry()` is the
    // populated one the sem CLI uses.
    thread_local! {
        static REGISTRY: RefCell<ParserRegistry> = RefCell::new(create_default_registry());
    }

    REGISTRY.with(|cell| {
        let registry = cell.borrow();
        let entities = registry.extract_entities(file_path, content);

        // Resolve parent_id -> parent name so methods can show their class.
        let name_by_id: HashMap<&str, &str> = entities
            .iter()
            .map(|e| (e.id.as_str(), e.name.as_str()))
            .collect();

        entities
            .iter()
            .map(|e| {
                let signature = e
                    .content
                    .lines()
                    .next()
                    .map(|l| l.trim().to_string())
                    .filter(|s| !s.is_empty());
                let parent = e
                    .parent_id
                    .as_deref()
                    .and_then(|pid| name_by_id.get(pid).map(|n| n.to_string()));
                EntitySummary {
                    name: e.name.clone(),
                    kind: e.entity_type.clone(),
                    start_line: e.start_line,
                    end_line: e.end_line,
                    signature,
                    parent,
                }
            })
            .collect()
    })
}

/// No-op when the `entity-map` feature is disabled, so the rest of the codebase
/// compiles and runs identically without the sem-core dependency.
#[cfg(not(feature = "entity-map"))]
pub fn extract_entities(_file_path: &str, _content: &str) -> Vec<EntitySummary> {
    Vec::new()
}

#[cfg(all(test, feature = "entity-map"))]
mod tests {
    use super::*;

    #[test]
    fn extracts_rust_entities() {
        let src = "pub struct Cache { size: usize }\n\nimpl Cache {\n    pub fn new(size: usize) -> Self { Cache { size } }\n}\n\nfn helper(x: i32) -> i32 { x * 2 }\n";
        let got = extract_entities("util.rs", src);
        assert!(!got.is_empty(), "expected entities, got none: {got:?}");
        assert!(got.iter().any(|e| e.name == "helper"));
    }

    #[test]
    fn extracts_python_entities() {
        let src = "class Calculator:\n    def add(self, a, b):\n        return a + b\n\ndef main():\n    pass\n";
        let got = extract_entities("math.py", src);
        assert!(!got.is_empty(), "expected entities, got none: {got:?}");
    }
}
