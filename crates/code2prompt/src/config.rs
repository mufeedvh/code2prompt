//! Configuration parsing and session creation utilities.
//!
//! This module handles the conversion of command-line arguments into
//! Code2PromptSession instances, consolidating all configuration parsing
//! logic in one place for better maintainability and separation of concerns.

use anyhow::{Context, Result};
use code2prompt_core::{
    configuration::Code2PromptConfig, session::Code2PromptSession, sort::FileSortMethod,
    template::extract_undefined_variables, tokenizer::TokenizerType,
};
use inquire::Text;
use log::error;
use std::{path::PathBuf, str::FromStr};

use crate::args::Cli;

/// Create a Code2PromptSession from command line arguments
///
/// # Arguments
///
/// * `args` - The parsed command line arguments
/// * `tui_mode` - Whether the application is running in TUI mode
///
/// # Returns
///
/// * `Result<Code2PromptSession>` - The configured session or an error
pub fn create_session_from_args(args: &Cli, tui_mode: bool) -> Result<Code2PromptSession> {
    let mut configuration = Code2PromptConfig::builder();

    configuration.path(args.path.clone());

    // Handle comma-separated patterns, but preserve brace expansion patterns
    let include_patterns = expand_comma_separated_patterns(&args.include);
    let exclude_patterns = expand_comma_separated_patterns(&args.exclude);

    configuration
        .include_patterns(include_patterns)
        .exclude_patterns(exclude_patterns);

    let output_format = args.output_format.clone();
    configuration
        .line_numbers(args.line_numbers)
        .absolute_path(args.absolute_paths)
        .full_directory_tree(args.full_directory_tree)
        .output_format(output_format);

    let sort_method = args
        .sort
        .as_deref()
        .map(FileSortMethod::from_str)
        .transpose()
        .unwrap_or_else(|err| {
            eprintln!("{}", err);
            std::process::exit(1);
        })
        .unwrap_or(FileSortMethod::NameAsc);

    configuration.sort_method(sort_method);

    let tokenizer_type = args
        .encoding
        .as_deref()
        .unwrap_or("cl100k")
        .parse::<TokenizerType>()
        .unwrap_or_default();

    configuration
        .encoding(tokenizer_type)
        .token_format(args.tokens.clone());

    let (template_str, template_name) = parse_template(&args.template).unwrap_or_else(|e| {
        error!("Failed to parse template: {}", e);
        std::process::exit(1);
    });

    configuration
        .template_str(template_str.clone())
        .template_name(template_name);

    let diff_branches = parse_branch_argument(&args.git_diff_branch);
    let log_branches = parse_branch_argument(&args.git_log_branch);

    configuration
        .diff_enabled(args.diff)
        .diff_branches(diff_branches)
        .log_branches(log_branches);

    configuration
        .no_ignore(args.no_ignore)
        .hidden(args.hidden)
        .no_codeblock(args.no_codeblock)
        .follow_symlinks(args.follow_symlinks)
        .token_map_enabled(args.token_map || tui_mode);

    let session = Code2PromptSession::new(configuration.build()?);
    Ok(session)
}

/// Parses the branch argument from command line options.
///
/// Takes an optional vector of strings and converts it to a tuple of two branch names
/// if exactly two branches are provided.
///
/// # Arguments
///
/// * `branch_arg` - An optional vector containing branch names
///
/// # Returns
///
/// * `Option<(String, String)>` - A tuple of (from_branch, to_branch) if two branches were provided, None otherwise
pub fn parse_branch_argument(branch_arg: &Option<Vec<String>>) -> Option<(String, String)> {
    match branch_arg {
        Some(branches) if branches.len() == 2 => Some((branches[0].clone(), branches[1].clone())),
        _ => None,
    }
}

/// Loads a template from a file path or returns default values.
///
/// # Arguments
///
/// * `template_arg` - An optional path to a template file
///
/// # Returns
///
/// * `Result<(String, String)>` - A tuple containing (template_content, template_name)
///   where template_name is "custom" for user-provided templates or "default" otherwise
pub fn parse_template(template_arg: &Option<PathBuf>) -> Result<(String, String)> {
    match template_arg {
        Some(path) => {
            let template_str =
                std::fs::read_to_string(path).context("Failed to load custom template file")?;
            Ok((template_str, "custom".to_string()))
        }
        None => Ok(("".to_string(), "default".to_string())),
    }
}

/// Handles user-defined variables in the template and adds them to the data.
///
/// This function extracts undefined variables from the template and prompts
/// the user to provide values for them through interactive input.
///
/// # Arguments
///
/// * `data` - The JSON data object to modify
/// * `template_content` - The template content string to analyze
///
/// # Returns
///
/// * `Result<()>` - An empty result indicating success or an error
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

/// Expands comma-separated patterns while preserving brace expansion patterns
///
/// This function handles the expansion of comma-separated include/exclude patterns
/// while being careful not to split patterns that contain brace expansion syntax.
///
/// # Arguments
///
/// * `patterns` - A vector of pattern strings that may contain comma-separated values
///
/// # Returns
///
/// * `Vec<String>` - A vector of individual patterns
fn expand_comma_separated_patterns(patterns: &[String]) -> Vec<String> {
    let mut expanded = Vec::new();

    for pattern in patterns {
        // If the pattern contains braces, don't split on commas (preserve brace expansion)
        if pattern.contains('{') && pattern.contains('}') {
            expanded.push(pattern.clone());
        } else {
            // Split on commas for regular patterns
            for part in pattern.split(',') {
                let trimmed = part.trim();
                if !trimmed.is_empty() {
                    expanded.push(trimmed.to_string());
                }
            }
        }
    }

    expanded
}
