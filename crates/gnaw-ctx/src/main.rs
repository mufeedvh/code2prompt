//! gnaw is a command-line tool to generate an LLM prompt from a codebase directory.
//!
//! Authors: Olivier D'Ancona (@ODAncona), Mufeed VH (@mufeedvh)
mod args;
mod banner;
mod clipboard;
mod config;
mod config_loader;
mod model;
mod token_map;
mod tui;
mod utils;
mod view;
mod widgets;

use crate::utils::format_number;
use anyhow::{Context, Result};
use args::Cli;
use clap::Parser;
use colored::*;
use gnaw_core::path::FileEntry;
use gnaw_core::template::write_to_file;
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
        run_cli_mode_with_args(args).await
    }
}

/// Run the CLI mode with parsed arguments
async fn run_cli_mode_with_args(args: Cli) -> Result<()> {
    use config_loader::{get_default_output_destination, load_config};
    use gnaw_core::configuration::OutputDestination;

    let quiet_mode = args.quiet;

    // ~~~ Load Configuration ~~~
    let config_source = load_config(quiet_mode)?; // load config files first (local > global), then apply CLI args on top

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

    if !output_to_stdout {
        banner::print_cli(quiet_mode);
    }

    // ~~~ Create Session ~~~
    let spinner = if !quiet_mode {
        Some(setup_spinner("Traversing directory and building tree..."))
    } else {
        None
    };

    // ~~~ Gather Repository Data ~~~
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

    // ~~~ Git Related ~~~
    // Git Diff
    if session.config.diff_enabled {
        if let Some(s) = spinner.as_ref() {
            s.set_message("Generating git diff...")
        }
        session.load_git_diff().unwrap_or_else(|e| {
            if let Some(s) = spinner.as_ref() {
                s.finish_with_message("Failed!".red().to_string())
            }
            error!("Failed to generate git diff: {}", e);
            std::process::exit(1);
        });
    }

    // Load Git diff between branches if provided
    if session.config.diff_branches.is_some() {
        if let Some(s) = spinner.as_ref() {
            s.set_message("Generating git diff between two branches...")
        }
        session
            .load_git_diff_between_branches()
            .unwrap_or_else(|e| {
                if let Some(s) = spinner.as_ref() {
                    s.finish_with_message("Failed!".red().to_string())
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

    // Handle undefined variables (modifies session.config.user_variables)
    let template_str_clone = session.config.template_str.clone();
    match spinner.as_ref() {
        Some(s) => {
            s.suspend(|| config::handle_undefined_variables(&mut session, &template_str_clone))
        }
        None => config::handle_undefined_variables(&mut session, &template_str_clone),
    }?;

    // Data - now build after handling undefined variables
    let data = session.build_template_data();
    debug!(
        "Template Context: absolute_code_path={}, files_count={}, has_user_vars={}",
        data.absolute_code_path,
        data.files.map(|f| f.len()).unwrap_or(0),
        !session.config.user_variables.is_empty()
    );

    // Render
    let rendered = session.render_prompt(&data).unwrap_or_else(|e| {
        error!("Failed to render prompt: {}", e);
        std::process::exit(1);
    });

    if let Some(ref s) = spinner {
        s.finish_with_message("Codebase Traversal Done!".green().to_string());
    }

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
        use crate::token_map::{display_token_map, generate_token_map_with_limit};

        if let Some(files) = session.data.files.as_ref() {
            // Calculate total tokens from individual file counts
            let total_from_files: usize = files.iter().map(|f| f.token_count).sum();
            // The map should be rooted at the repo, not the filesystem. FileEntry paths
            // can arrive canonicalized/absolute (gnaw . → canonicalize), which otherwise
            // renders as a chain of single-child 100% rows (Users/zero/projects/...).
            // Strip the configured root so the tree starts at the project.
            let root = session
                .config
                .path
                .canonicalize()
                .unwrap_or_else(|_| session.config.path.clone());
            let rel_files: Vec<FileEntry> = files
                .iter()
                .cloned()
                .map(|mut f| {
                    if let Ok(stripped) = std::path::Path::new(&f.path).strip_prefix(&root) {
                        f.path = stripped.to_string_lossy().into_owned();
                    }
                    f
                })
                .collect();
            // Get max lines from command line or calculate from terminal height
            let max_lines = args.token_map_lines.unwrap_or_else(|| {
                terminal_size::terminal_size()
                    .map(|(_, terminal_size::Height(h))| (h as usize).saturating_sub(10).max(20))
                    .unwrap_or(40)
            });

            // Use the sum of individual file tokens for the map with line limit
            let entries = generate_token_map_with_limit(
                &rel_files,
                total_from_files,
                Some(max_lines),
                args.token_map_min_percent,
            );
            display_token_map(&entries, total_from_files);
        }
    }

    // ~~~ Output to Stdout ~~~
    if output_to_stdout {
        print!("{}", rendered.prompt);
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

    // ~~~ Split Output ~~~
    if let Some(budget) = args.split_size {
        let base = args
            .output_file
            .as_deref()
            .filter(|f| *f != "-")
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "--split-size requires --output-file as a base name \
                 (e.g. -O ctx.md); it cannot write to stdout or the clipboard"
                )
            })?;
        return write_split_output(&mut session, base, budget, quiet_mode);
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

/// A group of files destined for one output part, plus its summed token count.
struct FilePart {
    indices: Vec<usize>,
    token_count: usize,
}

/// Greedily packs files (in their existing, sorted order) into parts so that
/// each part's summed token count stays at or below `budget`.
///
/// Order is preserved (determinism). A single file whose own token_count
/// exceeds `budget` is placed alone in its own part; the caller is expected
/// to warn, since we never drop or truncate content.
fn group_files_by_budget(token_counts: &[usize], budget: usize) -> Vec<FilePart> {
    // A zero/again-degenerate budget would loop forever or produce one file
    // per part with no progress guarantee; treat <=0 as "everything in one part".
    if budget == 0 {
        return vec![FilePart {
            indices: (0..token_counts.len()).collect(),
            token_count: token_counts.iter().sum(),
        }];
    }

    let mut parts: Vec<FilePart> = Vec::new();
    let mut current = FilePart {
        indices: Vec::new(),
        token_count: 0,
    };

    for (idx, &tokens) in token_counts.iter().enumerate() {
        let would_overflow = current.token_count + tokens > budget;
        // Flush the current part before starting a new one, but only if it
        // actually holds something — avoids emitting an empty leading part
        // when the very first file already exceeds the budget.
        if would_overflow && !current.indices.is_empty() {
            parts.push(std::mem::replace(
                &mut current,
                FilePart {
                    indices: Vec::new(),
                    token_count: 0,
                },
            ));
        }
        current.indices.push(idx);
        current.token_count += tokens;
    }

    if !current.indices.is_empty() {
        parts.push(current);
    }
    parts
}

/// Renders and writes split output parts. Each part is a complete, independently
/// pasteable document produced through the normal render path.
fn write_split_output(
    session: &mut gnaw_core::session::GnawSession,
    base_path: &str,
    budget: usize,
    quiet: bool,
) -> Result<()> {
    use gnaw_core::path::FileEntry;

    // Take ownership of the file list so we can render subsets without cloning
    // the (potentially large) code bodies repeatedly. Restored at the end.
    let all_files: Vec<FileEntry> = session.data.files.take().unwrap_or_default();

    if all_files.is_empty() {
        anyhow::bail!("Nothing to split: no files were selected");
    }

    let token_counts: Vec<usize> = all_files.iter().map(|f| f.token_count).collect();
    let parts = group_files_by_budget(&token_counts, budget);

    let (stem, ext) = split_base_path(base_path);
    let total_parts = parts.len();

    // Move files out by index so each FileEntry is relocated, not cloned.
    let mut slots: Vec<Option<FileEntry>> = all_files.into_iter().map(Some).collect();

    for (part_no, part) in parts.iter().enumerate() {
        let part_files: Vec<FileEntry> = part
            .indices
            .iter()
            .map(|&i| {
                slots[i]
                    .take()
                    .expect("each index appears in exactly one part")
            })
            .collect();

        if part.token_count > budget && part_files.len() == 1 && !quiet {
            eprintln!(
                "{}{}{} {}",
                "[".bold().white(),
                "!".bold().yellow(),
                "]".bold().white(),
                format!(
                    "File '{}' alone is {} tokens, over the {}-token budget; \
                     it gets its own part.",
                    part_files[0].path, part.token_count, budget
                )
                .yellow()
            );
        }

        // Swap this part's files into the session, render, then move them back out.
        session.data.files = Some(part_files);
        let data = session.build_template_data();
        let rendered = session
            .render_prompt(&data)
            .map_err(|e| anyhow::anyhow!("Failed to render part {}: {}", part_no + 1, e))?;

        // Reclaim the FileEntry vec for the next iteration (no clone).
        if let Some(returned) = session.data.files.take() {
            for (slot, file) in part.indices.iter().zip(returned) {
                slots[*slot] = Some(file);
            }
        }

        let part_path = format!("{}.part{}{}", stem, part_no + 1, ext);
        gnaw_core::template::write_to_file(&part_path, &rendered.prompt)
            .context(format!("Failed to write part file: {}", part_path))?;

        if !quiet {
            eprintln!(
                "{}{}{} {}",
                "[".bold().white(),
                "✓".bold().green(),
                "]".bold().white(),
                format!(
                    "Part {}/{} -> {} ({} tokens)",
                    part_no + 1,
                    total_parts,
                    part_path,
                    rendered.token_count
                )
                .green()
            );
        }
    }

    Ok(())
}

/// Splits "ctx.md" into ("ctx", ".md") and "ctx" into ("ctx", "").
/// Splits on the final dot of the file name only, leaving directory
/// components untouched so "out/ctx.md" -> ("out/ctx", ".md").
fn split_base_path(base: &str) -> (String, String) {
    match std::path::Path::new(base).extension() {
        Some(ext) => {
            let ext = ext.to_string_lossy();
            let stem_len = base.len() - ext.len() - 1; // -1 for the dot
            (base[..stem_len].to_string(), format!(".{}", ext))
        }
        None => (base.to_string(), String::new()),
    }
}

#[cfg(test)]
mod split_tests {
    use super::{group_files_by_budget, split_base_path};

    #[test]
    fn packs_files_under_budget() {
        let parts = group_files_by_budget(&[40, 40, 40], 100);
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].indices, vec![0, 1]); // 80 <= 100
        assert_eq!(parts[1].indices, vec![2]);
    }

    #[test]
    fn never_exceeds_budget_when_files_fit() {
        let counts = [30usize, 30, 30, 30, 30];
        let budget = 60;
        for part in group_files_by_budget(&counts, budget) {
            assert!(part.token_count <= budget, "part exceeded budget");
        }
    }

    #[test]
    fn oversized_single_file_gets_own_part() {
        let parts = group_files_by_budget(&[10, 500, 10], 100);
        // 10 | 500(alone) | 10
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[1].indices, vec![1]);
        assert!(parts[1].token_count > 100);
    }

    #[test]
    fn no_empty_leading_part_when_first_file_oversized() {
        let parts = group_files_by_budget(&[500, 10], 100);
        assert_eq!(parts.len(), 2);
        assert!(!parts[0].indices.is_empty());
    }

    #[test]
    fn every_index_appears_exactly_once() {
        let counts = [10usize, 90, 5, 200, 1];
        let parts = group_files_by_budget(&counts, 100);
        let mut seen: Vec<usize> = parts.iter().flat_map(|p| p.indices.clone()).collect();
        seen.sort_unstable();
        assert_eq!(seen, (0..counts.len()).collect::<Vec<_>>());
    }

    #[test]
    fn zero_budget_collapses_to_single_part() {
        let parts = group_files_by_budget(&[10, 20, 30], 0);
        assert_eq!(parts.len(), 1);
        assert_eq!(parts[0].token_count, 60);
    }

    #[test]
    fn base_path_splitting() {
        assert_eq!(split_base_path("ctx.md"), ("ctx".into(), ".md".into()));
        assert_eq!(
            split_base_path("out/ctx.md"),
            ("out/ctx".into(), ".md".into())
        );
        assert_eq!(split_base_path("ctx"), ("ctx".into(), "".into()));
    }
}
