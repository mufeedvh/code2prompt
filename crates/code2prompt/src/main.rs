//! code2prompt is a command-line tool to generate an LLM prompt from a codebase directory.
//!
//! Authors: Olivier D'Ancona (@ODAncona), Mufeed VH (@mufeedvh)
mod args;
mod clipboard;
mod model;
mod token_map;
mod tui;
mod utils;
mod view;
mod widgets;

use anyhow::{Context, Result};
use args::Cli;
use clap::Parser;
use code2prompt_core::{
    configuration::Code2PromptConfig,
    session::Code2PromptSession,
    sort::FileSortMethod,
    template::{extract_undefined_variables, write_to_file},
    tokenizer::{TokenFormat, TokenizerType},
};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use inquire::Text;
use log::{debug, error, info};
use num_format::{SystemLocale, ToFormattedString};
use std::io::Write;
use std::{path::PathBuf, str::FromStr};
use tui::run_tui_with_args;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    info! {"Args: {:?}", std::env::args().collect::<Vec<_>>()};

    let args: Cli = Cli::parse();

    // ~~~ Arguments Validation ~~~
    // if no_clipboard is true, output_file must be specified.
    if args.no_clipboard && args.output_file.is_none() {
        error!("Error: --output-file is required when --no-clipboard is used.");
        std::process::exit(1);
    }

    // ~~~ Clipboard Daemon ~~~
    #[cfg(target_os = "linux")]
    {
        use clipboard::serve_clipboard_daemon;
        if args.clipboard_daemon {
            info! {"Serving clipboard daemon..."};
            serve_clipboard_daemon()?;
            info! {"Shutting down gracefully..."};
            return Ok(());
        }
    }

    // ~~~ Build Session ~~~
    let mut session = create_session_from_args(&args, args.tui).unwrap_or_else(|e| {
        error!("Failed to create session: {}", e);
        std::process::exit(1);
    });

    // ~~~ TUI or CLI Mode ~~~
    if args.tui {
        run_tui_with_args(session).await
    } else {
        run_cli_mode_with_args(args, &mut session).await
    }
}

