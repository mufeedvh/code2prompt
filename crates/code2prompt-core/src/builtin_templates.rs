//! Built-in templates embedded as static resources.
//!
//! This module provides access to all built-in templates that are embedded
//! directly into the binary, making them available even when the crate is
//! installed from crates.io without access to the source file structure.

use std::collections::HashMap;

/// Information about a built-in template
#[derive(Debug, Clone)]
pub struct BuiltinTemplate {
    pub name: String,
    pub content: String,
    pub description: String,
}

/// All built-in templates embedded as static strings
pub struct BuiltinTemplates;

impl BuiltinTemplates {
    /// Get all available built-in templates
    pub fn get_all() -> HashMap<String, BuiltinTemplate> {
        let mut templates = HashMap::new();

        // Default Markdown Template
        templates.insert(
            "default-markdown".to_string(),
            BuiltinTemplate {
                name: "Default (Markdown)".to_string(),
                content: include_str!("default_template_md.hbs").to_string(),
                description: "Default markdown template for code analysis".to_string(),
            },
        );

        // Default XML Template
        templates.insert(
            "default-xml".to_string(),
            BuiltinTemplate {
                name: "Default (XML)".to_string(),
                content: include_str!("default_template_xml.hbs").to_string(),
                description: "Default XML template for code analysis".to_string(),
            },
        );

        // Binary Exploitation CTF Solver
        templates.insert(
            "binary-exploitation-ctf-solver".to_string(),
            BuiltinTemplate {
                name: "Binary Exploitation CTF Solver".to_string(),
                content: include_str!("../templates/binary-exploitation-ctf-solver.hbs")
                    .to_string(),
                description: "Template for solving binary exploitation CTF challenges".to_string(),
            },
        );

        // Clean Up Code
        templates.insert(
            "clean-up-code".to_string(),
            BuiltinTemplate {
                name: "Clean Up Code".to_string(),
                content: include_str!("../templates/clean-up-code.hbs").to_string(),
                description: "Template for code cleanup and refactoring".to_string(),
            },
        );

        // Cryptography CTF Solver
        templates.insert(
            "cryptography-ctf-solver".to_string(),
            BuiltinTemplate {
                name: "Cryptography CTF Solver".to_string(),
                content: include_str!("../templates/cryptography-ctf-solver.hbs").to_string(),
                description: "Template for solving cryptography CTF challenges".to_string(),
            },
        );

        // Document the Code
        templates.insert(
            "document-the-code".to_string(),
            BuiltinTemplate {
                name: "Document the Code".to_string(),
                content: include_str!("../templates/document-the-code.hbs").to_string(),
                description: "Template for generating code documentation".to_string(),
            },
        );

        // Find Security Vulnerabilities
        templates.insert(
            "find-security-vulnerabilities".to_string(),
            BuiltinTemplate {
                name: "Find Security Vulnerabilities".to_string(),
                content: include_str!("../templates/find-security-vulnerabilities.hbs").to_string(),
                description: "Template for security vulnerability analysis".to_string(),
            },
        );

        // Fix Bugs
        templates.insert(
            "fix-bugs".to_string(),
            BuiltinTemplate {
                name: "Fix Bugs".to_string(),
                content: include_str!("../templates/fix-bugs.hbs").to_string(),
                description: "Template for bug fixing and debugging".to_string(),
            },
        );

        // Improve Performance
        templates.insert(
            "improve-performance".to_string(),
            BuiltinTemplate {
                name: "Improve Performance".to_string(),
                content: include_str!("../templates/improve-performance.hbs").to_string(),
                description: "Template for performance optimization".to_string(),
            },
        );

        // Refactor
        templates.insert(
            "refactor".to_string(),
            BuiltinTemplate {
                name: "Refactor".to_string(),
                content: include_str!("../templates/refactor.hbs").to_string(),
                description: "Template for code refactoring".to_string(),
            },
        );

        // Reverse Engineering CTF Solver
        templates.insert(
            "reverse-engineering-ctf-solver".to_string(),
            BuiltinTemplate {
                name: "Reverse Engineering CTF Solver".to_string(),
                content: include_str!("../templates/reverse-engineering-ctf-solver.hbs")
                    .to_string(),
                description: "Template for solving reverse engineering CTF challenges".to_string(),
            },
        );

        // Web CTF Solver
        templates.insert(
            "web-ctf-solver".to_string(),
            BuiltinTemplate {
                name: "Web CTF Solver".to_string(),
                content: include_str!("../templates/web-ctf-solver.hbs").to_string(),
                description: "Template for solving web CTF challenges".to_string(),
            },
        );

        // Write Git Commit
        templates.insert(
            "write-git-commit".to_string(),
            BuiltinTemplate {
                name: "Write Git Commit".to_string(),
                content: include_str!("../templates/write-git-commit.hbs").to_string(),
                description: "Template for generating git commit messages".to_string(),
            },
        );

        // Write GitHub Pull Request
        templates.insert(
            "write-github-pull-request".to_string(),
            BuiltinTemplate {
                name: "Write GitHub Pull Request".to_string(),
                content: include_str!("../templates/write-github-pull-request.hbs").to_string(),
                description: "Template for generating GitHub pull request descriptions".to_string(),
            },
        );

        // Write GitHub README
        templates.insert(
            "write-github-readme".to_string(),
            BuiltinTemplate {
                name: "Write GitHub README".to_string(),
                content: include_str!("../templates/write-github-readme.hbs").to_string(),
                description: "Template for generating GitHub README files".to_string(),
            },
        );

        templates
    }

    /// Get a specific template by its key
    pub fn get_template(key: &str) -> Option<BuiltinTemplate> {
        Self::get_all().get(key).cloned()
    }

    /// Get all template keys
    pub fn get_template_keys() -> Vec<String> {
        Self::get_all().keys().cloned().collect()
    }

    /// Check if a template exists
    pub fn has_template(key: &str) -> bool {
        Self::get_all().contains_key(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_templates_loaded() {
        let templates = BuiltinTemplates::get_all();
        assert!(!templates.is_empty());

        // Verify some expected templates exist
        assert!(templates.contains_key("claude-xml"));
        assert!(templates.contains_key("clean-up-code"));
        assert!(templates.contains_key("document-the-code"));
    }

    #[test]
    fn test_get_specific_template() {
        let template = BuiltinTemplates::get_template("claude-xml");
        assert!(template.is_some());

        let template = template.unwrap();
        assert_eq!(template.name, "Claude XML");
        assert!(!template.content.is_empty());
    }

    #[test]
    fn test_template_keys() {
        let keys = BuiltinTemplates::get_template_keys();
        assert!(!keys.is_empty());
        assert!(keys.contains(&"claude-xml".to_string()));
    }
}
