//! code2prompt is a command-line and TUI tool based on the code2prompt-core library to explore and analyze text repositories.

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

use crate::token_map::display_token_map;
use crate::utils::format_number;
use anyhow::{Context, Result};
use args::Cli;
use clap::Parser;
use code2prompt_core::analysis::TokenMapOptions;
use code2prompt_core::configuration::OutputDestination;
use code2prompt_core::session::{Code2PromptSession, RenderedPrompt};
use code2prompt_core::template::write_to_file;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, error, info};
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
        run_cli(args).await
    }
}

/// Run the CLI mode with parsed arguments
async fn run_cli(args: Cli) -> Result<()> {
    use config_loader::{get_default_output_destination, load_config};

    let quiet_mode = args.quiet;

    // ~~~ Load Configuration ~~~
    let config_source = load_config(quiet_mode)?;
    let mut session = config::build_session(Some(&config_source), &args, false)?;

    // ~~~ Determine Output Behavior ~~~
    let default_output = get_default_output_destination(&config_source);
    let (output_to_clipboard, output_to_stdout) = determine_output_targets(&args, &default_output);

    // ~~~ Gather Repository Data ~~~
    gather_session_data(&mut session, quiet_mode)?;

    // ~~~ Template ~~~
    let rendered = render_session_template(&mut session)?;

    // ~~~ Emit Results ~~~
    emit_cli_results(
        rendered,
        &session,
        &args,
        output_to_stdout,
        output_to_clipboard,
    )?;

    Ok(())
}

// ============================================================================
// Pipeline Helper Functions
// ============================================================================

/// Determines whether to output to clipboard, stdout, or both based on args and config.
fn determine_output_targets(args: &Cli, default_output: &OutputDestination) -> (bool, bool) {
    let output_to_clipboard = if args.clipboard {
        true
    } else if args.output_file.is_some() {
        false
    } else {
        matches!(default_output, OutputDestination::Clipboard)
    };

    let output_to_stdout = if args.clipboard {
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

    (output_to_clipboard, output_to_stdout)
}

/// Loads codebase and git data into the session, driving the loading spinner.
fn gather_session_data(session: &mut Code2PromptSession, quiet: bool) -> Result<()> {
    let spinner = if !quiet {
        Some(setup_spinner("Traversing directory and building tree..."))
    } else {
        None
    };

    session.load_codebase().map_err(|e| {
        if let Some(s) = spinner.as_ref() {
            s.finish_with_message("Failed!".red().to_string())
        }
        error!("Failed to build directory tree: \n{}", e);
        anyhow::anyhow!("Failed to build directory tree: {}", e)
    })?;

    if let Some(s) = spinner.as_ref() {
        s.set_message("Proceeding…")
    }

    if session.config.diff_enabled {
        if let Some(s) = spinner.as_ref() {
            s.set_message("Generating git diff...")
        }
        if let Err(e) = session.load_git_diff() {
            if let Some(s) = spinner.as_ref() { s.finish_with_message("Failed!".red().to_string()) }
            return Err(anyhow::anyhow!("Failed to generate git diff: {}", e));
        }
    }

    if session.config.diff_branches.is_some() {
        if let Some(s) = spinner.as_ref() {
            s.set_message("Generating git diff between two branches...")
        }
        if let Err(e) = session.load_git_diff_between_branches() {
            if let Some(s) = spinner.as_ref() { s.finish_with_message("Failed!".red().to_string()) }
            return Err(anyhow::anyhow!("Failed to generate git diff: {}", e));
        }
    }

    if session.config.log_branches.is_some() {
        if let Some(ref s) = spinner {
            s.set_message("Generating git log between two branches...");
        }
        if let Err(e) = session.load_git_log_between_branches() {
            if let Some(ref s) = spinner { s.finish_with_message("Failed!".red().to_string()); }
            return Err(anyhow::anyhow!("Failed to generate git log: {}", e));
        }
    }

    if let Some(s) = spinner {
        s.finish_with_message("Codebase Traversal Done!".green().to_string());
    }

    Ok(())
}

/// Handles undefined variables and invokes the template renderer.
fn render_session_template(session: &mut Code2PromptSession) -> Result<RenderedPrompt> {
    let template_str_clone = session.config.template_str.clone();
    config::handle_undefined_variables(session, &template_str_clone)?;

    let data = session.build_template_data();
    debug!(
        "Template Context: absolute_code_path={}, files_count={}, has_user_vars={}",
        data.absolute_code_path,
        data.files.map(|f| f.len()).unwrap_or(0),
        !session.config.user_variables.is_empty()
    );

    let rendered = session.render_prompt(&data)
        .context("Failed to render prompt")?;

    Ok(rendered)
}
/// Dispatches the final rendered output to stdout, file, clipboard, and prints UI stats.
fn emit_cli_results(
    rendered: RenderedPrompt,
    session: &Code2PromptSession,
    args: &Cli,
    output_to_stdout: bool,
    output_to_clipboard: bool,
) -> Result<()> {
    let quiet_mode = args.quiet;

    // ~~~ Token Count ~~~
    let token_count = rendered.token_count;
    let formatted_token_count = format_number(token_count, &session.config.token_format);
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
        let max_lines = args.token_map_lines.unwrap_or_else(|| {
            terminal_size::terminal_size()
                .map(|(_, terminal_size::Height(h))| {
                    let height = h as usize;
                    if height > 20 { height - 10 } else { 10 }
                })
                .unwrap_or(20)
        });

        let token_map_entries = session
            .contextual_analysis(&rendered)
            .map(|analysis| {
                analysis.token_map(TokenMapOptions {
                    max_lines,
                    min_percent: args.token_map_min_percent.unwrap_or(0.5),
                })
            })
            .unwrap_or_default();

        display_token_map(&token_map_entries, rendered.token_count);
    }

    // ~~~ Output to Stdout ~~~
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
        write_prompt_to_file(
            std::path::Path::new(output_file),
            &rendered.prompt,
            quiet_mode,
        )?;
    }

    Ok(())
}

/// Sets up a progress spinner with a given message
fn setup_spinner(message: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.enable_steady_tick(std::time::Duration::from_millis(220));
    let done_symbol = format!(
        "{}{}{}",
        "[".bold().white(),
        "✓".bold().green(),
        "]".bold().white()
    );
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&[
                "▹▹▹▹▹",
                "▸▹▹▹▹",
                "▹▸▹▹▹",
                "▹▹▸▹▹",
                "▹▹▹▸▹",
                "▹▹▹▹▸",
                &done_symbol,
            ])
            .template("{spinner:.blue} {msg}")
            .unwrap(),
    );
    spinner.set_message(message.to_string());
    spinner
}

// ~~~ Output to file ~~~
fn write_prompt_to_file(output_path: &std::path::Path, rendered: &str, quiet: bool) -> Result<()> {
    let path_str = output_path.to_string_lossy();

    write_to_file(&path_str, rendered).context(format!("Failed to write to file: {}", path_str))?;

    if !quiet {
        eprintln!(
            "{}{}{} {}",
            "[".bold().white(),
            "✓".bold().green(),
            "]".bold().white(),
            format!("Prompt written to file: {}", path_str).green()
        );
    }

    Ok(())
}
