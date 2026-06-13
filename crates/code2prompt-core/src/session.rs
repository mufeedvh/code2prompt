//! This module defines a Code2promptSession struct that provide a stateful interface to code2prompt-core.
//! It allows you to load codebase data, Git info, and render prompts using a template.

use anyhow::{Context, Result};
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::analysis::CodebaseAnalysis;
use crate::configuration::Code2PromptConfig;
use crate::git::{get_git_diff, get_git_diff_between_branches, get_git_log};
use crate::path::{FileEntry, display_name, traverse_directory, wrap_code_block};
use crate::selection::SelectionEngine;
use crate::template::{OutputFormat, handlebars_setup, render_template};
use crate::tokenizer::{TokenizerType, count_tokens};

/// Main orchestrator for code prompt generation workflows.
/// 
/// Combines configuration, file processing, and template rendering into
/// a cohesive session. Maintains state during processing and coordinates
/// between filtering logic, codebase analysis, and output generation.
/// 
/// This struct acts as the high-level controller that binds together:
/// - Static Configuration ([`Code2PromptConfig`])
/// - Filtering Logic ([`SelectionEngine`])
/// - Codebase Data ([`SessionData`])
/// 
/// # Example
/// 
/// ```rust
/// use code2prompt_core::configuration::Code2PromptConfig;
/// use code2prompt_core::session::Code2PromptSession;
/// 
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a temporary directory for testing
/// let temp_dir = std::env::temp_dir().join("code2prompt_test");
/// std::fs::create_dir_all(&temp_dir)?;
/// std::fs::write(temp_dir.join("test.rs"), "fn main() {}")?;
/// 
/// let config = Code2PromptConfig::builder()
///     .path(&temp_dir)
///     .build()?;
/// let mut session = Code2PromptSession::new(config);
/// let output = session.generate_prompt()?;
/// println!("Generated {} tokens", output.token_count);
/// 
/// // Cleanup
/// std::fs::remove_dir_all(&temp_dir).ok();
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Code2PromptSession {
    pub config: Code2PromptConfig,
    pub selection_engine: SelectionEngine,
    pub data: SessionData,
}

/// Represents the collected data about the code (tree + files) and optional Git info.
/// It acts as a "Single Source of Truth" and persists throughout the session.
#[derive(Debug, Default, Clone)]
pub struct SessionData {
    pub absolute_code_path: Option<String>,
    pub source_tree: Option<String>,
    pub files: Option<Arc<Vec<FileEntry>>>,
    pub stats: Option<serde_json::Value>,
    pub git_diff: Option<String>,
    pub git_diff_branch: Option<String>,
    pub git_log_branch: Option<String>,
}

/// Represents a transient, zero-copy data view for template rendering
/// Created on-the-fly, it borrows references (`&'a`) from `SessionData` to avoid cloning heavy file content.
#[derive(Serialize)]
pub struct TemplateContext<'a> {
    pub absolute_code_path: &'a str,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_tree: &'a Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<&'a [FileEntry]>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_diff: &'a Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_diff_branch: &'a Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_log_branch: &'a Option<String>,

    #[serde(flatten)]
    pub user_variables: &'a HashMap<String, String>,

    pub no_codeblock: bool,
}

/// Encapsulates the final rendered prompt and some metadata
#[derive(Debug, Clone)]
pub struct RenderedPrompt {
    pub prompt: String,
    pub directory_name: String,
    pub token_count: usize,
    pub model_info: &'static str,
    pub files: Vec<String>,
}

impl Code2PromptSession {
    /// Create new session with configuration and selection engine.
    /// 
    /// Initializes a session with the provided configuration and creates
    /// a selection engine for pattern-based and user-driven file filtering.
    /// The project path is taken from the configuration.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use code2prompt_core::configuration::Code2PromptConfig;
    /// use code2prompt_core::session::Code2PromptSession;
    /// 
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // Use current directory for testing
    /// let current_dir = std::env::current_dir()?;
    /// 
    /// let config = Code2PromptConfig::builder()
    ///     .path(&current_dir)
    ///     .include_patterns(vec!["**/*.rs".to_string()])
    ///     .build()?;
    /// 
    /// let session = Code2PromptSession::new(config);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(config: Code2PromptConfig) -> Self {
        let selection_engine = SelectionEngine::new(
            config.include_patterns.clone(),
            config.exclude_patterns.clone(),
            config.deselected,
        );

