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
#[derive(Debug)]
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
    /// Creates a new builder with default settings (mostly false or empty).
    /// The only mandatory parameter is the root `path`.
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            include_patterns: vec![],
            exclude_patterns: vec![],
            include_priority: false,
            line_numbers: false,
            relative_paths: false,
            full_directory_tree: false,
            no_codeblock: false,
            follow_symlinks: false,
            hidden: false,
            no_ignore: false,
            sort_method: None,
            output_format: OutputFormat::Markdown,
            custom_template: None,
            encoding: TokenizerType::Cl100kBase,
            token_format: TokenFormat::Raw,
            diff_enabled: false,
            diff_branches: None,
            log_branches: None,
        }
    }

    /// Sets the list of include patterns.
    pub fn include_patterns(mut self, patterns: Vec<String>) -> Self {
        self.include_patterns = patterns;
        self
    }

    /// Sets the list of exclude patterns.
    pub fn exclude_patterns(mut self, patterns: Vec<String>) -> Self {
        self.exclude_patterns = patterns;
        self
    }

    /// Specifies whether include patterns should take priority over exclude patterns.
    pub fn include_priority(mut self, value: bool) -> Self {
        self.include_priority = value;
        self
    }

    /// Specifies whether code lines should be numbered in the output.
    pub fn line_numbers(mut self, value: bool) -> Self {
        self.line_numbers = value;
        self
    }

    /// Specifies whether paths in the output should be relative instead of absolute.
    pub fn relative_paths(mut self, value: bool) -> Self {
        self.relative_paths = value;
        self
    }

    /// Includes an entire directory tree (only for display), ignoring filtering for tree purposes.
    pub fn full_directory_tree(mut self, value: bool) -> Self {
        self.full_directory_tree = value;
        self
    }

    /// Disables Markdown fences (```).
    pub fn no_codeblock(mut self, value: bool) -> Self {
        self.no_codeblock = value;
        self
    }

    /// Follows symbolic links if set to true.
    pub fn follow_symlinks(mut self, value: bool) -> Self {
        self.follow_symlinks = value;
        self
    }

    /// Includes hidden files and directories if set to true.
    pub fn hidden(mut self, value: bool) -> Self {
        self.hidden = value;
        self
    }

    /// If true, .gitignore rules are ignored (all files included unless excluded by patterns).
    pub fn no_ignore(mut self, value: bool) -> Self {
        self.no_ignore = value;
        self
    }

    /// Sets the file sorting method (by name or by modification time).
    pub fn sort_method(mut self, method: Option<FileSortMethod>) -> Self {
        self.sort_method = method;
        self
    }

    /// Sets the output format (Markdown, JSON, or XML).
    pub fn output_format(mut self, format: OutputFormat) -> Self {
        self.output_format = format;
        self
    }

    /// Specifies a custom Handlebars template string, overriding any default template.
    pub fn custom_template(mut self, template: Option<String>) -> Self {
        self.custom_template = template;
        self
    }

    /// Selects the tokenizer to use for counting tokens.
    pub fn encoding(mut self, tokenizer: TokenizerType) -> Self {
        self.encoding = tokenizer;
        self
    }

    /// Selects the token counting format (raw or formatted).
    pub fn token_format(mut self, format: TokenFormat) -> Self {
        self.token_format = format;
        self
    }

    /// Enables or disables git diff retrieval for HEAD vs index.
    pub fn diff_enabled(mut self, value: bool) -> Self {
        self.diff_enabled = value;
        self
    }

    /// Specifies two branches for which code2prompt should generate a git diff.
    pub fn diff_branches(mut self, branches: Option<(String, String)>) -> Self {
        self.diff_branches = branches;
        self
    }

    /// Specifies two branches for which code2prompt should retrieve the git log.
    pub fn log_branches(mut self, branches: Option<(String, String)>) -> Self {
        self.log_branches = branches;
        self
    }

    /// Consumes the builder and returns a fully constructed `Code2PromptConfig`.
    pub fn build(self) -> Code2PromptConfig {
        Code2PromptConfig {
            path: self.path,
            include_patterns: self.include_patterns,
            exclude_patterns: self.exclude_patterns,
            include_priority: self.include_priority,
            line_numbers: self.line_numbers,
            relative_paths: self.relative_paths,
            full_directory_tree: self.full_directory_tree,
            no_codeblock: self.no_codeblock,
            follow_symlinks: self.follow_symlinks,
            hidden: self.hidden,
            no_ignore: self.no_ignore,
            sort_method: self.sort_method,
            output_format: self.output_format,
            custom_template: self.custom_template,
            encoding: self.encoding,
            token_format: self.token_format,
            diff_enabled: self.diff_enabled,
            diff_branches: self.diff_branches,
            log_branches: self.log_branches,
        }
    }
}
