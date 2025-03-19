//! This module defines the `Code2PromptConfig` struct and its Builder for configuring the behavior
//! of code2prompt in a stateless manner. It includes all parameters needed for file traversal,
//! code filtering, token counting, and more.

use crate::template::OutputFormat;
use crate::tokenizer::TokenizerType;
use crate::{sort::FileSortMethod, tokenizer::TokenFormat};
use derive_builder::Builder;
use std::collections::HashMap;
use std::path::PathBuf;

/// A stateless configuration object describing all the preferences and filters
/// applied when generating a code prompt. It does not store any mutable data,
/// so it can be cloned freely or shared across multiple sessions.
#[derive(Debug, Clone, Default, Builder)]
#[builder(setter(into), default)]
pub struct Code2PromptConfig {
    /// Path to the root directory of the codebase.
    pub path: PathBuf,

    /// List of glob-like patterns to include.
    #[builder(default)]
    pub include_patterns: Vec<String>,

    /// List of glob-like patterns to exclude.
    #[builder(default)]
    pub exclude_patterns: Vec<String>,

    /// If true, include patterns have priority over exclude patterns in case of conflicts.
    #[builder(default)]
    pub include_priority: bool,

    /// If true, code lines will be numbered in the output.
    #[builder(default)]
    pub line_numbers: bool,

    /// If true, paths in the output will be absolute instead of relative.
    #[builder(default)]
    pub absolute_path: bool,

    /// If true, code2prompt will generate a full directory tree, ignoring include/exclude rules.
    #[builder(default)]
    pub full_directory_tree: bool,

    /// If true, code blocks will not be wrapped in Markdown fences (```).
    #[builder(default)]
    pub no_codeblock: bool,

    /// If true, symbolic links will be followed during traversal.
    #[builder(default)]
    pub follow_symlinks: bool,

    /// If true, hidden files and directories will be included.
    #[builder(default)]
    pub hidden: bool,

    /// If true, .gitignore rules will be ignored.
    #[builder(default)]
    pub no_ignore: bool,

    /// Defines the sorting method for files.
    #[builder(default)]
    pub sort_method: Option<FileSortMethod>,

    /// Determines the output format of the final prompt.
    #[builder(default)]
    pub output_format: OutputFormat,

    /// An optional custom Handlebars template string.
    #[builder(default)]
    pub custom_template: Option<String>,

    /// The tokenizer encoding to use for counting tokens.
    #[builder(default)]
    pub encoding: TokenizerType,

    /// The counting format to use for token counting.
    #[builder(default)]
    pub token_format: TokenFormat,

    /// If true, the git diff between HEAD and index will be included.
    #[builder(default)]
    pub diff_enabled: bool,

    /// If set, contains two branch names for which code2prompt will generate a git diff.
    #[builder(default)]
    pub diff_branches: Option<(String, String)>,

    /// If set, contains two branch names for which code2prompt will retrieve the git log.
    #[builder(default)]
    pub log_branches: Option<(String, String)>,

    /// The name of the template used.
    #[builder(default)]
    pub template_name: String,

    /// The template string itself.
    #[builder(default)]
    pub template_str: String,

    /// Extra template data
    #[builder(default)]
    pub user_variables: HashMap<String, String>,
}

impl Code2PromptConfig {
    pub fn builder() -> Code2PromptConfigBuilder {
        Code2PromptConfigBuilder::default()
    }
}
