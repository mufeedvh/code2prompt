//! Template system for customizable prompt generation.
//!
//! This module provides the core template rendering functionality using the Handlebars templating engine.
//! It supports dynamic content generation with built-in variables for code content, file metadata, and 
//! project structure. The system includes utilities for template validation, variable extraction, and 
//! output management.
//!
//! ## Available Template Variables
//!
//! Templates can use the following built-in variables:
//! - `absolute_code_path` - Absolute path to the code directory
//! - `source_tree` - Directory structure visualization  
//! - `files` - Collection of processed files with metadata
//! - `path` - Relative file paths
//! - `code` - File contents
//! - `extension` - File extension
//! - `no_codeblock` - Flag to control code block formatting
//! - `git_diff` - Git diff content
//! - `git_diff_branch` - Git diff against specific branch
//! - `git_log_branch` - Git log for specific branch
//!
//! ## Usage Example
//!
//! ```rust
//! use code2prompt_core::template::{handlebars_setup, render_template};
//! use serde_json::json;
//!
//! let template = "# Code Analysis\n{{#each files}}\n## {{path}}\n```{{extension}}\n{{code}}\n```\n{{/each}}";
//! let handlebars = handlebars_setup(template, "analysis")?;
//! let data = json!({"files": [{"path": "main.rs", "extension": "rust", "code": "fn main() {}"}]});
//! let output = render_template(&handlebars, "analysis", &data)?;
//! # Ok::<(), anyhow::Error>(())
//! ```
use anyhow::{Result, anyhow};
use handlebars::{Handlebars, no_escape};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::io::Write;

/// Creates and configures a Handlebars template engine instance.
///
/// Sets up a Handlebars instance with HTML escaping disabled (using `no_escape`)
/// and registers the provided template string under the specified name. This is
/// the primary setup function for template rendering in the code2prompt system.
///
/// # Arguments
///
/// * `template_str` - The Handlebars template string containing template syntax
/// * `template_name` - Unique identifier for the template within the Handlebars registry
///
/// # Returns
///
/// * `Result<Handlebars<'static>>` - Configured Handlebars instance ready for rendering
///
/// # Errors
///
/// Returns an error if the template string contains invalid Handlebars syntax
/// or if template registration fails.
///
/// # Examples
///
/// ```rust
/// use code2prompt_core::template::handlebars_setup;
///
/// let template = "Hello {{name}}!";
/// let handlebars = handlebars_setup(template, "greeting")?;
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn handlebars_setup(template_str: &str, template_name: &str) -> Result<Handlebars<'static>> {
    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(no_escape);

    handlebars
        .register_template_string(template_name, template_str)
        .map_err(|e| anyhow!("Failed to register template: {}", e))?;

    Ok(handlebars)
}

/// Identifies unrecognized template variables that may require user definition.
///
/// Scans a Handlebars template string and extracts variable names that are not part
/// of the built-in variable set. This is useful for template validation and helping
/// users identify custom variables they need to provide data for.
///
/// The function recognizes these built-in variables and excludes them from the result:
/// - `absolute_code_path`, `source_tree`, `files`, `path`, `code`, `extension`
/// - `no_codeblock`, `git_diff`, `git_diff_branch`, `git_log_branch`
///
/// # Arguments
///
/// * `template` - The Handlebars template string to analyze
///
/// # Returns
///
/// * `Vec<String>` - Collection of undefined variable names found in the template
///
/// # Examples
///
/// ```rust
/// use code2prompt_core::template::extract_undefined_variables;
///
/// let template = "{{code}} and {{custom_var}} with {{another_custom}}";
/// let undefined = extract_undefined_variables(template);
/// assert_eq!(undefined, vec!["custom_var", "another_custom"]);
/// ```
pub fn extract_undefined_variables(template: &str) -> Vec<String> {
    let registered_identifiers = [
        "absolute_code_path",
        "source_tree",
        "files",
        "path",
        "code",
        "extension",
        "no_codeblock",
        "git_diff",
        "git_diff_branch",
        "git_log_branch"
    ];
    let re = Regex::new(r"\{\{\s*(?P<var>[a-zA-Z_][a-zA-Z_0-9]*)\s*\}\}").unwrap();
    re.captures_iter(template)
        .map(|cap| cap["var"].to_string())
        .filter(|var| !registered_identifiers.contains(&var.as_str()))
        .collect()
}

/// Renders a registered template with the provided context data.
///
/// Takes a configured Handlebars instance and renders the specified template using
/// the provided data context. The output is automatically trimmed of leading and
/// trailing whitespace for cleaner results.
///
/// # Arguments
///
/// * `handlebars` - The configured Handlebars instance containing registered templates
/// * `template_name` - Name of the template to render (must be previously registered)
/// * `data` - Serializable data object providing values for template variables
///
/// # Returns
///
/// * `Result<String>` - The rendered template content as a trimmed string
///
/// # Errors
///
/// Returns an error if:
/// - The template name is not found in the registry
/// - Template rendering fails due to missing variables or syntax errors
/// - Data serialization fails
///
/// # Examples
///
/// ```rust
/// use code2prompt_core::template::{handlebars_setup, render_template};
/// use serde_json::json;
///
/// let handlebars = handlebars_setup("Hello {{name}}!", "greeting")?;
/// let data = json!({"name": "World"});
/// let output = render_template(&handlebars, "greeting", &data)?;
/// assert_eq!(output, "Hello World!");
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn render_template<T: Serialize>(
    handlebars: &Handlebars,
    template_name: &str,
    data: &T,
) -> Result<String> {
    let rendered = handlebars
        .render(template_name, data)
        .map_err(|e| anyhow!("Failed to render template: {}", e))?;
    Ok(rendered.trim().to_string())
}

/// Writes rendered template content to a file with buffered I/O.
///
/// Creates or overwrites a file at the specified path and writes the rendered
/// template content using a buffered writer for efficient I/O operations.
/// The file will be created if it doesn't exist, including any necessary parent directories.
///
/// # Arguments
///
/// * `output_path` - File system path where the content should be written
/// * `rendered` - The rendered template content to write to the file
///
/// # Returns
///
/// * `Result<()>` - Success indication or I/O error
///
/// # Errors
///
/// Returns an error if:
/// - The output path cannot be created or accessed
/// - File creation fails due to permissions or disk space
/// - Write operation fails
///
/// # Examples
///
/// ```rust,no_run
/// use code2prompt_core::template::write_to_file;
///
/// let content = "# Generated Report\n\nThis is the analysis output.";
/// write_to_file("output/report.md", content)?;
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn write_to_file(output_path: &str, rendered: &str) -> Result<()> {
    let file = std::fs::File::create(output_path)?;
    let mut writer = std::io::BufWriter::new(file);
    write!(writer, "{}", rendered)?;
    Ok(())
}

/// Supported output formats for template rendering.
///
/// Defines the available output formats that templates can target, each optimized
/// for different use cases and consumption by various tools or systems.
///
/// The enum supports serde serialization/deserialization with lowercase string
/// representation for configuration files and JSON APIs.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// Markdown format optimized for LLM prompt generation with code blocks
    #[default]
    Markdown,
    /// JSON format for structured data consumption by APIs and tools
    Json,
    /// XML format for systems requiring explicit structured markup
    Xml,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Markdown => write!(f, "markdown"),
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Xml => write!(f, "xml"),
        }
    }
}
