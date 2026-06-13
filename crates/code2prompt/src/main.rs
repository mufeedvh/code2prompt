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
    info!("Args: {:?}", std::env::args().collect::<Vec<_>>());

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
        run_cli(args)
    }
}

/// Run the CLI mode with parsed arguments
fn run_cli(args: Cli) -> Result<()> {
    use config_loader::{get_default_output_destination, load_config};

    let quiet_mode = args.quiet;

    // ~~~ Load Configuration ~~~
    let config_source = load_config(quiet_mode)?;
    let mut session = config::build_session(Some(&config_source), &args, false)?;
    let default_output = get_default_output_destination(&config_source);

    // ~~~ Gather Repository Data ~~~
    gather_session_data(&mut session, quiet_mode)?;

    // ~~~ Template ~~~
    let rendered = render_session_template(&mut session)?;

    // ~~~ Emit Results ~~~
    emit_cli_results(rendered, &session, &args, &default_output)?;

    Ok(())
}

/// Loads codebase and git data into the session, driving the loading spinner.
fn gather_session_data(session: &mut Code2PromptSession, quiet: bool) -> Result<()> {
    let spinner = (!quiet).then(|| setup_spinner("Traversing directory and building tree..."));

    let fail_spinner = |e| {
        if let Some(s) = spinner.as_ref() {
            s.finish_with_message("Failed!".red().to_string());
        }
        e
    };

    session.load_codebase()
        .map_err(fail_spinner)
        .context("Failed to build directory tree")?;

    if let Some(s) = spinner.as_ref() { s.set_message("Proceeding…") }

    if session.config.diff_enabled {
        if let Some(s) = spinner.as_ref() { s.set_message("Generating git diff...") }
        session.load_git_diff().map_err(fail_spinner).context("Failed to generate git diff")?;
    }

    if session.config.diff_branches.is_some() {
        if let Some(s) = spinner.as_ref() { s.set_message("Generating git branch diff...") }
        session.load_git_diff_between_branches().map_err(fail_spinner).context("Failed to generate git branch diff")?;
    }

    if session.config.log_branches.is_some() {
        if let Some(s) = spinner.as_ref() { s.set_message("Generating git branch log...") }
        session.load_git_log_between_branches().map_err(fail_spinner).context("Failed to generate git branch log")?;
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

    let rendered = session
        .render_prompt(&data)
        .context("Failed to render prompt")?;

    Ok(rendered)
}

/// Dispatches the final rendered output to stdout, file, clipboard, and prints UI stats.
fn emit_cli_results(
    rendered: RenderedPrompt,
    session: &Code2PromptSession,
    args: &Cli,
    default_output: &OutputDestination,
) -> Result<()> {
    let quiet_mode = args.quiet;

    // ~~~ Token Count & Map Display ~~~ 
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

    // ~~~ Output Routing ~~~
    let is_stdout_explicit = args.output_file.as_deref() == Some("-");
    let has_file_target = args.output_file.is_some() && !is_stdout_explicit;

    // ~~~ File Output ~~~
    if has_file_target {
        write_prompt_to_file(
            std::path::Path::new(args.output_file.as_ref().unwrap()),
            &rendered.prompt,
            quiet_mode,
        )?;
    }

    // ~~~ Stdout Output ~~~
    let to_stdout = is_stdout_explicit || (!args.clipboard && !has_file_target && matches!(default_output, OutputDestination::Stdout));
    if to_stdout {
        print!("{}", &rendered.prompt);
        std::io::stdout()
            .flush()
            .context("Failed to flush stdout")?;
    }

    // ~~~ Clipboard Output ~~~
    let to_clipboard = args.clipboard || (!has_file_target && !is_stdout_explicit && matches!(default_output, OutputDestination::Clipboard));
    if to_clipboard {
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
