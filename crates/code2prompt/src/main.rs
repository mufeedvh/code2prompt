//! code2prompt is a command-line tool to generate an LLM prompt from a codebase directory.
//!
//! Authors: Olivier D'Ancona (@ODAncona), Mufeed VH (@mufeedvh)
mod args;
mod clipboard;
mod config;
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
    session::Code2PromptSession, template::write_to_file, tokenizer::TokenFormat,
};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, error, info};
use num_format::{SystemLocale, ToFormattedString};
use std::io::IsTerminal;
use std::io::Write;
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
    let mut session = config::create_session_from_args(&args, args.tui).unwrap_or_else(|e| {
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
    let effective_output = args.output_file.clone();
    // Disable clipboard when outputting to stdout (unless clipboard is explicitly enabled)
    let no_clipboard = args.no_clipboard || effective_output.as_ref().is_some_and(|f| f == "-");
    let is_terminal = std::io::stdout().is_terminal();
    let quiet_mode = args.quiet || !is_terminal;

    // ~~~ Create Session ~~~
    let spinner = if !quiet_mode {
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
    config::handle_undefined_variables(&mut data, &session.config.template_str)?;
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

    if !quiet_mode {
        eprintln!(
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
                        if height > 20 { height - 10 } else { 10 }
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
                if !quiet_mode {
                    eprintln!(
                        "{}{}{} {}",
                        "[".bold().white(),
                        "✓".bold().green(),
                        "]".bold().white(),
                        "Copied to clipboard successfully.".green()
                    );
                }
            }
            Err(e) => {
                if !quiet_mode {
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
        !quiet_mode,
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
            eprintln!(
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
