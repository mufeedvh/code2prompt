//! code2prompt is a command-line tool to generate an LLM prompt from a codebase directory.
//!
//! Authors: Mufeed VH (@mufeedvh), Olivier D'Ancona (@ODAncona)

use anyhow::{Context, Result};
use clap::Parser;
use code2prompt-cli::args::Cli;

use code2prompt::engine::{
    git::{get_git_diff, get_git_diff_between_branches, get_git_log},
    path::{label, traverse_directory},
    sort::FileSortMethod,
    template::{
        handle_undefined_variables, handlebars_setup, render_template, write_to_file, OutputFormat,
    },
    tokenizer::{count_tokens, TokenFormat, TokenizerType},
};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, error, info};
use num_format::{SystemLocale, ToFormattedString};
use serde_json::json;

fn main() -> Result<()> {
    env_logger::init();
    info! {"Args: {:?}", std::env::args().collect::<Vec<_>>()};
    let args = Cli::parse();

    // ~~~ Clipboard Daemon ~~~
    #[cfg(target_os = "linux")]
    {
        use code2prompt-cli::clipboard::serve_clipboard_daemon;
        if args.clipboard_daemon {
            info! {"Serving clipboard daemon..."};
            serve_clipboard_daemon()?;
            info! {"Shutting down gracefully..."};
            return Ok(());
        }
    }

    // ~~~ Initialization ~~~

    // Progress Bar Setup
    let spinner = setup_spinner("Traversing directory and building tree...");

    // Parse Patterns
    let include_patterns = parse_patterns(&args.include);
    let exclude_patterns = parse_patterns(&args.exclude);

    // Sort Method
    let sort_method: Option<FileSortMethod> = match args.sort.as_deref() {
        Some("name_asc") => Some(FileSortMethod::NameAsc),
        Some("name_desc") => Some(FileSortMethod::NameDesc),
        Some("date_asc") => Some(FileSortMethod::DateAsc),
        Some("date_desc") => Some(FileSortMethod::DateDesc),
        Some(other) => {
            eprintln!("Invalid sort method: {}. Supported values: name_asc, name_desc, date_asc, date_desc", other);
            std::process::exit(1);
        }
        None => None,
    };

    // ~~~ Traverse the directory ~~~
    let create_tree = traverse_directory(
        &args.path,
        &include_patterns,
        &exclude_patterns,
        args.include_priority,
        args.line_number,
        args.relative_paths,
        args.full_directory_tree,
        args.no_codeblock,
        args.follow_symlinks,
        args.hidden,
        args.no_ignore,
        sort_method,
    );

    let (tree, files) = match create_tree {
        Ok(result) => result,
        Err(e) => {
            spinner.finish_with_message("Failed!".red().to_string());
            eprintln!(
                "{}{}{} {}",
                "[".bold().white(),
                "!".bold().red(),
                "]".bold().white(),
                format!("Failed to build directory tree: {}", e).red()
            );
            std::process::exit(1);
        }
    };

    // ~~~ Git Related ~~~
    // Git Diff
    let git_diff = if args.diff {
        spinner.set_message("Generating git diff...");
        get_git_diff(&args.path).unwrap_or_default()
    } else {
        String::from("no git diff")
    };

    // Git diff between two branches
    let mut git_diff_branch: String = String::new();
    if let Some(branches) = &args.git_diff_branch {
        spinner.set_message("Generating git diff between two branches...");
        let branches = parse_patterns(&Some(branches.to_string()));
        if branches.len() != 2 {
            error!("Please provide exactly two branches separated by a comma.");
            std::process::exit(1);
        }
        git_diff_branch = get_git_diff_between_branches(&args.path, &branches[0], &branches[1])
            .unwrap_or_default()
    }

    // Git log between two branches
    let mut git_log_branch: String = String::new();
    if let Some(branches) = &args.git_log_branch {
        spinner.set_message("Generating git log between two branches...");
        let branches = parse_patterns(&Some(branches.to_string()));
        if branches.len() != 2 {
            error!("Please provide exactly two branches separated by a comma.");
            std::process::exit(1);
        }
        git_log_branch = get_git_log(&args.path, &branches[0], &branches[1]).unwrap_or_default()
    }

    spinner.finish_with_message("Done!".green().to_string());

    // ~~~ Template ~~~
    // Template Data
    let mut data = json!({
        "absolute_code_path": label(&args.path),
        "source_tree": tree,
        "files": files,
        "git_diff": git_diff,
        "git_diff_branch": git_diff_branch,
        "git_log_branch": git_log_branch
    });

    debug!(
        "JSON Data: {}",
        serde_json::to_string_pretty(&data).unwrap()
    );

    // Template Setup
    let (template_content, template_name) = get_template(&args)?;
    let handlebars = handlebars_setup(&template_content, &template_name)?;

    // Handle User Defined Variables
    handle_undefined_variables(&mut data, &template_content)?;

    // Template Rendering
    let rendered = render_template(&handlebars, &template_name, &data)?;

    // ~~~ Token Count ~~~
    let tokenizer_type = args
        .encoding
        .as_deref()
        .unwrap_or("cl100k")
        .parse::<TokenizerType>()
        .unwrap_or(TokenizerType::Cl100kBase);

    let token_count = count_tokens(&rendered, &tokenizer_type);
    let formatted_token_count: String = match args.tokens {
        TokenFormat::Raw => token_count.to_string(),
        TokenFormat::Format => token_count.to_formatted_string(&SystemLocale::default().unwrap()),
    };

    let model_info = tokenizer_type.description();

    println!(
        "{}{}{} Token count: {}, Model info: {}",
        "[".bold().white(),
        "i".bold().blue(),
        "]".bold().white(),
        formatted_token_count,
        model_info
    );

    // ~~~ Informations ~~~
    let paths: Vec<String> = files
        .iter()
        .filter_map(|file| {
            file.get("path")
                .and_then(|p| p.as_str())
                .map(|s| s.to_string())
        })
        .collect();

    if args.output_format == OutputFormat::Json {
        let json_output = json!({
            "prompt": rendered,
            "directory_name": &label(&args.path),
            "token_count": token_count,
            "model_info": model_info,
            "files": &paths,
        });
        println!("{}", serde_json::to_string_pretty(&json_output).unwrap());
        return Ok(());
    }

    // ~~~ Copy to Clipboard ~~~
    if !args.no_clipboard {
        #[cfg(target_os = "linux")]
        {
            use code2prompt-cli::clipboard::spawn_clipboard_daemon;
            spawn_clipboard_daemon(&rendered)?;
        }
        #[cfg(not(target_os = "linux"))]
        {
            use code2prompt::copy_text_to_clipboard;
            match copy_text_to_clipboard(&rendered) {
                Ok(_) => {
                    println!(
                        "{}{}{} {}",
                        "[".bold().white(),
                        "✓".bold().green(),
                        "]".bold().white(),
                        "Copied to clipboard successfully.".green()
                    );
                }
                Err(e) => {
                    eprintln!(
                        "{}{}{} {}",
                        "[".bold().white(),
                        "!".bold().red(),
                        "]".bold().white(),
                        format!("Failed to copy to clipboard: {}", e).red()
                    );
                    println!("{}", &rendered);
                }
            }
        }
    }

    // ~~~ Output File ~~~
    if let Some(output_path) = &args.output_file {
        write_to_file(output_path, &rendered)?;
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

/// Parses comma-separated patterns into a vector of strings
///
/// # Arguments
///
/// * `patterns` - An optional string containing comma-separated patterns
///
/// # Returns
///
/// * `Vec<String>` - A vector of parsed patterns
fn parse_patterns(patterns: &Option<String>) -> Vec<String> {
    match patterns {
        Some(patterns) if !patterns.is_empty() => {
            patterns.split(',').map(|s| s.trim().to_string()).collect()
        }
        _ => vec![],
    }
}

/// Retrieves the template content and name based on the CLI arguments
///
/// # Arguments
///
/// * `args` - The parsed CLI arguments
///
/// # Returns
///
/// * `Result<(String, &str)>` - A tuple containing the template content and name
fn get_template(args: &Cli) -> Result<(String, String)> {
    let format = &args.output_format;
    if let Some(template_path) = &args.template {
        let content = std::fs::read_to_string(template_path)
            .context("Failed to read custom template file")?;
        Ok((content, "custom".to_string()))
    } else {
        match format {
            OutputFormat::Markdown | OutputFormat::Json => Ok((
                include_str!("default_template_md.hbs").to_string(),
                "markdown".to_string(),
            )),
            OutputFormat::Xml => Ok((
                include_str!("default_template_xml.hbs").to_string(),
                "xml".to_string(),
            )),
        }
    }
}
