//! Command-line argument parsing and validation.
//!
//! This module defines the CLI structure using clap for parsing command-line arguments
//! and options for the code2prompt tool. It supports both TUI and CLI modes with
//! comprehensive configuration options for file selection, output formatting,
//! tokenization, and git integration.
use anyhow::{Result, anyhow};
use clap::{Parser, builder::ValueParser};
use code2prompt_core::{
    sort::FileSortMethod, template::OutputFormat, tokenizer::TokenFormat, tokenizer::TokenizerType,
};
use serde::de::DeserializeOwned;
use std::path::PathBuf;

// ~~~ CLI Arguments ~~~
#[derive(Parser, Debug)]
#[clap(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS")
)]
#[command(arg_required_else_help = true)]
pub struct Cli {
    /// Path to the codebase directory
    #[arg(value_name = "PATH_TO_ANALYZE", default_value = ".")]
    pub path: PathBuf,

    /// Optional output file (use "-" for stdout)
    #[arg(short = 'O', long = "output-file", value_name = "FILE")]
    pub output_file: Option<String>,

    /// Launch the Terminal User Interface
    #[clap(long)]
    pub tui: bool,

    /// Patterns to include
    #[clap(short = 'i', long = "include")]
    pub include: Vec<String>,

    /// Patterns to exclude
    #[clap(short = 'e', long = "exclude")]
    pub exclude: Vec<String>,

    /// Output format
    #[clap(
        short = 'F',
        long = "output-format",
        default_value = "markdown",
        value_name = "markdown, json, xml"
    )]
    pub output_format: OutputFormat,

    /// Optional Path to a custom Handlebars template
    #[clap(short, long, value_name = "TEMPLATE")]
    pub template: Option<PathBuf>,

    /// List the full directory tree
    #[clap(long)]
    pub full_directory_tree: bool,

    /// Token encoding to use for token count
    #[clap(
        long,
        value_name = "cl100k, p50k, p50k_edit, r50k",
        value_parser = ValueParser::new(parse_serde::<TokenizerType>),
    )]
    pub encoding: Option<TokenizerType>,

    /// Display the token count of the generated prompt. Accepts a format: "raw" (machine parsable) or "format" (human readable)
    #[clap(
        long,
        value_name = "raw,format",
        value_parser = ValueParser::new(parse_serde::<TokenFormat>),
    )]
    pub token_format: Option<TokenFormat>,

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

    /// Copy output to clipboard
    #[clap(short = 'c', long)]
    pub clipboard: bool,

    /// Optional Disable copying to clipboard (deprecated, use default behavior)
    #[clap(long, hide = true)]
    pub no_clipboard: bool,

    /// Skip .gitignore rules
    #[clap(long)]
    pub no_ignore: bool,

    /// Sort order for files
    #[clap(
        long,
        value_name = "name_asc, name_desc, date_asc, date_desc",
        value_parser = ValueParser::new(parse_serde::<FileSortMethod>),
    )]
    pub sort: Option<FileSortMethod>,

    /// Suppress progress and success messages
    #[clap(short = 'q', long)]
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

/// Helper function to parse serde deserializable enum from string inputs.
fn parse_serde<T: DeserializeOwned>(s: &str) -> Result<T> {
    serde_json::from_value(serde_json::Value::String(s.to_string()))
        .map_err(|e| anyhow!("Failed to parse value: {}", e))
}
