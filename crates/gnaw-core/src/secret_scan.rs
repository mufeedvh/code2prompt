// crates/gnaw-core/src/secret_scan.rs
//! Regex + entropy secret detection. Pure, no I/O. Behind the `secret-scan` feature.
//!
//! The `SecretScanner` trait is the seam: the default `RegexRuleScanner` is
//! deterministic; a future checksum or model-based scorer can implement the same
//! trait without touching callers.

use regex::Regex;
use serde::{Deserialize, Serialize};

pub static SCANNER: once_cell::sync::Lazy<crate::secret_scan::RegexRuleScanner> =
    once_cell::sync::Lazy::new(crate::secret_scan::RegexRuleScanner::default);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[serde(rename_all = "lowercase")]
pub enum SecretPolicy {
    /// Don't scan.
    Off,
    /// Include content unchanged; report findings.
    #[default]
    Warn,
    /// Replace each detected secret with a placeholder, then include.
    Redact,
    /// Treat any finding as fatal for that file (caller drops it / fails the run).
    Block,
}

#[derive(Debug, Clone)]
pub struct Finding {
    pub rule_id: &'static str,
    pub line: usize,
    pub entropy: f32,
    /// Redacted preview — never the full secret.
    pub preview: String,
}

/// Port: detect secrets in content. Pure; fakeable in tests; swappable later.
pub trait SecretScanner {
    fn scan(&self, content: &str) -> Vec<Finding>;
    /// Scan and rewrite per policy. For Off/Warn/Block, content is returned
    /// unchanged (Block is enforced by the caller using the findings).
    fn scrub(&self, content: &str, policy: SecretPolicy) -> (String, Vec<Finding>);
}

struct Rule {
    id: &'static str,
    re: Regex,
    min_entropy: f32,
    /// Capture group whose value is entropy-gated and redacted. 0 = whole match.
    group: usize,
}

pub struct RegexRuleScanner {
    rules: Vec<Rule>,
    allow_values: Vec<Regex>, // known false positives (doc/example keys)
}

impl Default for RegexRuleScanner {
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl RegexRuleScanner {
    pub fn with_defaults() -> Self {
        // PANIC: these are compile-time-constant patterns; a failure here is a
        // programmer error in this file, not reachable from user input.
        fn r(id: &'static str, pat: &str, min_entropy: f32, group: usize) -> Rule {
            Rule {
                id,
                re: Regex::new(pat).expect("invalid builtin secret regex"),
                min_entropy,
                group,
            }
        }
        let rules = vec![
            r("aws-access-key-id", r"\bAKIA[0-9A-Z]{16}\b", 3.0, 0),
            r("github-pat", r"\bgh[pousr]_[0-9A-Za-z]{36}\b", 3.5, 0),
            r(
                "github-fine-grained-pat",
                r"\bgithub_pat_[0-9A-Za-z_]{82}\b",
                3.5,
                0,
            ),
            r("slack-token", r"\bxox[baprs]-[0-9A-Za-z-]{10,48}\b", 3.5, 0),
            r("google-api-key", r"\bAIza[0-9A-Za-z_\-]{35}\b", 3.5, 0),
            r("anthropic-key", r"\bsk-ant-[0-9A-Za-z_\-]{20,}\b", 3.5, 0),
            r(
                "openai-key",
                r"\bsk-(?:proj-)?[0-9A-Za-z_\-]{20,}\b",
                3.5,
                0,
            ),
            r(
                "stripe-secret-key",
                r"\b[rs]k_live_[0-9A-Za-z]{20,}\b",
                3.5,
                0,
            ),
            r(
                "jwt",
                r"\beyJ[0-9A-Za-z_\-]{10,}\.[0-9A-Za-z_\-]{10,}\.[0-9A-Za-z_\-]{10,}\b",
                3.5,
                0,
            ),
            r(
                "private-key-block",
                r"(?s)-----BEGIN (?:RSA |EC |OPENSSH |PGP |DSA )?PRIVATE KEY-----.*?-----END (?:RSA |EC |OPENSSH |PGP |DSA )?PRIVATE KEY-----",
                0.0,
                0,
            ),
            r("crates-io-token", r"\bcio[0-9A-Za-z]{32,}\b", 4.0, 0),
            r(
                "generic-assigned-secret",
                r#"(?i)(?:secret|token|passwd|password|api[_-]?key|cf[_-]?api[_-]?token|cloudflare[_-]?\w*token|cf[_-]?api[_-]?key)["'\s]*[:=]\s*["']([0-9A-Za-z/+_\-]{20,})["']"#,
                4.0,
                1,
            ),
        ];
        let allow_values = [
            "AKIAIOSFODNN7EXAMPLE",
            r"EXAMPLE",
            r"(?i)dummy|placeholder|xxxx+",
        ]
        .into_iter()
        .map(|p| Regex::new(p).expect("invalid builtin allowlist regex"))
        .collect();
        Self {
            rules,
            allow_values,
        }
    }

    fn allowlisted(&self, value: &str) -> bool {
        self.allow_values.iter().any(|re| re.is_match(value))
    }
}

impl SecretScanner for RegexRuleScanner {
    fn scan(&self, content: &str) -> Vec<Finding> {
        let mut findings = Vec::new();
        for rule in &self.rules {
            for caps in rule.re.captures_iter(content) {
                let target = caps.get(rule.group).or_else(|| caps.get(0)).unwrap();
                let value = target.as_str();
                let entropy = shannon_entropy(value);
                if self.allowlisted(value) || entropy < rule.min_entropy {
                    continue;
                }
                findings.push(Finding {
                    rule_id: rule.id,
                    line: line_of(content, target.start()),
                    entropy,
                    preview: redact_preview(value),
                });
            }
        }
        findings.sort_by(|a, b| a.line.cmp(&b.line).then(a.rule_id.cmp(b.rule_id)));
        findings
    }

