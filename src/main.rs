//! code2prompt is a command-line tool to generate an LLM prompt from a codebase directory.
//!
//! Authors: Mufeed VH (@mufeedvh), Olivier D'Ancona (@ODAncona)

use anyhow::{Context, Result};
use clap::Parser;
use code2prompt::{
    count_tokens, get_git_diff, get_git_diff_between_branches, get_git_log,
    handle_undefined_variables, handlebars_setup, label, render_template, traverse_directory,
    write_to_file, FileSortMethod, OutputFormat, TokenFormat, TokenizerType,
};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, error, info};
use num_format::{SystemLocale, ToFormattedString};
use serde_json::json;
use std::path::PathBuf;

// ~~~ CLI Arguments ~~~
#[derive(Parser)]
#[clap(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS")
)]
#[command(arg_required_else_help = true)]
struct Cli {
    /// Path to the codebase directory
    #[arg()]
    path: PathBuf,

    /// Patterns to include
    #[clap(short = 'i', long)]
    include: Option<String>,

    /// Patterns to exclude
    #[clap(short = 'e', long)]
    exclude: Option<String>,

    /// Include files in case of conflict between include and exclude patterns
    #[clap(long)]
    include_priority: bool,

    /// Optional output file path
    #[clap(short = 'O', long = "output-file")]
    output_file: Option<String>,

    /// Output format: markdown, json, or xml
    #[clap(short = 'F', long = "output-format", default_value = "markdown")]
    output_format: OutputFormat,

    /// Optional Path to a custom Handlebars template
    #[clap(short, long)]
    template: Option<PathBuf>,

    /// Exclude files/folders from the source tree based on exclude patterns
    #[clap(long)]
    exclude_from_tree: bool,

    /// Optional tokenizer to use for token count
    ///
    /// Supported tokenizers: cl100k (default), p50k, p50k_edit, r50k, gpt2
    #[clap(short = 'c', long)]
    encoding: Option<String>,

    /// Display the token count of the generated prompt.
    /// Accepts a format: "raw" (machine parsable) or "format" (human readable).
    #[clap(long, value_name = "FORMAT", default_value = "format")]
    tokens: TokenFormat,

    /// Include git diff
    #[clap(short, long)]
    diff: bool,

    /// Generate git diff between two branches
    #[clap(long, value_name = "BRANCHES")]
    git_diff_branch: Option<String>,

    /// Retrieve git log between two branches
    #[clap(long, value_name = "BRANCHES")]
    git_log_branch: Option<String>,

    /// Add line numbers to the source code
    #[clap(short, long)]
    line_number: bool,

    /// Use relative paths instead of absolute paths, including the parent directory
    #[clap(long)]
    relative_paths: bool,

    /// Follow symlinks
    #[clap(short = 'L', long)]
    follow_symlinks: bool,

    /// Include hidden directories and files
    #[clap(long)]
    hidden: bool,

    /// Disable wrapping code inside markdown code blocks
    #[clap(long)]
    no_codeblock: bool,

    /// Optional Disable copying to clipboard
    #[clap(long)]
    no_clipboard: bool,

    /// Skip .gitignore rules
    #[clap(long)]
    no_ignore: bool,

    /// Sort order for files: one of "name_asc", "name_desc", "date_asc", or "date_desc"
    #[clap(long)]
    sort: Option<String>,

    #[arg(long, hide = true)]
    clipboard_daemon: bool,
}

fn main() -> Result<()> {
    env_logger::init();
    info! {"Args: {:?}", std::env::args().collect::<Vec<_>>()};
    let args = Cli::parse();

    // ~~~ Clipboard Daemon ~~~
    #[cfg(target_os = "linux")]
    {
        use code2prompt::serve_clipboard_daemon;
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
        args.exclude_from_tree,
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
        String::new()
    };

    // git diff between two branches
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

    // git log between two branches
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
            use code2prompt::spawn_clipboard_daemon;
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
