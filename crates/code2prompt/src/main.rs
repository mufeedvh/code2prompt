//! code2prompt is a command-line tool to generate an LLM prompt from a codebase directory.
//!
//! Authors: Olivier D'Ancona (@ODAncona), Mufeed VH (@mufeedvh)
mod args;
mod clipboard;
mod model;
mod token_map;
mod tui;
mod utils;

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
use std::{path::PathBuf, str::FromStr};
use tui::run_tui_with_args;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    info! {"Args: {:?}", std::env::args().collect::<Vec<_>>()};

    let args: Cli = Cli::parse();

    if args.tui {
        run_tui_with_args(args.path, args.include, args.exclude).await
    } else {
        run_cli_mode_with_args(args).await
    }
}

/// Run the CLI mode with parsed arguments
async fn run_cli_mode_with_args(args: Cli) -> Result<()> {
    // ~~~ Arguments Validation ~~~
    // if no_clipboard is true, output_file must be specified.
    if args.no_clipboard && args.output_file.is_none() {
        eprintln!("Error: --output-file is required when --no-clipboard is used.");
        std::process::exit(1);
    }

    // Disable clipboard when outputting to stdout (unless clipboard is explicitly enabled)
    let no_clipboard = args.no_clipboard || args.output_file.as_ref().is_some_and(|f| f == "-");

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

    // ~~~ Configuration ~~~
    let mut configuration = Code2PromptConfig::builder();

    configuration.path(args.path.clone());

    configuration
        .include_patterns(args.include)
        .exclude_patterns(args.exclude);

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
        .token_format(args.tokens);
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
        .token_map_enabled(args.token_map);

    // ~~~ Code2Prompt ~~~
    let mut session = Code2PromptSession::new(configuration.build()?);
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
    handle_undefined_variables(&mut data, &template_str)?;
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
        #[cfg(target_os = "linux")]
        {
            use clipboard::spawn_clipboard_daemon;
            spawn_clipboard_daemon(&rendered.prompt)?;
        }
        #[cfg(not(target_os = "linux"))]
        {
            use crate::clipboard::copy_text_to_clipboard;
            match copy_text_to_clipboard(&rendered.prompt) {
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
                    // Always print the prompt if clipboard fails, regardless of quiet mode
                    println!("{}", &rendered.prompt);
                }
            }
        }
    }

    // ~~~ Output File ~~~
    if let Some(output_path) = &args.output_file {
        write_to_file(output_path, &rendered.prompt, args.quiet)?;
    }

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
