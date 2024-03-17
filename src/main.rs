use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::Parser;
use colored::*;
use handlebars::{no_escape, Handlebars};
use ignore::WalkBuilder;
use indicatif::{ProgressBar, ProgressStyle};
use inquire::Text;
use regex::Regex;
use serde_json::json;
use termtree::Tree;
use tiktoken_rs::{cl100k_base, p50k_base, p50k_edit, r50k_base};

/// code2prompt is a command-line tool to generate an LLM prompt from a codebase directory.
///
/// Author: Mufeed VH (@mufeedvh)
#[derive(Parser)]
#[clap(name = "code2prompt", version = "1.0.0", author = "Mufeed VH")]
struct Cli {
    /// Path to the codebase directory
    #[arg()]
    path: PathBuf,

    /// Optional custom Handlebars template file path
    #[clap(short, long)]
    template: Option<PathBuf>,

    /// Optional comma-separated list of file extensions to filter
    #[clap(short, long)]
    filter: Option<String>,

    /// Optional comma-separated list of file extensions to exclude
    #[clap(short, long)]
    exclude: Option<String>,

    /// Display the token count of the generated prompt
    #[clap(long)]
    tokens: bool,

    /// Optional tokenizer to use for token count
    ///
    /// Supported tokenizers: cl100k (default), p50k, p50k_edit, r50k, gpt2
    #[clap(short, long)]
    encoding: Option<String>,

    /// Optional output file path
    #[clap(short, long)]
    output: Option<String>,
}

fn main() {
    let args = Cli::parse();

    let default_template = include_str!("default_template.hbs");

    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(no_escape);
    handlebars
        .register_template_string("default", default_template)
        .expect("Failed to register default template");

    let mut template = String::new();

    if let Some(template_path) = args.template.as_ref() {
        template = std::fs::read_to_string(template_path).expect("Failed to read template file");
        handlebars
            .register_template_string("custom", &template)
            .expect("Failed to register custom template");
    }

    let spinner = ProgressBar::new_spinner();
    spinner.enable_steady_tick(std::time::Duration::from_millis(120));
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["▹▹▹▹▹", "▸▹▹▹▹", "▹▸▹▹▹", "▹▹▸▹▹", "▹▹▹▸▹", "▹▹▹▹▸"])
            .template("{spinner:.blue} {msg}")
            .unwrap(),
    );

    spinner.set_message("Traversing directory and building tree...");

    let create_tree = traverse_directory(&args.path, &args.filter, &args.exclude);
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

    spinner.finish_with_message(format!("{}", "Done!".green()));

    let mut data = json!({
        "absolute_code_path": args.path.canonicalize().unwrap().display().to_string(),
        "source_tree": tree,
        "files": files,
    });

    let undefined_variables = extract_undefined_variables(&template);
    let mut user_defined_vars = serde_json::Map::new();

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

    let rendered = if args.template.is_some() {
        handlebars
            .render("custom", &data)
            .expect("Failed to render custom template")
    } else {
        handlebars
            .render("default", &data)
            .expect("Failed to render default template")
    };
    let rendered = rendered.trim();

    if args.tokens {
        let (bpe, model_info) = match args.encoding.as_deref().unwrap_or("cl100k") {
            "cl100k" => (cl100k_base(), "ChatGPT models, text-embedding-ada-002"),
            "p50k" => (
                p50k_base(),
                "Code models, text-davinci-002, text-davinci-003",
            ),
            "p50k_edit" => (
                p50k_edit(),
                "Edit models like text-davinci-edit-001, code-davinci-edit-001",
            ),
            "r50k" | "gpt2" => (r50k_base(), "GPT-3 models like davinci"),
            _ => (cl100k_base(), "ChatGPT models, text-embedding-ada-002"),
        };

        let token_count = bpe.unwrap().encode_with_special_tokens(&rendered).len();

        println!(
            "{}{}{} Token count: {}, Model info: {}",
            "[".bold().white(),
            "i".bold().blue(),
            "]".bold().white(),
            token_count.to_string().bold().yellow(),
            model_info
        );
    }

    cli_clipboard::set_contents(rendered.into()).expect("Failed to copy output to clipboard");

    println!(
        "{}{}{} {}",
        "[".bold().white(),
        "✓".bold().green(),
        "]".bold().white(),
        "Prompt copied to clipboard!".green()
    );

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
}

