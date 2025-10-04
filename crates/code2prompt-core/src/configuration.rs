//! This module defines the `Code2PromptConfig` struct and its Builder for configuring the behavior
//! of code2prompt in a stateless manner. It includes all parameters needed for file traversal,
//! code filtering, token counting, and more.

use crate::template::OutputFormat;
use crate::tokenizer::TokenizerType;
use crate::{sort::FileSortMethod, tokenizer::TokenFormat};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
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

    /// If true, token counting will be performed for each file (for token map display)
    #[builder(default)]
    pub token_map_enabled: bool,
}

impl Code2PromptConfig {
    pub fn builder() -> Code2PromptConfigBuilder {
        Code2PromptConfigBuilder::default()
    }
}

/// Output destination for code2prompt
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum OutputDestination {
    #[default]
    Stdout,
    Clipboard,
    File,
}

/// TOML configuration structure that can be serialized/deserialized
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TomlConfig {
    /// Default output behavior: "stdout", "clipboard", or "file"
    #[serde(default)]
    pub default_output: OutputDestination,

    /// Path to the codebase directory
    #[serde(default)]
    pub path: Option<String>,

    /// Patterns to include
    #[serde(default)]
    pub include_patterns: Vec<String>,

    /// Patterns to exclude
    #[serde(default)]
    pub exclude_patterns: Vec<String>,

    /// Display options
    #[serde(default)]
    pub line_numbers: bool,
    #[serde(default)]
    pub absolute_path: bool,
    #[serde(default)]
    pub full_directory_tree: bool,

    /// Output format
    #[serde(default)]
    pub output_format: Option<String>,

    /// Sort method
    #[serde(default)]
    pub sort_method: Option<String>,

    /// Tokenizer settings
    #[serde(default)]
    pub encoding: Option<String>,
    #[serde(default)]
    pub token_format: Option<String>,

    /// Git settings
    #[serde(default)]
    pub diff_enabled: bool,
    #[serde(default)]
    pub diff_branches: Option<Vec<String>>,
    #[serde(default)]
    pub log_branches: Option<Vec<String>>,

    /// Template settings
    #[serde(default)]
    pub template_name: Option<String>,
    #[serde(default)]
    pub template_str: Option<String>,

    /// User variables
    #[serde(default)]
    pub user_variables: HashMap<String, String>,

    /// Token map
    #[serde(default)]
    pub token_map_enabled: bool,
}

impl TomlConfig {
    /// Load TOML configuration from a string
    pub fn from_toml_str(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }

    /// Convert TOML configuration to string
    pub fn to_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// Convert TomlConfig to Code2PromptConfig
    pub fn to_code2prompt_config(&self) -> Code2PromptConfig {
        let mut builder = Code2PromptConfig::builder();

        if let Some(path) = &self.path {
            builder.path(PathBuf::from(path));
        }

        builder
            .include_patterns(self.include_patterns.clone())
            .exclude_patterns(self.exclude_patterns.clone())
            .line_numbers(self.line_numbers)
            .absolute_path(self.absolute_path)
            .full_directory_tree(self.full_directory_tree);

        if let Some(output_format) = &self.output_format
            && let Ok(format) = output_format.parse::<OutputFormat>()
        {
            builder.output_format(format);
        }

        if let Some(sort_method) = &self.sort_method
            && let Ok(method) = sort_method.parse::<FileSortMethod>()
        {
            builder.sort_method(Some(method));
        }

        if let Some(encoding) = &self.encoding
            && let Ok(tokenizer) = encoding.parse::<TokenizerType>()
        {
            builder.encoding(tokenizer);
        }

        if let Some(token_format) = &self.token_format
            && let Ok(format) = token_format.parse::<TokenFormat>()
        {
            builder.token_format(format);
        }

        builder.diff_enabled(self.diff_enabled);

        if let Some(diff_branches) = &self.diff_branches
            && diff_branches.len() == 2
        {
            builder.diff_branches(Some((diff_branches[0].clone(), diff_branches[1].clone())));
        }

        if let Some(log_branches) = &self.log_branches
            && log_branches.len() == 2
        {
            builder.log_branches(Some((log_branches[0].clone(), log_branches[1].clone())));
        }

        if let Some(template_name) = &self.template_name {
            builder.template_name(template_name.clone());
        }

        if let Some(template_str) = &self.template_str {
            builder.template_str(template_str.clone());
        }

        builder
            .user_variables(self.user_variables.clone())
            .token_map_enabled(self.token_map_enabled);

        builder.build().unwrap_or_default()
    }
}

/// Export a Code2PromptConfig to TOML format
pub fn export_config_to_toml(config: &Code2PromptConfig) -> Result<String, toml::ser::Error> {
    let toml_config = TomlConfig {
        default_output: OutputDestination::Stdout, // Default for new behavior
        path: Some(config.path.to_string_lossy().to_string()),
        include_patterns: config.include_patterns.clone(),
        exclude_patterns: config.exclude_patterns.clone(),
        line_numbers: config.line_numbers,
        absolute_path: config.absolute_path,
        full_directory_tree: config.full_directory_tree,
        output_format: Some(config.output_format.to_string()),
        sort_method: config.sort_method.as_ref().map(|s| s.to_string()),
        encoding: Some(config.encoding.to_string()),
        token_format: Some(config.token_format.to_string()),
        diff_enabled: config.diff_enabled,
        diff_branches: config
            .diff_branches
            .as_ref()
            .map(|(a, b)| vec![a.clone(), b.clone()]),
        log_branches: config
            .log_branches
            .as_ref()
            .map(|(a, b)| vec![a.clone(), b.clone()]),
        template_name: if config.template_name.is_empty() {
            None
        } else {
            Some(config.template_name.clone())
        },
        template_str: if config.template_str.is_empty() {
            None
        } else {
            Some(config.template_str.clone())
        },
        user_variables: config.user_variables.clone(),
        token_map_enabled: config.token_map_enabled,
    };

    toml_config.to_string()
}