        Self {
            selection_engine,
            config,
            data: SessionData::default(),
        }
    }

    /// Add pattern and recreate SelectionEngine
    pub fn add_include_pattern(&mut self, pattern: String) -> &mut Self {
        self.config.include_patterns.push(pattern);
        // Recreate SelectionEngine with new patterns
        self.selection_engine = SelectionEngine::new(
            self.config.include_patterns.clone(),
            self.config.exclude_patterns.clone(),
            self.config.deselected,
        );
        self
    }

    pub fn add_exclude_pattern(&mut self, pattern: String) -> &mut Self {
        self.config.exclude_patterns.push(pattern);
        // Recreate SelectionEngine with new patterns
        self.selection_engine = SelectionEngine::new(
            self.config.include_patterns.clone(),
            self.config.exclude_patterns.clone(),
            self.config.deselected,
        );
        self
    }

    /// User interaction: include a file (delegates to SelectionEngine)
    pub fn select_file(&mut self, path: PathBuf) -> &mut Self {
        let relative_path = if path.is_absolute() {
            path.strip_prefix(&self.config.path)
                .unwrap_or(&path)
                .to_path_buf()
        } else {
            path
        };

        self.selection_engine.include_file(relative_path);
        self
    }

    /// User interaction: exclude a file (delegates to SelectionEngine)
    pub fn deselect_file(&mut self, path: PathBuf) -> &mut Self {
        let relative_path = if path.is_absolute() {
            path.strip_prefix(&self.config.path)
                .unwrap_or(&path)
                .to_path_buf()
        } else {
            path
        };

        self.selection_engine.exclude_file(relative_path);
        self
    }

    /// User interaction: toggle file selection (delegates to SelectionEngine)
    pub fn toggle_file_selection(&mut self, path: PathBuf) -> &mut Self {
        let relative_path = if path.is_absolute() {
            path.strip_prefix(&self.config.path)
                .unwrap_or(&path)
                .to_path_buf()
        } else {
            path
        };

        self.selection_engine.toggle_file(relative_path);
        self
    }

    /// Check if a file is selected (delegates to SelectionEngine)
    pub fn is_file_selected(&mut self, path: &std::path::Path) -> bool {
        let relative_path = if path.is_absolute() {
            path.strip_prefix(&self.config.path).unwrap_or(path)
        } else {
            path
        };

        self.selection_engine.is_selected(relative_path)
    }

    /// Get all currently selected files (delegates to SelectionEngine)
    pub fn get_selected_files(&mut self) -> Result<Vec<PathBuf>> {
        Ok(self
            .selection_engine
            .get_selected_files(&self.config.path)?)
    }

    /// Clear all user actions (reset to pattern-only behavior)
    pub fn clear_user_actions(&mut self) -> &mut Self {
        self.selection_engine.clear_user_actions();
        self
    }

    /// Check if there are any user actions beyond base patterns
    pub fn has_user_actions(&self) -> bool {
        self.selection_engine.has_user_actions()
    }

    /// Set deselected by default and update selection engine
    pub fn set_deselected(&mut self, value: bool) -> &mut Self {
        self.config.deselected = value;
        self.selection_engine.set_deselected_by_default(value);
        self
    }

    /// Loads the codebase data (source tree and file list) into the session.
    pub fn load_codebase(&mut self) -> Result<()> {
        let (tree, files) = traverse_directory(&self.config, Some(&mut self.selection_engine))
            .with_context(|| "Failed to traverse directory")?;

        // Store absolute_code_path as Single Source of Truth
        self.data.absolute_code_path = Some(display_name(&self.config.path));
        self.data.source_tree = Some(tree);
        self.data.files = Some(Arc::new(files));

        Ok(())
    }

    /// Loads the Git diff into the session data.
    pub fn load_git_diff(&mut self) -> Result<()> {
        let diff = get_git_diff(&self.config.path)?;
        self.data.git_diff = Some(diff);
        Ok(())
    }

    /// Loads the Git diff between two branches into the session data.
    pub fn load_git_diff_between_branches(&mut self) -> Result<()> {
        if let Some((b1, b2)) = &self.config.diff_branches {
            let diff = get_git_diff_between_branches(&self.config.path, b1, b2)?;
            self.data.git_diff_branch = Some(diff);
        }
        Ok(())
    }

    /// Loads the Git log between two branches into the session data.
    pub fn load_git_log_between_branches(&mut self) -> Result<()> {
        if let Some((b1, b2)) = &self.config.log_branches {
            let log_output = get_git_log(&self.config.path, b1, b2)?;
            self.data.git_log_branch = Some(log_output);
        }
        Ok(())
    }

    /// Constructs a zero-copy template context for rendering.
    pub fn build_template_data(&self) -> TemplateContext<'_> {
        TemplateContext {
            absolute_code_path: self.data.absolute_code_path.as_deref().unwrap_or("unknown"),
            source_tree: &self.data.source_tree,
            files: self.data.files.as_deref().map(|v| v.as_slice()),
            git_diff: &self.data.git_diff,
            git_diff_branch: &self.data.git_diff_branch,
            git_log_branch: &self.data.git_log_branch,
            user_variables: &self.config.user_variables,
            no_codeblock: self.config.no_codeblock,
        }
    }

    /// Create a "raw" (intrinsic) analysis of the loaded codebase
    ///
    /// This analysis uses the sum of per-file token counts (Σ FileEntry.token_count)
    /// without including template structural overhead.
    ///
    /// **Use case**: Before prompt generation, for cost estimation, or for
    /// pure codebase statistics exploration (e.g., in TUI Statistics tab).
    ///
    /// # Returns
    ///
    /// * `Option<CodebaseAnalysis>` - Analysis facade if files are loaded, None otherwise
    pub fn raw_analysis(&self) -> Option<CodebaseAnalysis<'_>> {
        self.data.files.as_ref().map(|files| {
            let raw_token_sum: usize = files.iter().map(|f| f.token_count).sum();
            CodebaseAnalysis::new(files.as_slice(), raw_token_sum)
        })
    }

    /// Create a "contextual" (post-generation) analysis based on a rendered prompt
    ///
    /// This analysis uses the token count from a RenderedPrompt, which includes
    /// both file content tokens AND template structural overhead (tree, git info, etc.).
    ///
    /// **Use case**: After `generate_prompt()`, when you need analysis in the context
    /// of the actual rendered output (e.g., token map showing real percentages).
    ///
    /// # Arguments
    ///
    /// * `prompt` - A RenderedPrompt containing the contextual token count
    ///
    /// # Returns
    ///
    /// * `Option<CodebaseAnalysis>` - Analysis facade if files are loaded, None otherwise
    pub fn contextual_analysis(&self, prompt: &RenderedPrompt) -> Option<CodebaseAnalysis<'_>> {
        self.data
            .files
            .as_ref()
            .map(|files| CodebaseAnalysis::new(files.as_slice(), prompt.token_count))
    }

    /// Renders the final prompt given a template context. Returns both
    /// the rendered prompt and the token count information.
    pub fn render_prompt(&self, template_context: &TemplateContext) -> Result<RenderedPrompt> {
        // ~~~ Template selection ~~~
        let mut template_str = self.config.template_str.clone();
        let mut template_name = self.config.template_name.clone();
        if self.config.template_str.is_empty() {
            template_str = match self.config.output_format {
                OutputFormat::Markdown => include_str!("./default_template_md.hbs").to_string(),
                OutputFormat::Xml | OutputFormat::Json => {
                    include_str!("./default_template_xml.hbs").to_string()
                }
            };
            template_name = match self.config.output_format {
                OutputFormat::Markdown => "markdown".to_string(),
                OutputFormat::Xml | OutputFormat::Json => "xml".to_string(),
            };
        }

        // ~~~ Rendering ~~~
        let handlebars = handlebars_setup(&template_str, &template_name)?;
        let template_content = render_template(&handlebars, &template_name, template_context)?;

        // ~~~ Informations ~~~
        let tokenizer_type: TokenizerType = self.config.encoding;
        // Always use the cached calculation: Σ(FileTokens) + TemplateOverhead
        let token_count = self.calculate_token_count_from_cache(&tokenizer_type);

        let model_info = tokenizer_type.description();
        let directory_name = template_context.absolute_code_path.to_string();
        let files: Vec<String> = self
            .data
            .files
            .as_ref()
            .map(|files| files.iter().map(|file| file.path.clone()).collect())
            .unwrap_or_default();

        // ~~~ Final output format ~~~
        let final_output = match self.config.output_format {
            OutputFormat::Json => {
                let json_data = serde_json::json!({
                    "prompt": template_content,
                    "directory_name": directory_name.clone(),
                    "token_count": token_count,
                    "model_info": model_info,
                    "files": files.clone(),
                });
                serde_json::to_string_pretty(&json_data)?
            }
            _ => template_content,
        };

        Ok(RenderedPrompt {
            prompt: final_output,
            directory_name,
            token_count,
            model_info,
            files,
        })
    }

    /// Calculate exact token count using cached per-file token counts + skeleton rendering
    ///
    /// This method provides precise token counting by:
    /// 1. Summing the cached per-file token counts (from actual content tokenized in parallel)
    /// 2. Rendering a "skeleton" template with empty file contents to get structural tokens
    /// 3. Adding them together for an exact count
    ///
    /// This approach avoids re-tokenizing the entire rendered output (sequential bottleneck).
    ///
    /// # Arguments
    ///
    /// * `tokenizer_type` - The tokenizer to use for tokenization
    ///
    /// # Returns
    ///
    /// * `usize` - The exact total token count
    fn calculate_token_count_from_cache(&self, tokenizer_type: &TokenizerType) -> usize {
        // Sum up cached per-file token counts (tokens from actual file content)
        let files_token_count: usize = self
            .data
            .files
            .as_ref()
            .map(|files| files.iter().map(|file| file.token_count).sum())
            .unwrap_or(0);

        // Calculate exact structural/template overhead using skeleton rendering
        let structural_tokens = self.calculate_structural_tokens(tokenizer_type);

        files_token_count + structural_tokens
    }

    /// Calculate structural tokens by rendering a skeleton template
    ///
    /// Creates FileEntry "skeletons" with empty code blocks but same structure,
    /// renders the template, and counts tokens. This gives us the exact token count
    /// for everything except the actual file content (tree, headers, wrappers, git info).
    ///
    /// # Arguments
    ///
    /// * `tokenizer_type` - The tokenizer to use for counting
    ///
    /// # Returns
    ///
    /// * `usize` - The number of structural tokens
    fn calculate_structural_tokens(&self, tokenizer_type: &TokenizerType) -> usize {
        // Create skeleton file entries (empty code, but same structure/metadata)
        let skeleton_files: Option<Vec<FileEntry>> = self.data.files.as_ref().map(|files| {
            files
                .iter()
                .map(|file| {
                    // Create empty code block with same wrapping structure
                    let empty_code_block = wrap_code_block("", self.config.line_numbers);

                    FileEntry {
                        path: file.path.clone(),
                        extension: file.extension.clone(),
                        code: empty_code_block,
                        token_count: 0, // Not used in skeleton
                        metadata: file.metadata,
                        mod_time: file.mod_time,
                    }
                })
                .collect()
        });

        // Build skeleton template context (same structure, but with empty file contents)
        let skeleton_context = TemplateContext {
            absolute_code_path: self.data.absolute_code_path.as_deref().unwrap_or("unknown"),
            source_tree: &self.data.source_tree,
            files: skeleton_files.as_deref(),
            git_diff: &self.data.git_diff,
            git_diff_branch: &self.data.git_diff_branch,
            git_log_branch: &self.data.git_log_branch,
            user_variables: &self.config.user_variables,
            no_codeblock: self.config.no_codeblock,
        };

        // Render skeleton template
        let template_str = if self.config.template_str.is_empty() {
            match self.config.output_format {
                OutputFormat::Markdown => include_str!("./default_template_md.hbs").to_string(),
                OutputFormat::Xml | OutputFormat::Json => {
                    include_str!("./default_template_xml.hbs").to_string()
                }
            }
        } else {
            self.config.template_str.clone()
        };

        let template_name = if self.config.template_name.is_empty() {
            match self.config.output_format {
                OutputFormat::Markdown => "markdown".to_string(),
                OutputFormat::Xml | OutputFormat::Json => "xml".to_string(),
            }
        } else {
            self.config.template_name.clone()
        };

        // Render and count tokens
        match handlebars_setup(&template_str, &template_name) {
            Ok(handlebars) => {
                match render_template(&handlebars, &template_name, &skeleton_context) {
                    Ok(skeleton_rendered) => count_tokens(&skeleton_rendered, tokenizer_type),
                    Err(_) => {
                        // Fallback to simple estimation if rendering fails
                        self.fallback_structural_estimate(tokenizer_type)
                    }
                }
            }
            Err(_) => {
                // Fallback to simple estimation if handlebars setup fails
                self.fallback_structural_estimate(tokenizer_type)
            }
        }
    }

    /// Fallback estimation when skeleton rendering fails
    ///
    /// Uses a simple heuristic based on tree/git sizes as a safety net.
    ///
    /// # Arguments
    ///
    /// * `tokenizer_type` - The tokenizer to use
    ///
    /// # Returns
    ///
    /// * `usize` - Estimated structural tokens
    fn fallback_structural_estimate(&self, tokenizer_type: &TokenizerType) -> usize {
        let mut total_chars = 0;

        if let Some(tree) = &self.data.source_tree {
            total_chars += tree.len();
        }
        if let Some(diff) = &self.data.git_diff {
            total_chars += diff.len();
        }
        if let Some(diff_branch) = &self.data.git_diff_branch {
            total_chars += diff_branch.len();
        }
        if let Some(log_branch) = &self.data.git_log_branch {
            total_chars += log_branch.len();
        }

        // Simple approximation: ~4 chars per token + buffer for headers
        let estimated = (total_chars / 4) + 100;

        // For better accuracy on smaller sizes, actually tokenize
        if total_chars < 10000 {
            let combined = format!(
                "{}{}{}{}",
                self.data.source_tree.as_deref().unwrap_or(""),
                self.data.git_diff.as_deref().unwrap_or(""),
                self.data.git_diff_branch.as_deref().unwrap_or(""),
                self.data.git_log_branch.as_deref().unwrap_or("")
            );
            count_tokens(&combined, tokenizer_type)
        } else {
            estimated
        }
    }

    /// Process all files and generate final prompt output.
    /// 
    /// Orchestrates the complete workflow by loading codebase data, applying
    /// filters, processing Git information if enabled, and rendering using
    /// the specified template. This is the main entry point for generating
    /// prompts from configured projects.
    /// 
    /// # Returns
    /// 
    /// [`RenderedPrompt`] containing generated content, metadata, and token counts.
    /// 
    /// # Errors
    /// 
    /// Returns error if:
    /// - Project path is not accessible
    /// - File processing fails
    /// - Template rendering fails
    /// - Git operations fail (when enabled)
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use code2prompt_core::configuration::Code2PromptConfig;
    /// use code2prompt_core::session::Code2PromptSession;
    /// 
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // Create a temporary directory for testing
    /// let temp_dir = std::env::temp_dir().join("code2prompt_generate_test");
    /// std::fs::create_dir_all(&temp_dir)?;
    /// std::fs::write(temp_dir.join("test.rs"), "fn main() { println!(\"Hello world!\"); }")?;
    /// 
    /// let config = Code2PromptConfig::builder()
    ///     .path(&temp_dir)
    ///     .diff_enabled(false) // Disable git diff for test
    ///     .build()?;
    /// 
    /// let mut session = Code2PromptSession::new(config);
    /// let output = session.generate_prompt()?;
    /// 
    /// println!("Generated prompt with {} tokens", output.token_count);
    /// println!("Processed {} files", output.files.len());
    /// 
    /// // Cleanup
    /// std::fs::remove_dir_all(&temp_dir).ok();
    /// # Ok(())
    /// # }
    /// ```
    pub fn generate_prompt(&mut self) -> Result<RenderedPrompt> {
        self.load_codebase()?;

        // ~~~~ Load Git info ~~~
        if self.config.diff_enabled {
            match self.load_git_diff() {
                Ok(_) => {}
                Err(e) => log::warn!("Git diff could not be loaded: {}", e),
            }
        }

        // ~~~ Load Git info between branches ~~~
        if self.config.diff_branches.is_some() {
            match self.load_git_diff_between_branches() {
                Ok(_) => {}
                Err(e) => log::warn!("Git branch diff could not be loaded: {}", e),
            }
        }

        // ~~~ Load Git log between branches ~~~
        if self.config.log_branches.is_some() {
            match self.load_git_log_between_branches() {
                Ok(_) => {}
                Err(e) => log::warn!("Git branch log could not be loaded: {}", e),
            }
        }
        let template_data = self.build_template_data();
        let rendered = self.render_prompt(&template_data)?;
        Ok(rendered)
    }
}
