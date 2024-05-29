//! code2prompt is a command-line tool to generate an LLM prompt from a codebase directory.
use anyhow::Result;
use arboard::Clipboard;
use clap::Parser;
use code2prompt::{
    count_tokens, extract_undefined_variables, get_git_diff, handlebars_setup, label,
    render_template, traverse_directory,
};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use inquire::Text;
use log::{debug};
use serde_json::json;
use std::io::Write;
use std::path::PathBuf;

const DEFAULT_TEMPLATE_NAME: &str = "default";
const CUSTOM_TEMPLATE_NAME: &str = "custom";

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
    #[clap(short = 'l', long)]
    line_number: bool,

    /// Use relative paths instead of absolute paths, including the parent directory
    #[clap(long)]
    relative_paths: bool,

    /// Optional Disable copying to clipboard
    #[clap(long)]
    no_clipboard: bool,

    /// Optional Path to a custom Handlebars template
    #[clap(short, long)]
    template: Option<PathBuf>,
}
fn main() -> Result<()> {
    env_logger::init();
    let args = Cli::parse();

    // Handlebars Template Setup
    let (template_content, template_name) = get_template(&args)?;
    let handlebars = handlebars_setup(&template_content, template_name)?;

    // Progress Bar Setup
    let spinner = setup_spinner("Traversing directory and building tree...");

    // Parse Patterns
    let include_patterns = parse_patterns(&args.include);
    let exclude_patterns = parse_patterns(&args.exclude);

    // Traverse the directory
    let create_tree = traverse_directory(
        &args.path,
        &include_patterns,
        &exclude_patterns,
        args.include_priority,
        args.line_number,
        args.relative_paths,
    );

    let (tree, files) = create_tree.unwrap_or_else(|e| {
        spinner.finish_with_message("Failed!".red().to_string());
        eprintln!(
            "{}{}{} {}",
            "[".bold().white(),
            "!".bold().red(),
            "]".bold().white(),
            format!("Failed to build directory tree: {}", e).red()
        );
        std::process::exit(1);
    });

    // Git Diff
    let git_diff = if args.diff {
        spinner.set_message("Generating git diff...");
        get_git_diff(&args.path).unwrap_or_default()
    } else {
        String::new()
    };

    spinner.finish_with_message("Done!".green().to_string());

    // Prepare JSON Data
    let mut data = json!({
        "absolute_code_path": label(&args.path),
        "source_tree": tree,
        "files": files,
        "git_diff": git_diff,
    });

    debug!(
        "JSON Data: {}",
        serde_json::to_string_pretty(&data).unwrap()
    );

    // Handle undefined variables
    handle_undefined_variables(&mut data, &template_content)?;

    // Render the template
    let rendered = render_template(&handlebars, template_name, &data)?;

    // Display Token Count
    if args.tokens {
        count_tokens(&rendered, &args.encoding);
    }

    // Copy to Clipboard
    if !args.no_clipboard {
        copy_to_clipboard(&rendered)?;
    }

    // Output File
    if let Some(output_path) = &args.output {
        write_to_file(output_path, &rendered)?;
    }

    Ok(())
}

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

fn get_template(args: &Cli) -> Result<(String, &str)> {
    if let Some(template_path) = &args.template {
        let content = std::fs::read_to_string(template_path)
            .map_err(|e| anyhow::anyhow!("Failed to read custom template file: {}", e))?;
        Ok((content, CUSTOM_TEMPLATE_NAME))
    } else {
        Ok((include_str!("default_template.hbs").to_string(), DEFAULT_TEMPLATE_NAME))
    }
}

fn handle_undefined_variables(data: &mut serde_json::Value, template_content: &str) -> Result<()> {
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

fn copy_to_clipboard(rendered: &str) -> Result<()> {
    let mut clipboard = Clipboard::new().expect("Failed to initialize clipboard");
    clipboard.set_text(rendered.to_string()).map_err(|e| {
        anyhow::anyhow!(
            "Failed to copy to clipboard: {}",
            e
        )
    })?;
    println!(
        "{}{}{} {}",
        "[".bold().white(),
        "✓".bold().green(),
        "]".bold().white(),
        "Prompt copied to clipboard!".green()
    );
    Ok(())
}

fn write_to_file(output_path: &str, rendered: &str) -> Result<()> {
    let file = std::fs::File::create(output_path)?;
    let mut writer = std::io::BufWriter::new(file);
    write!(writer, "{}", rendered)?;
    println!(
        "{}{}{} {}",
        "[".bold().white(),
        "✓".bold().green(),
        "]".bold().white(),
        format!("Prompt written to file: {}", output_path).green()
    );
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
