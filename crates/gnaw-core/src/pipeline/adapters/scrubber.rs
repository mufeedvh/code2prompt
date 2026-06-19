//! Secret-scan stage. Runs before chunking, scans whole-file (or whole-diff)
//! content, and—per policy—scrubs it. Collects findings independent of the
//! budgeter, so a finding survives even if its file's chunk is later dropped.
//!
//! This is where the secret scan lives now; `extract_raw_file` no longer scans
//! (it yields genuinely raw content, as its name claims). `Off` is a fast
//! passthrough so the no-scan path costs nothing.

use crate::configuration::GnawConfig;
use crate::pipeline::{FindingDto, RawContent, RawItem, Scrubber};
use crate::secret_scan::{SCANNER, SecretPolicy, SecretScanner};

pub struct SecretScrubber {
    policy: SecretPolicy,
    allow_paths: Vec<String>,
}

impl SecretScrubber {
    pub fn new(config: &GnawConfig) -> Self {
        Self {
            policy: config.secret_scan,
            allow_paths: config.secret_scan_allow_paths.clone(),
        }
    }

    /// Scrub one text field, appending findings tagged with `path`. Returns the
    /// (possibly redacted) text. For non-Redact policies the text is unchanged;
    /// findings are still collected.
    fn scrub_field(&self, path: &str, text: &str, out: &mut Vec<FindingDto>) -> String {
        let (scrubbed, found) = SCANNER.scrub(text, self.policy);
        for f in &found {
            out.push(FindingDto::from_core(path.to_string(), f));
        }
        scrubbed
    }
}

impl Scrubber for SecretScrubber {
    fn scrub(&self, items: Vec<RawItem>) -> (Vec<RawItem>, Vec<FindingDto>) {
        if self.policy == SecretPolicy::Off {
            return (items, Vec::new());
        }

        let mut findings = Vec::new();
        let scrubbed_items = items
            .into_iter()
            .map(|item| {
                if path_is_allowlisted(&item.path, &self.allow_paths) {
                    return item;
                }

                // Per-item finding window, so we can dedup within this item
                // (a secret in both `patch` and `after` is one finding, not two).
                let start = findings.len();

                let content = match item.content {
                    RawContent::Text { text } => {
                        let scrubbed = self.scrub_field(&item.path, &text, &mut findings);
                        RawContent::Text { text: scrubbed }
                    }
                    RawContent::Changed {
                        after,
                        before,
                        patch,
                    } => {
                        // Scrub every populated field — a secret can live in the
                        // added lines (after/patch +), removed lines (before/patch -),
                        // or both. Missing a field would leave a hole.
                        let after = self.scrub_field(&item.path, &after, &mut findings);
                        let before =
                            before.map(|b| self.scrub_field(&item.path, &b, &mut findings));
                        let patch = patch.map(|p| self.scrub_field(&item.path, &p, &mut findings));
                        RawContent::Changed {
                            after,
                            before,
                            patch,
                        }
                    }
                    // Binary/omitted: nothing to scan.
                    RawContent::Omitted => RawContent::Omitted,
                };

                // Dedup this item's findings by (rule_id, line) — diff fields
                // re-surface the same secret across before/after/patch.
                dedup_item_findings(&mut findings, start);

                RawItem { content, ..item }
            })
            .collect();

        (scrubbed_items, findings)
    }
}

/// Dedup findings added for one item (indices `start..`) by (rule_id, line),
/// keeping first occurrence. In place, order-preserving.
fn dedup_item_findings(findings: &mut Vec<FindingDto>, start: usize) {
    let mut seen = std::collections::HashSet::new();
    let mut write = start;
    for read in start..findings.len() {
        let key = (findings[read].rule_id.clone(), findings[read].line);
        if seen.insert(key) {
            findings.swap(write, read);
            write += 1;
        }
    }
    findings.truncate(write);
}

/// Same allowlist check the legacy path used — substring match, with a builtin
/// default set when the config list is empty. Lifted from path.rs so the
/// Scrubber owns its own policy logic.
fn path_is_allowlisted(path: &str, allow_paths: &[String]) -> bool {
    const DEFAULTS: &[&str] = &[
        "/tests/",
        "/test/",
        "/fixtures/",
        "/testdata/",
        "/__tests__/",
        "_test.",
    ];
    if allow_paths.is_empty() {
        DEFAULTS.iter().any(|frag| path.contains(frag))
    } else {
        allow_paths.iter().any(|frag| path.contains(frag.as_str()))
    }
}
