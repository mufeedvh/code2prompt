//! code2prompt is a command-line tool to generate an LLM prompt from a codebase directory.
//!
//! Authors: Mufeed VH (@mufeedvh), Olivier D'Ancona (@ODAncona)
mod args;
mod clipboard;

use anyhow::{Context, Ok, Result};
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
// use serde_json::json;
use std::{path::PathBuf, str::FromStr};

fn main() -> Result<()> {
    env_logger::init();
    info! {"Args: {:?}", std::env::args().collect::<Vec<_>>()};
    let args = Cli::parse();

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

    // Configure Path
    configuration
        .path(args.path.clone())
        .include_priority(args.include_priority);

    // Configure Selection Patterns
    configuration
        .include_patterns(args.include)
        .exclude_patterns(args.exclude);

    // Configure Output Format
    let output_format = args.output_format.clone();
    configuration
        .line_numbers(args.line_numbers)
        .relative_paths(args.relative_paths)
        .full_directory_tree(args.full_directory_tree)
        .output_format(output_format);

    // Configure Sort Method
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

    // Configure Tokenizer
    let tokenizer_type = args
        .encoding
        .as_deref()
        .unwrap_or("cl100k")
        .parse::<TokenizerType>()
        .unwrap_or_default();

    configuration
        .encoding(tokenizer_type)
        .token_format(args.tokens);

    // Configure Template
    let (template_str, template_name) = parse_template(&args.template).unwrap_or_else(|e| {
        error!("Failed to parse template: {}", e);
        std::process::exit(1);
    });

    configuration
        .template_str(template_str.clone())
        .template_name(template_name);

    // Configure Git
    let diff_branches = parse_branch_argument(&args.git_diff_branch);
    let log_branches = parse_branch_argument(&args.git_log_branch);

    configuration
        .diff_enabled(args.diff)
        .diff_branches(diff_branches)
        .log_branches(log_branches);

    // ~~~ Code2Prompt ~~~
    let mut session = Code2PromptSession::new(configuration.build()?);
    let spinner = setup_spinner("Traversing directory and building tree...");

    // ~~~ Gather Repository Data ~~~
    // Load Codebase
    session.load_codebase().unwrap_or_else(|e| {
        spinner.finish_with_message("Failed!".red().to_string());
        error!("Failed to build directory tree: {}", e);
        std::process::exit(1);
    });
    spinner.finish_with_message("Done!".green().to_string());

    // ~~~ Git Related ~~~
    // Git Diff
    if session.config.diff_enabled {
        spinner.set_message("Generating git diff...");
        session.load_git_diff().unwrap_or_else(|e| {
            spinner.finish_with_message("Failed!".red().to_string());
            error!("Failed to generate git diff: {}", e);
            std::process::exit(1);
        });
    }

    // Load Git diff between branches if provided
    if session.config.diff_branches.is_some() {
        spinner.set_message("Generating git diff between two branches...");
        session
            .load_git_diff_between_branches()
            .unwrap_or_else(|e| {
                spinner.finish_with_message("Failed!".red().to_string());
                error!("Failed to generate git diff: {}", e);
                std::process::exit(1);
            });
    }

    // Load Git log between branches if provided
    if session.config.log_branches.is_some() {
        spinner.set_message("Generating git log between two branches...");
        session.load_git_log_between_branches().unwrap_or_else(|e| {
            spinner.finish_with_message("Failed!".red().to_string());
            error!("Failed to generate git log: {}", e);
            std::process::exit(1);
        });
    }

    spinner.finish_with_message("Done!".green().to_string());

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

    println!(
        "{}{}{} Token count: {}, Model info: {}",
        "[".bold().white(),
        "i".bold().blue(),
        "]".bold().white(),
        formatted_token_count,
        model_info
    );

    // if args.output_format == OutputFormat::Json {
    //     let json_output = json!({
    //         "prompt": rendered,
    //         "directory_name": &label(&args.path),
    //         "token_count": token_count,
    //         "model_info": model_info,
    //         "files": &paths,
    //     });
    //     println!("{}", serde_json::to_string_pretty(&json_output).unwrap());
    //     return Ok(());
    // }

    // ~~~ Copy to Clipboard ~~~
    if !args.no_clipboard {
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
                    println!("{}", &rendered.prompt);
                }
            }
        }
    }

    // ~~~ Output File ~~~
    if let Some(output_path) = &args.output_file {
        write_to_file(output_path, &rendered.prompt)?;
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

fn parse_branch_argument(branch_arg: &Option<Vec<String>>) -> Option<(String, String)> {
    match branch_arg {
        Some(branches) if branches.len() == 2 => Some((branches[0].clone(), branches[1].clone())),
        _ => None,
    }
}

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
