//! code2prompt is a command-line tool to generate an LLM prompt from a codebase directory.
use anyhow::Result;
use clap::Parser;
use code2prompt::{
    count_tokens, extract_undefined_variables, get_git_diff, handlebars_setup, label,
    render_template, traverse_directory,
};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use inquire::Text;
use log::debug;
use serde_json::json;
use std::io::Write;
use std::path::PathBuf;

/// code2prompt is a command-line tool to generate an LLM prompt from a codebase directory.
///
/// Author: Mufeed VH (@mufeedvh)
#[derive(Parser)]
#[clap(name = "code2prompt", version = "1.0.0", author = "Mufeed VH")]
struct Cli {
    /// Path to the codebase directory
    #[arg()]
    path: PathBuf,

    /// Patterns to include
    #[clap(long)]
    include: Option<String>,

    /// Patterns to exclude
    #[clap(long)]
    exclude: Option<String>,

    /// Include files in case of conflict between include and exclude patterns
    #[clap(long)]
    include_priority: bool,

    /// Display the token count of the generated prompt
    #[clap(long)]
    tokens: bool,

    /// Optional tokenizer to use for token count
    ///
    /// Supported tokenizers: cl100k (default), p50k, p50k_edit, r50k, gpt2
    #[clap(short = 'c', long)]
    encoding: Option<String>,

    /// Optional output file path
    #[clap(short, long)]
    output: Option<String>,

    /// Include git diff
    #[clap(short, long)]
    diff: bool,

    /// Add line numbers to the source code
    #[clap(short, long)]
    line_number: bool,

    /// Use relative paths instead of absolute paths, including the parent directory
    #[clap(long)]
    relative_paths: bool,

    /// Disable copying to clipboard
    #[clap(long)]
    no_clipboard: bool,
}

fn main() -> Result<()> {
    // ~~~ CLI Setup ~~~
    env_logger::init();
    let args = Cli::parse();

    // ~~~ Handlebars Template Setup ~~~
    let default_template = include_str!("default_template.hbs");
    let handlebars = handlebars_setup(default_template).expect("Failed to set up Handlebars");

    // ~~~ Progress Bar Setup ~~~
    let spinner = ProgressBar::new_spinner();
    spinner.enable_steady_tick(std::time::Duration::from_millis(120));
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["▹▹▹▹▹", "▸▹▹▹▹", "▹▸▹▹▹", "▹▹▸▹▹", "▹▹▹▸▹", "▹▹▹▹▸"])
            .template("{spinner:.blue} {msg}")
            .unwrap(),
    );

    spinner.set_message("Traversing directory and building tree...");

    // ~~~ Parse Patterns ~~~
    let include_patterns = parse_patterns(&args.include);
    let exclude_patterns = parse_patterns(&args.exclude);

    // ~~~ Traverse the directory ~~~
    let create_tree = traverse_directory(
        &args.path,
        &include_patterns,
        &exclude_patterns,
        args.include_priority,
        args.line_number,
        args.relative_paths,
    );

    let (tree, files) = create_tree.unwrap_or_else(|e| {
        spinner.finish_with_message(format!("{}", "Failed!".red()));
        eprintln!(
            "{}{}{} {}",
            "[".bold().white(),
            "!".bold().red(),
            "]".bold().white(),
            format!("Failed to build directory tree: {}", e).red()
        );
        std::process::exit(1);
    });

    // ~~~ Git Diff ~~~
    let mut git_diff = String::new();
    if args.diff {
        spinner.set_message("Generating git diff...");
        git_diff = get_git_diff(&args.path).unwrap();
    }

    spinner.finish_with_message(format!("{}", "Done!".green()));

    // ~~~ Prepare JSON Data ~~~
    let mut data = json!({
        "absolute_code_path": if args.relative_paths {
            label(args.path.canonicalize().unwrap())
        } else {
            args.path.canonicalize().unwrap().display().to_string()
        },
        "source_tree": tree,
        "files": files,
        "git_diff": git_diff,
    });

    debug!(
        "JSON Data: {}",
        serde_json::to_string_pretty(&data).unwrap()
    );

    // Handle undefined variables
    let undefined_variables = extract_undefined_variables(default_template);
    let mut user_defined_vars = serde_json::Map::new();

    // Prompt user for undefined variables
    for var in undefined_variables.iter() {
        if !data.as_object().unwrap().contains_key(var) {
            let prompt = format!("Enter value for '{}': ", var);
            let answer = Text::new(&prompt)
                .with_help_message("Fill user defined variable in template")
                .prompt()
                .unwrap_or_else(|_| "".to_string());

            user_defined_vars.insert(var.clone(), serde_json::Value::String(answer));
        }
    }

    // Merge user_defined_vars into the existing `data` JSON object
    if let Some(obj) = data.as_object_mut() {
        for (key, value) in user_defined_vars {
            obj.insert(key, value);
        }
    }

    // ~~~ Render the template ~~~
    let rendered = render_template(&handlebars, "default", &data);

    // ~~~ Display Token Count ~~~
    if args.tokens {
        count_tokens(&rendered, &args.encoding);
    }

    // ~~~ Clipboard ~~~
    if !args.no_clipboard {
        match cli_clipboard::set_contents(rendered.to_string()) {
            Ok(_) => {
                println!(
                    "{}{}{} {}",
                    "[".bold().white(),
                    "✓".bold().green(),
                    "]".bold().white(),
                    "Prompt copied to clipboard!".green()
                );
            }
            Err(e) => {
                eprintln!(
                    "{}{}{} {}: {}",
                    "[".bold().white(),
                    "!".bold().red(),
                    "]".bold().white(),
                    "Failed to copy to clipboard".red(),
                    e
                );
            }
        }
    }

    // ~~~ Output File ~~~
    if let Some(output_path) = args.output {
        let file = std::fs::File::create(&output_path).expect("Failed to create output file");
        let mut writer = std::io::BufWriter::new(file);
        write!(writer, "{}", rendered).expect("Failed to write to output file");

        println!(
            "{}{}{} {}",
            "[".bold().white(),
            "✓".bold().green(),
            "]".bold().white(),
            format!("Prompt written to file: {}", output_path).green()
        );
    }

    Ok(())
}

fn parse_patterns(patterns: &Option<String>) -> Vec<String> {
    match patterns {
        Some(patterns) if !patterns.is_empty() => {
            patterns.split(',').map(|s| s.trim().to_string()).collect()
        }
        _ => vec![],
    }
}
