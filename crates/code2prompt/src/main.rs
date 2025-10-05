//! code2prompt is a command-line tool to generate an LLM prompt from a codebase directory.
//!
//! Authors: Olivier D'Ancona (@ODAncona), Mufeed VH (@mufeedvh)
mod args;
mod clipboard;
mod config;
mod config_loader;
mod model;
mod token_map;
mod tui;
mod utils;
mod view;
mod widgets;

use anyhow::{Context, Result};
use args::Cli;
use clap::Parser;
use code2prompt_core::{template::write_to_file, tokenizer::TokenFormat};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, error, info};
use num_format::{SystemLocale, ToFormattedString};
use std::io::Write;
use tui::run_tui;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    info! {"Args: {:?}", std::env::args().collect::<Vec<_>>()};

    let args: Cli = Cli::parse();

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

    // ~~~ TUI or CLI Mode ~~~
    if args.tui {
        // ~~~ Build Session for TUI ~~~
        let session = config::build_session(None, &args, args.tui).unwrap_or_else(|e| {
            error!("Failed to create session: {}", e);
            std::process::exit(1);
        });
        run_tui(session).await
    } else {
        run_cli_mode_with_args(args).await
    }
}

/// Run the CLI mode with parsed arguments
async fn run_cli_mode_with_args(args: Cli) -> Result<()> {
    use code2prompt_core::configuration::OutputDestination;
    use config_loader::{get_default_output_destination, load_config};

    let quiet_mode = args.quiet;

    // ~~~ Load Configuration ~~~
    // Always load config files first (local > global), then apply CLI args on top
    let config_source = load_config(quiet_mode)?;

    // ~~~ Build Session with config + CLI args ~~~
    let mut session = config::build_session(Some(&config_source), &args, false)?;

    // ~~~ Determine Output Behavior ~~~
    let default_output = get_default_output_destination(&config_source);

    // Determine final output destinations (Solution B: Unix-style behavior)
    let output_to_clipboard = if args.clipboard {
        // Explicit clipboard flag - ONLY clipboard, no stdout
        true
    } else if args.output_file.is_some() {
        // Output file specified, don't use clipboard unless explicitly requested
        false
    } else {
        // Use config default
        matches!(default_output, OutputDestination::Clipboard)
    };

    let output_to_stdout = if args.clipboard {
        // When -c is used, ONLY output to clipboard, not stdout
        false
    } else if let Some(ref output_file) = args.output_file {
        output_file == "-"
    } else {
        match default_output {
            OutputDestination::Stdout => true,
            OutputDestination::Clipboard => false,
            OutputDestination::File => false,
        }
    };

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
        s.set_message("Proceeding…".to_string());
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
        TokenFormat::Format => SystemLocale::default()
            .map(|loc| token_count.to_formatted_string(&loc))
            .unwrap_or_else(|_| token_count.to_string()),
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

    // ~~~ Output to Stdout (NEW DEFAULT BEHAVIOR) ~~~
    if output_to_stdout {
        print!("{}", &rendered.prompt);
        std::io::stdout()
            .flush()
            .context("Failed to flush stdout")?;
    }

    // ~~~ Copy to Clipboard ~~~
    if output_to_clipboard {
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
            }
        }
    }

    // ~~~ Output File ~~~
    if let Some(ref output_file) = args.output_file
        && output_file != "-"
    {
        output_prompt(
            Some(std::path::Path::new(output_file)),
            &rendered.prompt,
            quiet_mode,
        )?;
    }

    if let Some(ref s) = spinner {
        s.finish_with_message("Done!".green().to_string());
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
