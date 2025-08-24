use clap::Parser;
use code2prompt_core::{template::OutputFormat, tokenizer::TokenFormat};
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
    #[clap(short = 'i', long = "include", value_delimiter = ',')]
    pub include: Vec<String>,

    /// Patterns to exclude
    #[clap(short = 'e', long = "exclude", value_delimiter = ',')]
    pub exclude: Vec<String>,

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
    #[clap(long, value_name = "BRANCHES", num_args = 2, value_delimiter = ',')]
    pub git_diff_branch: Option<Vec<String>>,

    /// Retrieve git log between two branches
    #[clap(long, value_name = "BRANCHES", num_args = 2, value_delimiter = ',')]
    pub git_log_branch: Option<Vec<String>>,

    /// Add line numbers to the source code
    #[clap(short, long)]
    pub line_numbers: bool,

    /// If true, paths in the output will be absolute instead of relative.
    #[clap(long)]
    pub absolute_paths: bool,

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

    /// Extra ignore files to use (e.g., .dockerignore, .npmignore)
    #[clap(short = 'E', long = "extra-ignore-files", value_delimiter = ',')]
    pub extra_ignore_files: Vec<String>,

    /// Skip .promptignore rules (both local and global)
    #[clap(long)]
    pub no_promptignore: bool,

    /// Sort order for files: one of "name_asc", "name_desc", "date_asc", or "date_desc"
    #[clap(long)]
    pub sort: Option<String>,

    /// Suppress progress and success messages
    #[clap(short, long)]
    pub quiet: bool,

    /// Display a visual token map of files (similar to disk usage tools)
    #[clap(long)]
    pub token_map: bool,

    /// Maximum number of lines to display in token map (default: terminal height - 10)
    #[clap(long, value_name = "NUMBER")]
    pub token_map_lines: Option<usize>,

    /// Minimum percentage of tokens to display in token map (default: 0.1%)
    #[clap(long, value_name = "PERCENT")]
    pub token_map_min_percent: Option<f64>,

    #[arg(long, hide = true)]
    pub clipboard_daemon: bool,
}