    fn scrub(&self, content: &str, policy: SecretPolicy) -> (String, Vec<Finding>) {
        let findings = self.scan(content);
        if policy != SecretPolicy::Redact {
            return (content.to_string(), findings);
        }
        let mut out = content.to_string();
        for rule in &self.rules {
            out = rule
                .re
                .replace_all(&out, |caps: &regex::Captures| {
                    let whole = caps.get(0).unwrap();
                    let target = caps.get(rule.group).unwrap_or(whole);
                    let value = target.as_str();
                    if self.allowlisted(value) || shannon_entropy(value) < rule.min_entropy {
                        return whole.as_str().to_string();
                    }
                    if rule.group == 0 {
                        format!("[REDACTED: {}]", rule.id)
                    } else {
                        let w = whole.as_str();
                        let (s, e) = (target.start() - whole.start(), target.end() - whole.start());
                        format!("{}[REDACTED: {}]{}", &w[..s], rule.id, &w[e..])
                    }
                })
                .into_owned();
        }
        (out, findings)
    }
}

fn shannon_entropy(s: &str) -> f32 {
    if s.is_empty() {
        return 0.0;
    }
    let mut counts = [0usize; 256];
    for &b in s.as_bytes() {
        counts[b as usize] += 1;
    }
    let len = s.len() as f32;
    counts
        .iter()
        .filter(|&&c| c > 0)
        .map(|&c| {
            let p = c as f32 / len;
            -p * p.log2()
        })
        .sum()
}

fn line_of(content: &str, byte_offset: usize) -> usize {
    content[..byte_offset]
        .bytes()
        .filter(|&b| b == b'\n')
        .count()
        + 1
}

fn redact_preview(value: &str) -> String {
    let n = value.chars().count();
    let head: String = value.chars().take(4).collect();
    format!("{head}… ({n} chars)") // prefix is identifying, not secret; body never shown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_and_redacts_github_pat() {
        let s = RegexRuleScanner::with_defaults();
        let text = "token = ghp_0123456789abcdefABCDEF0123456789abcdef";
        let (out, findings) = s.scrub(text, SecretPolicy::Redact);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "github-pat");
        assert!(out.contains("[REDACTED: github-pat]"));
        assert!(!out.contains("ghp_0123"));
    }

    #[test]
    fn aws_example_key_is_allowlisted() {
        let s = RegexRuleScanner::with_defaults();
        assert!(s.scan("key = AKIAIOSFODNN7EXAMPLE").is_empty());
    }

    #[test]
    fn warn_leaves_content_intact() {
        let s = RegexRuleScanner::with_defaults();
        let text = "ghp_0123456789abcdefABCDEF0123456789abcdef";
        let (out, findings) = s.scrub(text, SecretPolicy::Warn);
        assert_eq!(out, text);
        assert_eq!(findings.len(), 1);
    }
}
