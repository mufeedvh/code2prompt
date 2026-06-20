//! This module defines the `GnawConfig` struct and its Builder for configuring the behavior
//! of gnaw in a stateless manner. It includes all parameters needed for file traversal,
//! code filtering, token counting, and more.
use crate::secret_scan::SecretPolicy;
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
pub struct GnawConfig {
    /// Path to the root directory of the codebase.
    pub path: PathBuf,

    /// List of glob-like patterns to include.
    pub include_patterns: Vec<String>,

    /// List of glob-like patterns to exclude.
    pub exclude_patterns: Vec<String>,

    /// If true, code lines will be numbered in the output.
    pub line_numbers: bool,

    /// If true, paths in the output will be absolute instead of relative.
    pub absolute_path: bool,

    /// If true, gnaw will generate a full directory tree, ignoring include/exclude rules.
    pub full_directory_tree: bool,

    /// If true, code blocks will not be wrapped in Markdown fences (```).
    pub no_codeblock: bool,

    /// If true, symbolic links will be followed during traversal.
    pub follow_symlinks: bool,

    /// If true, hidden files and directories will be included.
    pub hidden: bool,

    /// If true, .gitignore rules will be ignored.
    pub no_ignore: bool,

    /// Defines the sorting method for files.
    pub sort_method: Option<FileSortMethod>,

    /// Determines the output format of the final prompt.
    pub output_format: OutputFormat,

    /// An optional custom Handlebars template string.
    pub custom_template: Option<String>,

    /// The tokenizer encoding to use for counting tokens.
    pub encoding: TokenizerType,

    /// The counting format to use for token counting.
    pub token_format: TokenFormat,

    /// If true, the git diff between HEAD and index will be included.
    pub diff_enabled: bool,

    /// If set, contains two branch names for which gnaw will generate a git diff.
    pub diff_branches: Option<(String, String)>,

    /// (ref1, ref2) for changed-file before/after extraction (`--git-diff-shas`).
    pub diff_shas: Option<(String, String)>,
    /// What `--git-diff-shas` emits per file.
    pub diff_shas_content: DiffShaContent,
    /// Skip files larger than this many bytes (0 = no limit).
    pub diff_shas_max_bytes: usize,

    /// If set, contains two branch names for which gnaw will retrieve the git log.
    pub log_branches: Option<(String, String)>,

    /// The name of the template used.
    pub template_name: String,

    /// The template string itself.
    pub template_str: String,

    /// Extra template data
    pub user_variables: HashMap<String, String>,

    /// If true, detailed token map breakdown will be displayed in output.
    ///
    /// Note: Token counting always happens internally for performance optimization
    /// (parallelized during file I/O). This flag only controls whether the breakdown
    /// is shown to users in the final output.
    pub token_map_enabled: bool,

    /// If true, starts with all files deselected.
    pub deselected: bool,

    /// Syntax-aware compression. No-op without the `compression` feature or for
    /// unsupported languages.
    pub compression: CompressionOptions,

    /// Which working-tree changes `load_git_diff` should show.
    pub diff_mode: DiffMode,

    pub secret_scan: SecretPolicy,

    #[builder(default)]
    pub secret_scan_allow_paths: Vec<String>,

    /// True when the resolved template reasons about a *change* (commit,
    /// changeset, PR). Computed at config-build time from the template
    /// selection — NOT a user-set knob, NOT serialized to TOML. The pipeline
    /// reads this to scope the source tree to changed files; it never
    /// re-derives the intent from the template name.
    pub git_narrative: bool,
}

impl GnawConfig {
    pub fn builder() -> GnawConfigBuilder {
        GnawConfigBuilder::default()
    }
}

/// What `--git-diff-shas` emits per changed file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Deserialize)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[serde(rename_all = "kebab-case")]
pub enum DiffShaContent {
    /// Patch only; full `after` body for additions. ~1x changed content.
    Patch,
    #[default]
    AfterPatch, // full after for every file + patch; no before  ← the lean+useful one
    /// Full before + after, no patch. ~2x.
    Full,
    /// Full before + after plus the patch. Heaviest.
    FullPatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize, Default)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[serde(rename_all = "lowercase")]
pub enum DiffMode {
    #[cfg_attr(feature = "clap", value(name = "staged"))]
    Staged,
    #[default]
    #[cfg_attr(feature = "clap", value(name = "unstaged"))]
    Unstaged,
    #[cfg_attr(feature = "clap", value(name = "all"))]
    All,
}

/// Output destination for gnaw
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum OutputDestination {
    #[default]
    Stdout,
    Clipboard,
    File,
}

/// TOML configuration structure that can be serialized/deserialized
#[derive(Debug, Clone, Serialize, Deserialize, Default, Builder)]
#[serde(default)]
pub struct TomlConfig {
    /// Default output behavior: "stdout", "clipboard", or "file"
    pub default_output: OutputDestination,

    /// Path to the codebase directory
    pub path: Option<String>,

    /// Patterns to include
    pub include_patterns: Vec<String>,

    /// Patterns to exclude
    pub exclude_patterns: Vec<String>,

    /// Display options
    pub line_numbers: bool,
    pub absolute_path: bool,
    pub full_directory_tree: bool,

    /// Output format
    pub output_format: Option<OutputFormat>,

