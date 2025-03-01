use clap::Parser;
use code2prompt::engine::{template::OutputFormat, tokenizer::TokenFormat};
use std::path::PathBuf;

// ~~~ CLI Arguments ~~~
#[derive(Parser)]
#[clap(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS")
)]
#[command(arg_required_else_help = true)]
pub struct Cli {
    /// Path to the codebase directory
    #[arg()]
    pub path: PathBuf,

    /// Patterns to include
    #[clap(short = 'i', long)]
    pub include: Option<String>,

    /// Patterns to exclude
    #[clap(short = 'e', long)]
    pub exclude: Option<String>,

    /// Include files in case of conflict between include and exclude patterns
    #[clap(long)]
    pub include_priority: bool,

    /// Optional output file path
    #[clap(short = 'O', long = "output-file")]
    pub output_file: Option<String>,

    /// Output format: markdown, json, or xml
    #[clap(short = 'F', long = "output-format", default_value = "markdown")]
    pub output_format: OutputFormat,

    /// Optional Path to a custom Handlebars template
    #[clap(short, long)]
    pub template: Option<PathBuf>,

    /// List the full directory tree
    #[clap(long)]
    pub full_directory_tree: bool,

    /// Optional tokenizer to use for token count
    ///
    /// Supported tokenizers: cl100k (default), p50k, p50k_edit, r50k, gpt2
    #[clap(short = 'c', long)]
    pub encoding: Option<String>,

    /// Display the token count of the generated prompt.
    /// Accepts a format: "raw" (machine parsable) or "format" (human readable).
    #[clap(long, value_name = "FORMAT", default_value = "format")]
    pub tokens: TokenFormat,

    /// Include git diff
    #[clap(short, long)]
    pub diff: bool,

    /// Generate git diff between two branches
    #[clap(long, value_name = "BRANCHES")]
    pub git_diff_branch: Option<String>,

    /// Retrieve git log between two branches
    #[clap(long, value_name = "BRANCHES")]
    pub git_log_branch: Option<String>,

    /// Add line numbers to the source code
    #[clap(short, long)]
    pub line_number: bool,

    /// Use relative paths instead of absolute paths, including the parent directory
    #[clap(long)]
    pub relative_paths: bool,

    /// Follow symlinks
    #[clap(short = 'L', long)]
    pub follow_symlinks: bool,

    /// Include hidden directories and files
    #[clap(long)]
    pub hidden: bool,

    /// Disable wrapping code inside markdown code blocks
    #[clap(long)]
    pub no_codeblock: bool,

    /// Optional Disable copying to clipboard
    #[clap(long)]
    pub no_clipboard: bool,

    /// Skip .gitignore rules
    #[clap(long)]
    pub no_ignore: bool,

    /// Sort order for files: one of "name_asc", "name_desc", "date_asc", or "date_desc"
    #[clap(long)]
    pub sort: Option<String>,

    #[arg(long, hide = true)]
    pub clipboard_daemon: bool,
}
