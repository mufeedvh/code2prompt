//! Built-in templates embedded as static resources.
//!
//! This module provides access to all built-in templates that are embedded
//! directly into the binary, making them available even when the crate is
//! installed from crates.io without access to the source file structure.

use std::{collections::HashMap, sync::OnceLock};

/// Information about a built-in template
#[derive(Debug, Clone, Copy)]
pub struct BuiltinTemplate {
    pub name: &'static str,
    pub content: &'static str,
    pub description: &'static str,
}

/// All built-in templates embedded as static strings
pub struct BuiltinTemplates;

static TEMPLATES: OnceLock<HashMap<&'static str, BuiltinTemplate>> = OnceLock::new();

impl BuiltinTemplates {
    /// Get all available built-in templates
    pub fn get_all() -> &'static HashMap<&'static str, BuiltinTemplate> {
        TEMPLATES.get_or_init(|| {
            HashMap::from([
                (
                    "default-markdown",
                    BuiltinTemplate {
                        name: "Default (Markdown)",
                        content: include_str!("default_template_md.hbs"),
                        description: "Default markdown template for code analysis",
                    },
                ),
                (
                    "default-xml",
                    BuiltinTemplate {
                        name: "Default (XML)",
                        content: include_str!("default_template_xml.hbs"),
                        description: "Default XML template for code analysis",
                    },
                ),
                (
                    "binary-exploitation-ctf-solver",
                    BuiltinTemplate {
                        name: "Binary Exploitation CTF Solver",
                        content: include_str!("../templates/binary-exploitation-ctf-solver.hbs"),
                        description: "Template for solving binary exploitation CTF challenges",
                    },
                ),
                (
                    "clean-up-code",
                    BuiltinTemplate {
                        name: "Clean Up Code",
                        content: include_str!("../templates/clean-up-code.hbs"),
                        description: "Template for code cleanup and refactoring",
                    },
                ),
                (
                    "cryptography-ctf-solver",
                    BuiltinTemplate {
                        name: "Cryptography CTF Solver",
                        content: include_str!("../templates/cryptography-ctf-solver.hbs"),
                        description: "Template for solving cryptography CTF challenges",
                    },
                ),
                (
                    "document-the-code",
                    BuiltinTemplate {
                        name: "Document the Code",
                        content: include_str!("../templates/document-the-code.hbs"),
                        description: "Template for generating code documentation",
                    },
                ),
                (
                    "find-security-vulnerabilities",
                    BuiltinTemplate {
                        name: "Find Security Vulnerabilities",
                        content: include_str!("../templates/find-security-vulnerabilities.hbs"),
                        description: "Template for security vulnerability analysis",
                    },
                ),
                (
                    "fix-bugs",
                    BuiltinTemplate {
                        name: "Fix Bugs",
                        content: include_str!("../templates/fix-bugs.hbs"),
                        description: "Template for bug fixing and debugging",
                    },
                ),
                (
                    "improve-performance",
                    BuiltinTemplate {
                        name: "Improve Performance",
                        content: include_str!("../templates/improve-performance.hbs"),
                        description: "Template for performance optimization",
                    },
                ),
                (
                    "refactor",
                    BuiltinTemplate {
                        name: "Refactor",
                        content: include_str!("../templates/refactor.hbs"),
                        description: "Template for code refactoring",
                    },
                ),
                (
                    "reverse-engineering-ctf-solver",
                    BuiltinTemplate {
                        name: "Reverse Engineering CTF Solver",
                        content: include_str!("../templates/reverse-engineering-ctf-solver.hbs"),
                        description: "Template for solving reverse engineering CTF challenges",
                    },
                ),
                (
                    "web-ctf-solver",
                    BuiltinTemplate {
                        name: "Web CTF Solver",
                        content: include_str!("../templates/web-ctf-solver.hbs"),
                        description: "Template for solving web CTF challenges",
                    },
                ),
                (
                    "write-git-commit",
                    BuiltinTemplate {
                        name: "Write Git Commit",
                        content: include_str!("../templates/write-git-commit.hbs"),
                        description: "Template for generating git commit messages",
                    },
                ),
                (
                    "write-github-pull-request",
                    BuiltinTemplate {
                        name: "Write GitHub Pull Request",
                        content: include_str!("../templates/write-github-pull-request.hbs"),
                        description: "Template for generating GitHub pull request descriptions",
                    },
                ),
                (
                    "write-github-readme",
                    BuiltinTemplate {
                        name: "Write GitHub README",
                        content: include_str!("../templates/write-github-readme.hbs"),
                        description: "Template for generating GitHub README files",
                    },
                ),
            ])
        })
    }

    /// Get a specific template by its key
    pub fn get_template(key: &str) -> Option<BuiltinTemplate> {
        Self::get_all().get(key).cloned()
    }

    /// Get all template keys
    pub fn get_template_keys() -> Vec<&'static str> {
        Self::get_all().keys().copied().collect()
    }

    /// Check if a template exists
    pub fn has_template(key: &str) -> bool {
        Self::get_all().contains_key(key)
    }
}