/// Run the CLI mode with parsed arguments
async fn run_cli_mode_with_args(args: Cli, session: &mut Code2PromptSession) -> Result<()> {
    // ~~~ Consolidate Arguments ~~~
    let effective_output = args.output_file.clone().or(args.output.clone());
    // Disable clipboard when outputting to stdout (unless clipboard is explicitly enabled)
    let no_clipboard = args.no_clipboard || effective_output.as_ref().is_some_and(|f| f == "-");

    // ~~~ Create Session ~~~
    let spinner = if !args.quiet {
        Some(setup_spinner("Traversing directory and building tree..."))
    } else {
        None
    };

    // ~~~ Gather Repository Data ~~~
    session.load_codebase().unwrap_or_else(|e| {
        if let Some(ref s) = spinner {
            s.finish_with_message("Failed!".red().to_string());
        }
        error!("Failed to build directory tree: {}", e);
        std::process::exit(1);
    });
    if let Some(ref s) = spinner {
        s.finish_with_message("Done!".green().to_string());
    }

    // ~~~ Git Related ~~~
    // Git Diff
    if session.config.diff_enabled {
        if let Some(ref s) = spinner {
            s.set_message("Generating git diff...");
        }
        session.load_git_diff().unwrap_or_else(|e| {
            if let Some(ref s) = spinner {
                s.finish_with_message("Failed!".red().to_string());
            }
            error!("Failed to generate git diff: {}", e);
            std::process::exit(1);
        });
    }

    // Load Git diff between branches if provided
    if session.config.diff_branches.is_some() {
        if let Some(ref s) = spinner {
            s.set_message("Generating git diff between two branches...");
        }
        session
            .load_git_diff_between_branches()
            .unwrap_or_else(|e| {
                if let Some(ref s) = spinner {
                    s.finish_with_message("Failed!".red().to_string());
                }
                error!("Failed to generate git diff: {}", e);
                std::process::exit(1);
            });
    }

    // Load Git log between branches if provided
    if session.config.log_branches.is_some() {
        if let Some(ref s) = spinner {
            s.set_message("Generating git log between two branches...");
        }
        session.load_git_log_between_branches().unwrap_or_else(|e| {
            if let Some(ref s) = spinner {
                s.finish_with_message("Failed!".red().to_string());
            }
            error!("Failed to generate git log: {}", e);
            std::process::exit(1);
        });
    }

    if let Some(ref s) = spinner {
        s.finish_with_message("Done!".green().to_string());
    }

    // ~~~ Template ~~~

    // Data
    let mut data = session.build_template_data();
    handle_undefined_variables(&mut data, &session.config.template_str)?;
    debug!(
        "JSON Data: {}",
        serde_json::to_string_pretty(&data).unwrap()
    );

    // Render
    let rendered = session.render_prompt(&data).unwrap_or_else(|e| {
        error!("Failed to render prompt: {}", e);
        std::process::exit(1);
    });

    // ~~~ Token Count ~~~
    let token_count = rendered.token_count;
    let formatted_token_count: String = match session.config.token_format {
        TokenFormat::Raw => token_count.to_string(),
        TokenFormat::Format => token_count.to_formatted_string(&SystemLocale::default().unwrap()),
    };
    let model_info = rendered.model_info;

    if !args.quiet {
        println!(
            "{}{}{} Token count: {}, Model info: {}",
            "[".bold().white(),
            "i".bold().blue(),
            "]".bold().white(),
            formatted_token_count,
            model_info
        );
    }

    // ~~~ Token Map Display ~~~
    if args.token_map {
        use crate::token_map::{display_token_map, generate_token_map_with_limit};

        if let Some(files) = session.data.files.as_ref().and_then(|f| f.as_array()) {
            // Calculate total tokens from individual file counts
            let total_from_files: usize = files
                .iter()
                .filter_map(|f| f.get("token_count"))
                .filter_map(|tc| tc.as_u64())
                .map(|tc| tc as usize)
                .sum();

            // Get max lines from command line or calculate from terminal height
            let max_lines = args.token_map_lines.unwrap_or_else(|| {
                terminal_size::terminal_size()
                    .map(|(_, terminal_size::Height(h))| {
                        let height = h as usize;
                        // Ensure minimum of 10 lines, subtract 10 for other output
                        if height > 20 {
                            height - 10
                        } else {
                            10
                        }
                    })
                    .unwrap_or(20) // Default to 20 lines if terminal size detection fails
            });

            // Use the sum of individual file tokens for the map with line limit
            let entries = generate_token_map_with_limit(
                files,
                total_from_files,
                Some(max_lines),
                args.token_map_min_percent,
            );
            display_token_map(&entries, total_from_files);
        }
    }

    // ~~~ Copy to Clipboard ~~~
    if !no_clipboard {
        use crate::clipboard::copy_to_clipboard;
        match copy_to_clipboard(&rendered.prompt) {
            Ok(_) => {
                if !args.quiet {
                    println!(
                        "{}{}{} {}",
                        "[".bold().white(),
                        "✓".bold().green(),
                        "]".bold().white(),
                        "Copied to clipboard successfully.".green()
                    );
                }
            }
            Err(e) => {
                if !args.quiet {
                    eprintln!(
                        "{}{}{} {}",
                        "[".bold().white(),
                        "!".bold().red(),
                        "]".bold().white(),
                        format!("Failed to copy to clipboard: {}", e).red()
                    );
                }
                // optional: fallback
                println!("{}", &rendered.prompt);
            }
        }
    }

    // ~~~ Output File ~~~
    output_prompt(
        effective_output.as_deref().map(std::path::Path::new),
        &rendered.prompt,
        !args.quiet,
    )?;

    Ok(())
}

/// Sets up a progress spinner with a given message
///
/// # Arguments
///
/// * `message` - A message to display with the spinner
///
/// # Returns
///
/// * `ProgressBar` - The configured progress spinner
fn setup_spinner(message: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.enable_steady_tick(std::time::Duration::from_millis(120));
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["▹▹▹▹▹", "▸▹▹▹▹", "▹▸▹▹▹", "▹▹▸▹▹", "▹▹▹▸▹", "▹▹▹▹▸"])
            .template("{spinner:.blue} {msg}")
            .unwrap(),
    );
    spinner.set_message(message.to_string());
    spinner
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
fn parse_branch_argument(branch_arg: &Option<Vec<String>>) -> Option<(String, String)> {
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

/// Expands comma-separated patterns while preserving brace expansion patterns
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

/// Create a Code2PromptSession from command line arguments
fn create_session_from_args(args: &Cli, tui_mode: bool) -> Result<Code2PromptSession> {
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

// ~~~ Output to file or stdout ~~~
fn output_prompt(
    effective_output: Option<&std::path::Path>,
    rendered: &str,
    quiet: bool,
) -> Result<()> {
    let output_path = match effective_output {
        Some(path) => path,
        None => return Ok(()), // nothing to do
    };

    let path_str = output_path.to_string_lossy();
    if path_str == "-" {
        // stdout
        print!("{}", rendered);
        std::io::stdout()
            .flush()
            .context("Failed to flush stdout")?;
    } else {
        // file
        write_to_file(&path_str, rendered)
            .context(format!("Failed to write to file: {}", path_str))?;

        if !quiet {
            println!(
                "{}{}{} {}",
                "[".bold().white(),
                "✓".bold().green(),
                "]".bold().white(),
                format!("Prompt written to file: {}", path_str).green()
            );
        }
    }

    Ok(())
}
