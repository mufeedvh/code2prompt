//! This module defines a Code2promptSession struct that provide a stateful interface to code2prompt-core.
//! It allows you to load codebase data, Git info, and render prompts using a template.

use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::configuration::Code2PromptConfig;
use crate::git::{get_git_diff, get_git_diff_between_branches, get_git_log};
use crate::path::{label, traverse_directory};
use crate::selection::SelectionEngine;
use crate::template::{OutputFormat, handlebars_setup, render_template};
use crate::tokenizer::{TokenizerType, count_tokens};

/// Represents a live session that holds stateful data about the user's codebase,
/// including which files have been added or removed, or other data that evolves over time.
#[derive(Debug, Clone)]
pub struct Code2PromptSession {
    pub config: Code2PromptConfig,
    pub selection_engine: SelectionEngine,
    pub data: SessionData,
}

/// Represents the collected data about the code (tree + files) and optional Git info.
/// The session loads these pieces separately, so you can manage them step by step.
#[derive(Debug, Default, Clone)]
pub struct SessionData {
    pub source_tree: Option<String>,
    pub files: Option<serde_json::Value>,
    pub stats: Option<serde_json::Value>,
    pub git_diff: Option<String>,
    pub git_diff_branch: Option<String>,
    pub git_log_branch: Option<String>,
}

/// Encapsulates the final rendered prompt and some metadata
#[derive(Debug)]
pub struct RenderedPrompt {
    pub prompt: String,
    pub directory_name: String,
    pub token_count: usize,
    pub model_info: &'static str,
    pub files: Vec<String>,
}

impl Code2PromptSession {
    /// Creates a new session with SelectionEngine for pattern-based and user-driven file selection
    pub fn new(config: Code2PromptConfig) -> Self {
        let selection_engine = SelectionEngine::new(
            config.include_patterns.clone(),
            config.exclude_patterns.clone(),
        );

        Self {
            selection_engine,
            config,
            data: SessionData::default(),
        }
    }

    /// Initialize selections based on patterns (for CLI usage like `code2prompt . -i "*.rs"`)
    pub fn initialize_from_patterns(&mut self) -> Result<()> {
        // This ensures that when TUI starts, files matching patterns are already "selected"
        // The SelectionEngine will handle this automatically through pattern matching
        Ok(())
    }

    /// Add pattern and recreate SelectionEngine
    pub fn add_include_pattern(&mut self, pattern: String) -> &mut Self {
        self.config.include_patterns.push(pattern);
        // Recreate SelectionEngine with new patterns
        self.selection_engine = SelectionEngine::new(
            self.config.include_patterns.clone(),
            self.config.exclude_patterns.clone(),
        );
        self
    }

    pub fn add_exclude_pattern(&mut self, pattern: String) -> &mut Self {
        self.config.exclude_patterns.push(pattern);
        // Recreate SelectionEngine with new patterns
        self.selection_engine = SelectionEngine::new(
            self.config.include_patterns.clone(),
            self.config.exclude_patterns.clone(),
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

    /// Loads the codebase data (source tree and file list) into the session.
    pub fn load_codebase(&mut self) -> Result<()> {
        let (tree, files_json) = traverse_directory(&self.config, Some(&mut self.selection_engine))
            .with_context(|| "Failed to traverse directory")?;

        self.data.source_tree = Some(tree);
        self.data.files = Some(serde_json::Value::Array(files_json));

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

    /// Constructs a JSON object that merges the session data and your config’s path label.
    pub fn build_template_data(&self) -> serde_json::Value {
        let mut data = serde_json::json!({
            "absolute_code_path": label(&self.config.path),
            "source_tree": self.data.source_tree,
            "files": self.data.files,
            "git_diff": self.data.git_diff,
            "git_diff_branch": self.data.git_diff_branch,
            "git_log_branch": self.data.git_log_branch
        });

        // Add user-defined variables to the template data
        if !self.config.user_variables.is_empty()
            && let Some(obj) = data.as_object_mut()
        {
            for (key, value) in &self.config.user_variables {
                obj.insert(key.clone(), serde_json::Value::String(value.clone()));
            }
        }

        data
    }

    /// Renders the final prompt given a template-data JSON object. Returns both
    /// the rendered prompt and the token count information. The session
    /// does not do any printing or user prompting — that’s up to the caller.
    pub fn render_prompt(&self, template_data: &serde_json::Value) -> Result<RenderedPrompt> {
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
        let template_content = render_template(&handlebars, &template_name, template_data)?;

        // ~~~ Informations ~~~
        let tokenizer_type: TokenizerType = self.config.encoding;
        let token_count = count_tokens(&template_content, &tokenizer_type);
        let model_info = tokenizer_type.description();
        let directory_name = label(&self.config.path);
        let files: Vec<String> = self
            .data
            .files
            .as_ref()
            .and_then(|files_json| files_json.as_array())
            .map(|files| {
                files
                    .iter()
                    .filter_map(|file| {
                        file.get("path")
                            .and_then(|p| p.as_str())
                            .map(|s| s.to_string())
                    })
                    .collect()
            })
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
