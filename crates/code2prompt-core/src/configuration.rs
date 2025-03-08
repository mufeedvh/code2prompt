//! This module defines the `Code2PromptConfig` struct and its Builder for configuring the behavior
//! of code2prompt in a stateless manner. It includes all parameters needed for file traversal,
//! code filtering, token counting, and more.

use crate::template::OutputFormat;
use crate::tokenizer::TokenizerType;
use crate::{sort::FileSortMethod, tokenizer::TokenFormat};
use std::path::{Path, PathBuf};

/// A stateless configuration object describing all the preferences and filters
/// applied when generating a code prompt. It does not store any mutable data,
/// so it can be cloned freely or shared across multiple sessions.
#[derive(Debug, Clone)]
pub struct Code2PromptConfig {
    /// Path to the root directory of the codebase.
    pub path: PathBuf,

    /// List of glob-like patterns to include.
    pub include_patterns: Vec<String>,

    /// List of glob-like patterns to exclude.
    pub exclude_patterns: Vec<String>,

    /// If true, include patterns have priority over exclude patterns in case of conflicts.
    pub include_priority: bool,

    /// If true, code lines will be numbered in the output.
    pub line_numbers: bool,

    /// If true, paths in the output will be relative instead of absolute.
    pub relative_paths: bool,

    /// If true, code2prompt will generate a full directory tree, ignoring include/exclude rules
    /// only for the tree display (though filtering may still apply to code extraction).
    pub full_directory_tree: bool,

    /// If true, code blocks will not be wrapped in Markdown fences (```).
    pub no_codeblock: bool,

    /// If true, symbolic links will be followed during traversal.
    pub follow_symlinks: bool,

    /// If true, hidden files and directories will be included.
    pub hidden: bool,

    /// If true, .gitignore rules will be ignored (i.e. everything is included unless otherwise excluded).
    pub no_ignore: bool,

    /// Defines the sorting method for files (by name or modification time, ascending or descending).
    pub sort_method: Option<FileSortMethod>,

    /// Determines the output format of the final prompt (Markdown, JSON, or XML).
    pub output_format: OutputFormat,

    /// An optional custom Handlebars template string. If present, it overrides the default template.
    pub custom_template: Option<String>,

    /// The tokenizer encoding to use for counting tokens (e.g. cl100k, p50k, etc.).
    pub encoding: TokenizerType,

    /// The counting format to use for token counting (raw or formatted).
    pub token_format: TokenFormat,

    /// If true, the git diff between HEAD and index will be included in the prompt (if applicable).
    pub diff_enabled: bool,

    /// If set, contains two branch names for which code2prompt will generate a git diff.
    pub diff_branches: Option<(String, String)>,

    /// If set, contains two branch names for which code2prompt will retrieve the git log.
    pub log_branches: Option<(String, String)>,
}

impl Code2PromptConfig {
    /// Creates a new builder for `Code2PromptConfig`, using `path` as the root directory.
    pub fn builder(path: impl AsRef<Path>) -> Code2PromptConfigBuilder {
        Code2PromptConfigBuilder::new(path)
    }
}

/// A builder struct that allows you to customize `Code2PromptConfig` in a fluent style.
/// This is helpful if you have a lot of optional parameters.
#[derive(Debug, Default)]
pub struct Code2PromptConfigBuilder {
    path: PathBuf,
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
    include_priority: bool,
    line_numbers: bool,
    relative_paths: bool,
    full_directory_tree: bool,
    no_codeblock: bool,
    follow_symlinks: bool,
    hidden: bool,
    no_ignore: bool,
    sort_method: Option<FileSortMethod>,
    output_format: OutputFormat,
    custom_template: Option<String>,
    encoding: TokenizerType,
    token_format: TokenFormat,
    diff_enabled: bool,
    diff_branches: Option<(String, String)>,
    log_branches: Option<(String, String)>,
}

impl Code2PromptConfigBuilder {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            ..Default::default()
        }
    }
}
