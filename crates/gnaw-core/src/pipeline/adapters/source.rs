//! The two launch sources. "Working-tree files" and "files changed between
//! two refs" are genuinely different sources, not one source plus a filter.

use crate::configuration::DiffMode;
use crate::configuration::GnawConfig;
use crate::git::{
    get_branch_changed_paths, get_changed_files_with_contents, get_working_tree_changed_paths,
};
use crate::path::{RawFile, extract_raw_file};
use crate::pipeline::{ContextSource, PipelineError, RawContent, RawItem, SourceOpts};
use crate::secret_scan::Finding;
use ignore::WalkBuilder;
use std::path::PathBuf;

/// Wraps the legacy working-tree walk. Discovery + per-file raw extraction,
/// reusing the same ignore/hidden rules as `traverse_directory`. Yields raw
/// content; wrapping and counting happen downstream.
pub struct WorkingTreeSource {
    config: GnawConfig,
    /// TEMPORARY (2.5): findings collected during extraction, surfaced here
    /// because they have no DTO home yet. The Scrubber stage will own these.
    findings: std::sync::Mutex<Vec<(String, Finding)>>,
}

impl WorkingTreeSource {
    pub fn new(config: GnawConfig) -> Self {
        Self {
            config,
            findings: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Drain findings accumulated during the last `items` call.
    pub fn take_findings(&self) -> Vec<(String, Finding)> {
        std::mem::take(&mut self.findings.lock().unwrap())
    }
}

impl ContextSource for WorkingTreeSource {
    fn items(&self, _opts: &SourceOpts) -> Result<Vec<RawItem>, PipelineError> {
        let root = self
            .config
            .path
            .canonicalize()
            .map_err(|e| PipelineError::Source(format!("canonicalize root: {e}")))?;

        let walker = WalkBuilder::new(&root)
            .hidden(!self.config.hidden)
            .git_ignore(!self.config.no_ignore)
            .follow_links(self.config.follow_symlinks)
            .build()
            .filter_map(|e| e.ok());

        let mut items = Vec::new();
        let mut all_findings = Vec::new();

        for entry in walker {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Ok(rel) = path.strip_prefix(&root) else {
                continue;
            };
            if let Some(RawFile {
                path: p,
                extension: ext,
                code,
                findings,
            }) = extract_raw_file(path, rel, &self.config)
            {
                all_findings.extend(findings);
                items.push(RawItem {
                    path: p,
                    extension: ext,
                    content: RawContent::Text { text: code },
                    status: None,
                    old_path: None,
                });
            }
        }

        // Deterministic order — snapshots and wire output must be byte-stable.
        items.sort_by(|a, b| a.path.cmp(&b.path));
        *self.findings.lock().unwrap() = all_findings;
        Ok(items)
    }
}

/// Wraps `get_changed_files_with_contents`. Yields one item per changed file
/// with its `after` content (or marks it omitted for binary/absent). Does NOT
/// walk the working tree — that's the whole reason the token bug dies here.
pub struct CommitRangeSource {
    config: GnawConfig,
    ref1: String,
    ref2: String,
}

impl CommitRangeSource {
    pub fn new(config: GnawConfig, ref1: String, ref2: String) -> Self {
        Self { config, ref1, ref2 }
    }
}

impl ContextSource for CommitRangeSource {
    fn items(&self, _opts: &SourceOpts) -> Result<Vec<RawItem>, PipelineError> {
        let changed = get_changed_files_with_contents(
            &self.config.path,
            &self.ref1,
            &self.ref2,
            self.config.diff_shas_content,
            self.config.diff_shas_max_bytes,
        )
        .map_err(|e| PipelineError::Source(format!("changed files: {e}")))?;

        let mut items: Vec<RawItem> = changed
            .into_iter()
            .map(|cf| {
                let extension = PathBuf::from(&cf.path)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_string();
                // Step 2: surface the `after` body as the item's content,
                // mirroring what git-diff-shas.hbs renders. Patch/before
                // handling stays in the renderer for now — this source is a
                // straight wrap, not a redesign of the changed-files format.
                let content = if cf.binary {
                    RawContent::Omitted
                } else if let Some(after) = cf.after {
                    // Lossless: carry whatever this mode populated. `before`
                    // and `patch` ride along; the renderer (step 4) decides
                    // how to present them per the changed-files format.
                    RawContent::Changed {
                        after,
                        before: cf.before,
                        patch: cf.patch,
                    }
                } else if let Some(patch) = cf.patch {
                    // Patch-only mode (no `after` blob): still lossless —
                    // model it as a Changed with an empty after and the patch.
                    RawContent::Changed {
                        after: String::new(),
                        before: cf.before,
                        patch: Some(patch),
                    }
                } else {
                    RawContent::Omitted
                };
                RawItem {
                    path: cf.path,
                    extension,
                    content,
                    status: Some(cf.status.to_string()),
                    old_path: cf.old_path,
                }
            })
            .collect();

        items.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(items)
    }
}

/// Where a changed-paths run gets its file list from.
pub enum ChangedScope {
    /// Working-tree changes (commit / changeset runs), per `DiffMode`.
    WorkingTree(DiffMode),
    /// Changes between two refs (PR runs): (ref1, ref2).
    Refs(String, String),
}

/// Yields changed-file *paths* with NO content (`RawContent::Omitted`). The diff
/// body is rendered as chrome by the frontend; this source exists so the source
/// tree lists only changed files in commit/changeset/PR runs, with zero content
/// extraction — no walk, no read, no tokenize of file bodies.
///
/// Contrast `CommitRangeSource`, which carries per-file patch content for the
/// `--git-diff-shas` per-file view. Both produce a changed-files tree; only that
/// one renders the changes inline.
pub struct ChangedPathsSource {
    config: GnawConfig,
    scope: ChangedScope,
}

impl ChangedPathsSource {
    pub fn new(config: GnawConfig, scope: ChangedScope) -> Self {
        Self { config, scope }
    }
}

impl ContextSource for ChangedPathsSource {
    fn items(&self, _opts: &SourceOpts) -> Result<Vec<RawItem>, PipelineError> {
        let changed = match &self.scope {
            ChangedScope::WorkingTree(mode) => {
                get_working_tree_changed_paths(&self.config.path, *mode)
                    .map_err(|e| PipelineError::Source(format!("working-tree changes: {e}")))?
            }
            ChangedScope::Refs(r1, r2) => get_branch_changed_paths(&self.config.path, r1, r2)
                .map_err(|e| PipelineError::Source(format!("branch changes: {e}")))?,
        };

        let mut items: Vec<RawItem> = changed
            .into_iter()
            .map(|cf| {
                let extension = PathBuf::from(&cf.path)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_string();
                RawItem {
                    path: cf.path,
                    extension,
                    content: RawContent::Omitted, // paths only — diff is chrome
                    status: Some(cf.status.to_string()),
                    old_path: cf.old_path,
                }
            })
            .collect();

        // Deterministic for snapshots / byte-stable trees.
        items.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(items)
    }
}
