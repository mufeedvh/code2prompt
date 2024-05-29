//! This module contains the functions to set up the Handlebars template engine and render the template with the provided data.
//! It also includes functions for handling user-defined variables, copying the rendered output to the clipboard, and writing it to a file.

use anyhow::{Context, Result};
use arboard::Clipboard;
use colored::*;
use handlebars::{no_escape, Handlebars};
use inquire::Text;
use regex::Regex;
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
        .map_err(|e| anyhow::anyhow!("Failed to register template: {}", e))?;

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
        .map_err(|e| anyhow::anyhow!("Failed to render template: {}", e))?;
    Ok(rendered.trim().to_string())
}

/// Handles user-defined variables in the template and adds them to the data.
///
/// # Arguments
///
/// * `data` - The JSON data object.
/// * `template_content` - The template content string.
///
/// # Returns
///
/// * `Result<()>` - An empty result indicating success or an error.
pub fn handle_undefined_variables(
    data: &mut serde_json::Value,
    template_content: &str,
) -> Result<()> {
    let undefined_variables = extract_undefined_variables(template_content);
    let mut user_defined_vars = serde_json::Map::new();

    for var in undefined_variables.iter() {
        if !data.as_object().unwrap().contains_key(var) {
            let prompt = format!("Enter value for '{}': ", var);
            let answer = Text::new(&prompt)
                .with_help_message("Fill user defined variable in template")
                .prompt()
                .unwrap_or_default();
            user_defined_vars.insert(var.clone(), serde_json::Value::String(answer));
        }
    }

    if let Some(obj) = data.as_object_mut() {
        for (key, value) in user_defined_vars {
            obj.insert(key, value);
        }
    }
    Ok(())
}

/// Copies the rendered template to the clipboard.
///
/// # Arguments
///
/// * `rendered` - The rendered template string.
///
/// # Returns
///
/// * `Result<()>` - An empty result indicating success or an error.
pub fn copy_to_clipboard(rendered: &str) -> Result<()> {
    let mut clipboard = Clipboard::new().expect("Failed to initialize clipboard");
    clipboard
        .set_text(rendered.to_string())
        .context("Failed to copy to clipboard")?;
    println!(
        "{}{}{} {}",
        "[".bold().white(),
        "✓".bold().green(),
        "]".bold().white(),
        "Prompt copied to clipboard!".green()
    );
    Ok(())
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
