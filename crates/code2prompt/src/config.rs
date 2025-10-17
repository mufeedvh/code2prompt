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
use std::path::PathBuf;

use crate::{args::Cli, config_loader::ConfigSource};

/// Unified session builder that merges configuration layering in one place
/// - base: Some(&ConfigSource) to use loaded config as defaults; None to use CLI defaults
/// - args: CLI arguments
/// - tui_mode: whether running in TUI mode (enables token map by default)
pub fn build_session(
    base: Option<&ConfigSource>,
    args: &Cli,
    tui_mode: bool,
) -> Result<Code2PromptSession> {
    let mut configuration = Code2PromptConfig::builder();

    let cfg = base.map(|b| &b.config);

    // Path: config path takes precedence if provided, otherwise CLI path
    if let Some(c) = cfg {
        if let Some(path) = &c.path {
            configuration.path(PathBuf::from(path));
        } else {
            configuration.path(args.path.clone());
        }
    } else {
        configuration.path(args.path.clone());
    }

    // Include/Exclude patterns:
    // If CLI provides any patterns, they override config patterns completely (to avoid conflicts)
    let use_cli_patterns = !args.include.is_empty() || !args.exclude.is_empty();
    let (include_patterns, exclude_patterns) = if use_cli_patterns {
        (
            expand_comma_separated_patterns(&args.include),
            expand_comma_separated_patterns(&args.exclude),
        )
    } else if let Some(c) = cfg {
        (c.include_patterns.clone(), c.exclude_patterns.clone())
    } else {
        (
            expand_comma_separated_patterns(&args.include),
            expand_comma_separated_patterns(&args.exclude),
        )
    };

    configuration
        .include_patterns(include_patterns)
        .exclude_patterns(exclude_patterns);

    // Display options: CLI overrides config (logical-or semantics for booleans)
    let cfg_line_numbers = cfg.map(|c| c.line_numbers).unwrap_or(false);
    let cfg_absolute = cfg.map(|c| c.absolute_path).unwrap_or(false);
    let cfg_full_tree = cfg.map(|c| c.full_directory_tree).unwrap_or(false);

    configuration
        .line_numbers(args.line_numbers || cfg_line_numbers)
        .absolute_path(args.absolute_paths || cfg_absolute)
        .full_directory_tree(args.full_directory_tree || cfg_full_tree);

    // Output format: CLI value
    configuration.output_format(args.output_format.clone());

    // Sort method: CLI overrides config
    let sort_method = if let Some(sort_str) = args.sort {
        sort_str
    } else if let Some(c) = cfg {
        c.sort_method.unwrap_or(FileSortMethod::NameAsc)
    } else {
        FileSortMethod::NameAsc
    };

    configuration.sort_method(sort_method);

    let tokenizer_type = if let Some(encoding) = args.encoding {
        encoding
    } else if let Some(c) = cfg {
        c.encoding.unwrap_or(TokenizerType::Cl100kBase)
    } else {
        TokenizerType::Cl100kBase
    };

    let token_format = if let Some(format) = args.token_format {
        format
    } else if let Some(c) = cfg {
        c.token_format
            .unwrap_or(code2prompt_core::tokenizer::TokenFormat::Format)
    } else {
        code2prompt_core::tokenizer::TokenFormat::Format
    };

    configuration
        .encoding(tokenizer_type)
        .token_format(token_format);

    // Template: CLI overrides config
    let (template_str, template_name) = if args.template.is_some() {
        parse_template(&args.template).map_err(|e| {
            error!("Failed to parse template: {}", e);
            e
        })?
    } else if let Some(c) = cfg {
        (
            c.template_str.clone().unwrap_or_default(),
            c.template_name
                .clone()
                .unwrap_or_else(|| "default".to_string()),
        )
    } else {
        ("".to_string(), "default".to_string())
    };

    configuration
        .template_str(template_str)
        .template_name(template_name);

    // Git options: CLI overrides config
    let diff_branches = parse_branch_argument(&args.git_diff_branch).or_else(|| {
        cfg.and_then(|c| {
            c.diff_branches.as_ref().and_then(|branches| {
                if branches.len() == 2 {
                    Some((branches[0].clone(), branches[1].clone()))
                } else {
                    None
                }
            })
        })
    });

    let log_branches = parse_branch_argument(&args.git_log_branch).or_else(|| {
        cfg.and_then(|c| {
            c.log_branches.as_ref().and_then(|branches| {
                if branches.len() == 2 {
                    Some((branches[0].clone(), branches[1].clone()))
                } else {
                    None
                }
            })
        })
    });

    let cfg_diff_enabled = cfg.map(|c| c.diff_enabled).unwrap_or(false);
    let cfg_token_map_enabled = cfg.map(|c| c.token_map_enabled).unwrap_or(false);

    configuration
        .diff_enabled(args.diff || cfg_diff_enabled)
        .diff_branches(diff_branches)
        .log_branches(log_branches)
        .no_ignore(args.no_ignore)
        .hidden(args.hidden)
        .no_codeblock(args.no_codeblock)
        .follow_symlinks(args.follow_symlinks)
        .token_map_enabled(args.token_map || cfg_token_map_enabled || tui_mode);

    // User variables from config (if available)
    if let Some(c) = cfg {
        configuration.user_variables(c.user_variables.clone());
    }

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
    if let Some(obj) = data.as_object() {
        for var in undefined_variables.iter() {
            if !obj.contains_key(var) {
                let prompt = format!("Enter value for '{}': ", var);
                let answer = Text::new(&prompt)
                    .with_help_message("Fill user defined variable in template")
                    .prompt()
                    .unwrap_or_default();
                user_defined_vars.insert(var.clone(), serde_json::Value::String(answer));
            }
        }
    } else {
        // Data is not an object; nothing to prompt for in this shape.
        return Ok(());
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