    /// Sort method
    pub sort_method: Option<FileSortMethod>,

    /// Tokenizer settings
    pub encoding: Option<TokenizerType>,
    pub token_format: Option<TokenFormat>,

    /// Git settings
    pub diff_enabled: bool,
    pub diff_branches: Option<Vec<String>>,
    pub log_branches: Option<Vec<String>>,

    /// Template settings
    pub template_name: Option<String>,
    pub template_str: Option<String>,

    /// User variables
    pub user_variables: HashMap<String, String>,

    /// Token map
    pub token_map_enabled: bool,

    /// Initial selection state
    pub deselected: bool,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compression: Option<CompressionOptions>,

    pub secret_scan: Option<SecretPolicy>,

    #[builder(default)]
    pub secret_scan_allow_paths: Vec<String>,
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

    /// Convert TomlConfig to GnawConfig
    pub fn to_gnaw_config(&self) -> GnawConfig {
        let mut builder = GnawConfig::builder();

        if let Some(path) = &self.path {
            builder.path(PathBuf::from(path));
        }

        builder
            .include_patterns(self.include_patterns.clone())
            .exclude_patterns(self.exclude_patterns.clone())
            .line_numbers(self.line_numbers)
            .absolute_path(self.absolute_path)
            .full_directory_tree(self.full_directory_tree);

        builder.output_format(self.output_format.unwrap_or_default());

        builder.sort_method(self.sort_method);

        builder.encoding(self.encoding.unwrap_or_default());

        builder.token_format(self.token_format.unwrap_or_default());

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
            .token_map_enabled(self.token_map_enabled)
            .deselected(self.deselected)
            .compression(self.compression.unwrap_or_default());

        builder.build().unwrap_or_default()
    }
}

/// Export a GnawConfig to TOML format
pub fn export_config_to_toml(config: &GnawConfig) -> Result<String, toml::ser::Error> {
    let toml_config = TomlConfig {
        default_output: OutputDestination::Stdout, // Default for new behavior
        path: Some(config.path.to_string_lossy().to_string()),
        include_patterns: config.include_patterns.clone(),
        exclude_patterns: config.exclude_patterns.clone(),
        line_numbers: config.line_numbers,
        absolute_path: config.absolute_path,
        full_directory_tree: config.full_directory_tree,
        output_format: Some(config.output_format),
        sort_method: config.sort_method,
        encoding: Some(config.encoding),
        token_format: Some(config.token_format),
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
        deselected: config.deselected,
        compression: if config.compression.any() {
            Some(config.compression)
        } else {
            None
        },
        secret_scan: if config.secret_scan == SecretPolicy::default() {
            None
        } else {
            Some(config.secret_scan)
        },
        secret_scan_allow_paths: config.secret_scan_allow_paths.clone(),
    };

    toml_config.to_string()
}

/// Independent, composable compression transforms. Presets resolve to a set of
/// these at the CLI layer, so the core engine never needs to know about levels.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompressionOptions {
    pub strip_test_modules: bool,
    pub strip_fn_bodies: bool,
    pub strip_doc_comments: bool,
    pub strip_private_bodies: bool, // renamed from strip_private_items
}

impl CompressionOptions {
    pub fn any(&self) -> bool {
        self.strip_test_modules
            || self.strip_fn_bodies
            || self.strip_doc_comments
            || self.strip_private_bodies
    }
}

/// Named aggressiveness presets (not a quality ranking).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[serde(rename_all = "lowercase")]
pub enum CompressionLevel {
    #[cfg_attr(feature = "clap", value(name = "light"))]
    Light,
    #[cfg_attr(feature = "clap", value(name = "moderate"))]
    Moderate,
    #[cfg_attr(feature = "clap", value(name = "full"))]
    Full,
}

impl CompressionLevel {
    pub fn options(self) -> CompressionOptions {
        match self {
            CompressionLevel::Light => CompressionOptions {
                strip_test_modules: true,
                ..Default::default()
            },
            CompressionLevel::Moderate => CompressionOptions {
                strip_test_modules: true,
                strip_fn_bodies: true,
                ..Default::default()
            },
            // Full strips ALL bodies (fn_bodies), so private_bodies is subsumed and
            // intentionally left out — it's a manual-only lever for the public-API view.
            CompressionLevel::Full => CompressionOptions {
                strip_test_modules: true,
                strip_fn_bodies: true,
                strip_doc_comments: true,
                ..Default::default()
            },
        }
    }
}

// bottom of crates/gnaw-core/src/configuration.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compression_survives_toml_round_trip() {
        let cfg = GnawConfig {
            compression: CompressionOptions {
                strip_fn_bodies: true,
                strip_doc_comments: true,
                ..Default::default()
            },
            ..Default::default()
        };
        let toml = export_config_to_toml(&cfg).unwrap();
        let back = TomlConfig::from_toml_str(&toml).unwrap().to_gnaw_config();
        assert_eq!(back.compression, cfg.compression);
    }

    #[test]
    fn empty_compression_omitted_from_toml() {
        // a no-compression config shouldn't emit a [compression] table
        let cfg = GnawConfig::default();
        let toml = export_config_to_toml(&cfg).unwrap();
        assert!(!toml.contains("[compression]"));
    }
}
