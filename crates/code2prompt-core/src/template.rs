//! This module contains the functions to set up the Handlebars template engine and render the template with the provided data.
//! It also includes functions for handling user-defined variables, copying the rendered output to the clipboard, and writing it to a file.
use anyhow::{anyhow, Result};
use colored::*;
use handlebars::{
    no_escape, Context, Handlebars, Helper, HelperResult, Output, RenderContext,
};
use regex::Regex;
use std::io::Write;
use std::str::FromStr;

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
    
    // Register custom helper for first N lines
    handlebars.register_helper("firstLines", Box::new(|helper: &Helper, 
                                                     _: &Handlebars,
                                                     _: &Context,
                                                     _: &mut RenderContext,
                                                     out: &mut dyn Output| -> HelperResult {
        // Get the text parameter
        let text = helper.param(0).and_then(|v| v.value().as_str()).unwrap_or("");
        // Get the count parameter
        let count = helper.param(1).and_then(|v| v.value().as_u64()).unwrap_or(5) as usize;
        
        let lines: Vec<&str> = text.lines().collect();
        let first_n_lines = lines.iter().take(count).cloned().collect::<Vec<&str>>().join("\n");
        
        out.write(&first_n_lines)?;
        Ok(())
    }));

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
/// * `data` - The JSON data object.
///
/// # Returns
///
/// * `Result<String>` - The rendered template as a string.
pub fn render_template(
    handlebars: &Handlebars,
    template_name: &str,
    data: &serde_json::Value,
) -> Result<String> {
    let rendered = handlebars
        .render(template_name, data)
        .map_err(|e| anyhow!("Failed to render template: {}", e))?;
    Ok(rendered.trim().to_string())
}

/// Writes the rendered template to a specified output file.
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
    println!(
        "{}{}{} {}",
        "[".bold().white(),
        "✓".bold().green(),
        "]".bold().white(),
        format!("Prompt written to file: {}", output_path).green()
    );
    Ok(())
}

/// Enum to represent the output format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputFormat {
    Markdown,
    Json,
    Xml,
}

impl FromStr for OutputFormat {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "markdown" | "md" => Ok(OutputFormat::Markdown),
            "json" => Ok(OutputFormat::Json),
            "xml" => Ok(OutputFormat::Xml),
            _ => Err(anyhow!(
                "Invalid output format: {}. Allowed values: markdown, json, xml",
                s
            )),
        }
    }
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Markdown
    }
}
