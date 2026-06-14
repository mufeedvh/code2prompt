//! Configuration parsing and session creation utilities.
//!
//! This module handles the conversion of command-line arguments into
//! GnawSession instances, consolidating all configuration parsing
//! logic in one place for better maintainability and separation of concerns.

use anyhow::{Context, Result};
use gnaw_core::{
    configuration::{GnawConfig, TomlConfig},
    session::GnawSession,
    sort::FileSortMethod,
    template::{OutputFormat, extract_undefined_variables},
    tokenizer::TokenizerType,
};
use inquire::Text;
use log::error;
use std::path::PathBuf;

use crate::{args::Cli, config_loader::ConfigSource};
use gnaw_core::configuration::CompressionOptions;

const STRIP_TOKENS: [&str; 4] = ["tests", "fn-bodies", "doc-comments", "private-bodies"];
/// Unified session builder that merges configuration layering in one place
/// - base: Some(&ConfigSource) to use loaded config as defaults; None to use CLI defaults
/// - args: CLI arguments
/// - tui_mode: whether running in TUI mode (enables token map by default)
pub fn build_session(
    base: Option<&ConfigSource>,
    args: &Cli,
    tui_mode: bool,
) -> Result<GnawSession> {
    let mut configuration = GnawConfig::builder();

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

    // Merge CLI patterns onto config patterns instead of replacing wholesale.
    // A CLI --exclude should add to the configured ignores, not discard them
    // (replacing made `--exclude=*.toml` drop every config ignore and pull
    //  target/, lockfiles, etc. back in — inflating output instead of trimming it).
    let (cfg_include, cfg_exclude) = cfg
        .map(|c| (c.include_patterns.clone(), c.exclude_patterns.clone()))
        .unwrap_or_default();

    let mut include_patterns = cfg_include;
    include_patterns.extend(expand_comma_separated_patterns(&args.include));

    let mut exclude_patterns = cfg_exclude;
    exclude_patterns.extend(expand_comma_separated_patterns(&args.exclude));

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

    // Output format: CLI overrides config
    let output_format = if let Some(output_format_str) = args.output_format {
        output_format_str
    } else if let Some(c) = cfg {
        c.output_format.unwrap_or(OutputFormat::Markdown)
    } else {
        OutputFormat::Markdown
    };
    configuration.output_format(output_format);

    // Sort method: CLI overrides config
    let sort_method = if let Some(sort_str) = args.sort {
        sort_str
    } else if let Some(c) = cfg {
        c.sort_method.unwrap_or(FileSortMethod::NameAsc)
    } else {
        FileSortMethod::NameAsc
    };
    configuration.sort_method(sort_method);

    // Tokenizer settings: CLI overrides config
    let tokenizer_type = if let Some(encoding) = args.encoding {
        encoding
    } else if let Some(c) = cfg {
        c.encoding.unwrap_or(TokenizerType::Cl100kBase)
    } else {
        TokenizerType::Cl100kBase
    };

    // Token format: CLI overrides config
    let token_format = if let Some(format) = args.token_format {
        format
    } else if let Some(c) = cfg {
        c.token_format
            .unwrap_or(gnaw_core::tokenizer::TokenFormat::Format)
    } else {
        gnaw_core::tokenizer::TokenFormat::Format
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
    let cfg_deselected = cfg.map(|c| c.deselected).unwrap_or(false);

    let policy = args
        .secret_scan
        .or_else(|| cfg.and_then(|c| c.secret_scan))
        .unwrap_or_default(); // Warn
    configuration.secret_scan(policy);

    // CLI list wins if non-empty; else config list; else empty (→ path.rs defaults)
    let allow_paths = if !args.secret_scan_allow.is_empty() {
        args.secret_scan_allow.clone()
    } else {
        cfg.map(|c| c.secret_scan_allow_paths.clone())
            .unwrap_or_default()
    };
    configuration.secret_scan_allow_paths(allow_paths);

    configuration.compression(resolve_compression(args, cfg)?);

    configuration
        .diff_enabled(args.diff || cfg_diff_enabled)
        .diff_mode(args.diff_mode.unwrap_or_default())
        .diff_branches(diff_branches)
        .log_branches(log_branches)
        .no_ignore(args.no_ignore)
        .hidden(args.hidden)
        .no_codeblock(args.no_codeblock)
        .follow_symlinks(args.follow_symlinks)
        .token_map_enabled(args.token_map || cfg_token_map_enabled || tui_mode)
        .deselected(args.deselected || cfg_deselected);

    // User variables from config (if available)
    if let Some(c) = cfg {
        configuration.user_variables(c.user_variables.clone());
    }

    let session = GnawSession::new(configuration.build()?);
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

/// Resolves a template argument to its content, checking built-in templates by
/// name before falling back to reading a custom template file from disk.
///
/// # Arguments
///
/// * `template_arg` - An optional built-in template name (e.g. `claude-xml`,
///   `refactor`) or a path to a custom Handlebars template file
///
/// # Returns
///
/// * `Result<(String, String)>` - A tuple of (template_content, template_name).
///   The name is the built-in's key when matched, `"custom"` for a file loaded
///   from disk, or `"default"` when no argument was given. Errors if the
///   argument is neither a known built-in nor a readable file path.
pub fn parse_template(template_arg: &Option<String>) -> Result<(String, String)> {
    match template_arg {
        Some(arg) => {
            // 1. Builtin by name takes precedence.
            if let Some(t) = gnaw_core::builtin_templates::BuiltinTemplates::get_template(arg) {
                return Ok((t.content.to_string(), arg.clone()));
            }
            // 2. Otherwise treat it as a path to a custom template file.
            let content = std::fs::read_to_string(arg).with_context(|| {
                let keys = gnaw_core::builtin_templates::BuiltinTemplates::get_template_keys();
                format!(
                    "'{arg}' is not a built-in template and no file exists at that path.\n\
                     Available built-ins: {}",
                    keys.join(", ")
                )
            })?;
            Ok((content, "custom".to_string()))
        }
        None => Ok(("".to_string(), "default".to_string())),
    }
}

/// Handles user-defined variables in the template and adds them to the session.
///
/// This function extracts undefined variables from the template and prompts
/// the user to provide values for them through interactive input.
///
/// # Arguments
///
/// * `session` - The GnawSession to modify
/// * `template_content` - The template content string to analyze
///
/// # Returns
///
/// * `Result<()>` - An empty result indicating success or an error
pub fn handle_undefined_variables(session: &mut GnawSession, template_content: &str) -> Result<()> {
    let undefined_variables = extract_undefined_variables(template_content);

    for var in undefined_variables.iter() {
        // Check if variable is already defined in user_variables
        if !session.config.user_variables.contains_key(var) {
            let prompt = format!("Enter value for '{}': ", var);
            let answer = Text::new(&prompt)
                .with_help_message("Fill user defined variable in template")
                .prompt()
                .unwrap_or_default();
            session.config.user_variables.insert(var.clone(), answer);
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

fn resolve_compression(args: &Cli, cfg: Option<&TomlConfig>) -> Result<CompressionOptions> {
    // CLI preset wins; else config-file compression; else none. CSV overrides apply on top.
    let base = match args.compress {
        Some(level) => level.options(),
        None => cfg.and_then(|c| c.compression).unwrap_or_default(),
    };
    match &args.compress_strip {
        Some(csv) => apply_strip_overrides(base, csv),
        None => Ok(base),
    }
}

/// Apply CSV toggles over a baseline. Each token flips one flag; `no-` disables.
/// Errors loudly on unknown tokens — a silently-ignored typo would misreport the
/// token budget.
fn apply_strip_overrides(
    mut o: CompressionOptions,
    tokens: &[String],
) -> Result<CompressionOptions> {
    for raw in tokens {
        let raw = raw.trim();
        if raw.is_empty() {
            continue;
        }
        let (on, name) = match raw.strip_prefix("no-") {
            Some(rest) => (false, rest),
            None => (true, raw),
        };
        match name {
            "tests" => o.strip_test_modules = on,
            "fn-bodies" => o.strip_fn_bodies = on,
            "doc-comments" => o.strip_doc_comments = on,
            "private-bodies" => o.strip_private_bodies = on,
            other => {
                let hint = closest(other)
                    .map(|s| format!(" — did you mean '{s}'?"))
                    .unwrap_or_default();
                anyhow::bail!(
                    "unknown compression flag '{other}'{hint}\n  valid tokens: {} \
                     (each optionally prefixed with `no-` to disable)",
                    STRIP_TOKENS.join(", ")
                );
            }
        }
    }
    Ok(o)
}

fn closest(input: &str) -> Option<&'static str> {
    STRIP_TOKENS
        .iter()
        .map(|&t| (t, levenshtein(input, t)))
        .filter(|(_, d)| *d <= 3)
        .min_by_key(|(_, d)| *d)
        .map(|(t, _)| t)
}

fn levenshtein(a: &str, b: &str) -> usize {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr = vec![0usize; b.len() + 1];
    for (i, ca) in a.iter().enumerate() {
        curr[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            curr[j + 1] = (prev[j + 1] + 1).min(curr[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()]
}