/// Extracts undefined variable names from the template string.
#[inline]
fn extract_undefined_variables(template: &str) -> Vec<String> {
    let registered_identifiers = vec!["path", "code"];
    let re = Regex::new(r"\{\{\s*(?P<var>[a-zA-Z_][a-zA-Z_0-9]*)\s*\}\}").unwrap();
    re.captures_iter(template)
        .map(|cap| cap["var"].to_string())
        .filter(|var| !registered_identifiers.contains(&var.as_str()))
        .collect()
}

/// Wraps the code block with backticks and adds the file extension as a label
#[inline]
fn wrap_code_block(code: &str, extension: &str) -> String {
    let backticks = "`".repeat(7);
    format!("{}{}\n{}\n{}", backticks, extension, code, backticks)
}

/// Returns the directory name as a label
#[inline]
fn label<P: AsRef<Path>>(p: P) -> String {
    let path = p.as_ref();
    if path.file_name().is_none() {
        // If the path is the current directory or a root directory
        path.to_str().unwrap_or(".").to_owned()
    } else {
        // Otherwise, use the file name as the label
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_owned()
    }
}

/// Traverses the directory, builds the tree, and collects information about each file.
fn traverse_directory(
    root_path: &PathBuf,
    filter: &Option<String>,
    exclude: &Option<String>,
) -> Result<(String, Vec<serde_json::Value>)> {
    let mut files = Vec::new();

    let canonical_root_path = root_path.canonicalize()?;

    let tree = WalkBuilder::new(&canonical_root_path)
        .git_ignore(true)
        .build()
        .filter_map(|e| e.ok())
        .fold(Tree::new(label(&canonical_root_path)), |mut root, entry| {
            let path = entry.path();
            // Calculate the relative path from the root directory to this entry
            if let Ok(relative_path) = path.strip_prefix(&canonical_root_path) {
                let mut current_tree = &mut root;
                for component in relative_path.components() {
                    let component_str = component.as_os_str().to_string_lossy().to_string();

                    current_tree = if let Some(pos) = current_tree
                        .leaves
                        .iter_mut()
                        .position(|child| child.root == component_str)
                    {
                        &mut current_tree.leaves[pos]
                    } else {
                        let new_tree = Tree::new(component_str.clone());
                        current_tree.leaves.push(new_tree);
                        current_tree.leaves.last_mut().unwrap()
                    };
                }

                if path.is_file() {
                    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

                    if let Some(ref exclude_ext) = exclude {
                        let exclude_extensions: Vec<&str> =
                            exclude_ext.split(',').map(|s| s.trim()).collect();
                        if exclude_extensions.contains(&extension) {
                            return root;
                        }
                    }

                    if let Some(ref filter_ext) = filter {
                        let filter_extensions: Vec<&str> =
                            filter_ext.split(',').map(|s| s.trim()).collect();
                        if !filter_extensions.contains(&extension) {
                            return root;
                        }
                    }

                    let code_bytes = fs::read(&path).expect("Failed to read file");
                    let code = String::from_utf8_lossy(&code_bytes);

                    let code_block = wrap_code_block(&code, extension);

                    if !code.trim().is_empty() && !code.contains(char::REPLACEMENT_CHARACTER) {
                        files.push(json!({
                            "path": path.display().to_string(),
                            "extension": extension,
                            "code": code_block,
                        }));
                    }
                }
            }

            root
        });

    Ok((tree.to_string(), files))
}
