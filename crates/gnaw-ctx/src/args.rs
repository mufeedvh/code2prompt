//! Command-line argument parsing and validation.
//!
//! This module defines the CLI structure using clap for parsing command-line arguments
//! and options for the gnaw tool. It supports both TUI and CLI modes with
//! comprehensive configuration options for file selection, output formatting,
//! tokenization, and git integration.
use clap::{Parser, ValueHint};
use clap_complete::engine::{ArgValueCandidates, CompletionCandidate};
use gnaw_core::secret_scan::SecretPolicy;
use gnaw_core::{
    configuration::{DiffMode, DiffShaContent},
    sort::FileSortMethod,
    template::OutputFormat,
    tokenizer::TokenFormat,
    tokenizer::TokenizerType,
};
use std::path::PathBuf;

// ~~~ CLI Arguments ~~~
#[derive(Parser, Debug)]
#[clap(
    name = "gnaw",
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS")
)]
#[command(arg_required_else_help = true)]
pub struct Cli {
    /// Path to the codebase directory
    #[arg(value_name = "PATH_TO_ANALYZE", default_value = ".", value_hint = ValueHint::AnyPath)]
    pub path: PathBuf,

    /// Optional output file (use "-" for stdout)
    #[arg(short = 'O', long = "output-file", value_name = "FILE", value_hint = ValueHint::FilePath)]
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
    #[clap(short = 'F', long = "output-format", value_enum)]
    pub output_format: Option<OutputFormat>,

    /// Optional Path to a custom Handlebars template
    #[clap(short, long, value_name = "NAME_OR_PATH", value_hint = ValueHint::FilePath,
       add = ArgValueCandidates::new(template_candidates))]
    pub template: Option<String>,

    /// List the full directory tree
    #[clap(long)]
    pub full_directory_tree: bool,

    /// Token encoding to use for token count
    #[clap(long, value_enum)]
    pub encoding: Option<TokenizerType>,

    /// Display the token count of the generated prompt. Accepts a format: "raw" (machine parsable) or "format" (human readable)
    #[clap(long, value_enum)]
    pub token_format: Option<TokenFormat>,

    /// Include git diff
    #[clap(short, long)]
    pub diff: bool,

    /// Which changes to diff: staged (default), unstaged, or all uncommitted
    #[clap(long, value_enum, requires = "diff")]
    pub diff_mode: Option<DiffMode>,

    /// Generate git diff between two branches
    #[clap(long, value_name = "BRANCHES", num_args = 2, value_delimiter = ',')]
    pub git_diff_branch: Option<Vec<String>>,

    /// Retrieve git log between two branches
    #[clap(long, value_name = "BRANCHES", num_args = 2, value_delimiter = ',')]
    pub git_log_branch: Option<Vec<String>>,

    /// Files changed between two refs. Accepts `ref1..ref2`, `ref1,ref2`, or `ref1 ref2`
    #[clap(long, value_name = "REF1..REF2", num_args = 1..=2)]
    pub git_diff_shas: Option<Vec<String>>,

    /// What to emit per changed file: patch (lean, default), full, or full-patch
    #[clap(
        long,
        value_enum,
        default_value = "after-patch",
        requires = "git_diff_shas"
    )]
    pub git_diff_shas_content: DiffShaContent,

    /// With --git-diff-shas, skip files larger than this many bytes (0 = no limit)
    #[clap(
        long,
        value_name = "BYTES",
        default_value_t = 0,
        requires = "git_diff_shas"
    )]
    pub git_diff_shas_max_bytes: usize,

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
    #[clap(long, value_enum)]
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

    /// Start with all files deselected
    #[clap(long)]
    pub deselected: bool,

    #[arg(long, hide = true)]
    pub clipboard_daemon: bool,

    /// Split output into multiple files, each capped at roughly this many tokens.
    /// Requires --output-file as a base name (e.g. ctx.md -> ctx.part1.md, ctx.part2.md).
    /// Mutually exclusive with --clipboard.
    #[clap(long, value_name = "TOKENS", conflicts_with = "clipboard")]
    pub split_size: Option<usize>,

    /// Compression preset: light (tests), moderate (+ fn bodies), full (maximal)
    #[clap(long, value_enum)]
    pub compress: Option<gnaw_core::configuration::CompressionLevel>,

    /// Manual compression toggles (CSV), applied over --compress. Tokens:
    /// tests, fn-bodies, doc-comments, private-bodies; prefix `no-` to disable.
    #[clap(long, value_name = "TOKENS", value_delimiter = ',',
       add = ArgValueCandidates::new(compress_strip_candidates))]
    pub compress_strip: Option<Vec<String>>,

    #[clap(long, value_enum, default_value = "warn")]
    pub secret_scan: Option<SecretPolicy>,

    /// Paths (substrings) to skip during secret scanning, e.g. tests/ fixtures/.
    /// Overrides the config file's secret_scan_allow_paths.
    #[clap(long = "secret-scan-allow", value_name = "FRAGMENT")]
    pub secret_scan_allow: Vec<String>,
}

/// Built-in template names for `--template` completion. Must be `'static`.
fn template_candidates() -> Vec<CompletionCandidate> {
    gnaw_core::builtin_templates::BuiltinTemplates::get_all() // swap for your real accessor
        .iter()
        .map(|(key, tpl)| CompletionCandidate::new(*key).help(Some(tpl.description.into())))
        .collect()
}

/// Valid `--compress-strip` tokens plus their `no-` negations.
fn compress_strip_candidates() -> Vec<CompletionCandidate> {
    ["tests", "fn-bodies", "doc-comments", "private-bodies"]
        .iter()
        .flat_map(|t| {
            [
                CompletionCandidate::new(*t),
                CompletionCandidate::new(format!("no-{t}")),
            ]
        })
        .collect()
}
