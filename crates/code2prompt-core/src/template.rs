//! This module contains the functions to set up the Handlebars template engine and render the template with the provided data.
//! It also includes functions for handling user-defined variables, copying the rendered output to the clipboard, and writing it to a file.
use anyhow::{Result, anyhow};
use handlebars::{Handlebars, no_escape};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::io::Write;

/// Set up the Handlebars template engine with a template string and a template name.
///
/// # Arguments
///
/// * `template_str` - The Handlebars template string.
/// * `template_name` - The name of the template.
///
/// # Returns
///
/// * `Result<Handlebars<'static>>` - The configured Handlebars instance.
pub fn handlebars_setup(template_str: &str, template_name: &str) -> Result<Handlebars<'static>> {
    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(no_escape);

    handlebars
        .register_template_string(template_name, template_str)
        .map_err(|e| anyhow!("Failed to register template: {}", e))?;

    Ok(handlebars)
}

/// Extracts the undefined variables from the template string.
///
/// # Arguments
///
/// * `template` - The Handlebars template string.
///
/// # Returns
///
/// * `Vec<String>` - A vector of undefined variable names.
pub fn extract_undefined_variables(template: &str) -> Vec<String> {
    let registered_identifiers = ["path", "code", "git_diff"];
    let re = Regex::new(r"\{\{\s*(?P<var>[a-zA-Z_][a-zA-Z_0-9]*)\s*\}\}").unwrap();
    re.captures_iter(template)
        .map(|cap| cap["var"].to_string())
        .filter(|var| !registered_identifiers.contains(&var.as_str()))
        .collect()
}

/// Renders the template with the provided data.
///
/// # Arguments
///
/// * `handlebars` - The configured Handlebars instance.
/// * `template_name` - The name of the template.
/// * `data` - Any serializable data object.
///
/// # Returns
///
/// * `Result<String>` - The rendered template as a string.
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

/// Writes the rendered template to a specified output file
///
/// # Arguments
///
/// * `output_path` - The path to the output file.
/// * `rendered` - The rendered template string.
///
/// # Returns
///
/// * `Result<()>` - An empty result indicating success or an error.
pub fn write_to_file(output_path: &str, rendered: &str) -> Result<()> {
    let file = std::fs::File::create(output_path)?;
    let mut writer = std::io::BufWriter::new(file);
    write!(writer, "{}", rendered)?;
    Ok(())
}

/// Enum to represent the output format.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    #[default]
    Markdown,
    Json,
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
