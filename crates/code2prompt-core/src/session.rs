//! This module defines a Code2promptSession struct that provide a stateful interface to code2prompt-core.
//! It allows you to load codebase data, Git info, and render prompts using a template.

use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::configuration::Code2PromptConfig;
use crate::git::{get_git_diff, get_git_diff_between_branches, get_git_log};
use crate::path::{label, traverse_directory};
use crate::template::{handlebars_setup, render_template, OutputFormat};
use crate::tokenizer::{count_tokens, TokenizerType};

/// Represents a live session that holds stateful data about the user's codebase,
/// including which files have been added or removed, or other data that evolves over time.
#[derive(Clone)]
pub struct Code2PromptSession {
    pub config: Code2PromptConfig,
    pub selected_files: Vec<PathBuf>,
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
    /// Creates a new session that can track additional state if needed.
    pub fn new(config: Code2PromptConfig) -> Self {
        Self {
            config,
            selected_files: Vec::new(),
            data: SessionData::default(),
        }
    }

    #[allow(clippy::unused_self)]
    /// Allows adding a file to the included set manually.
    pub fn select_file(&mut self, file_path: PathBuf) {
        self.selected_files.push(file_path);
    }

    #[allow(clippy::unused_self)]
    /// Allows removing a file from the included set manually.
    pub fn deselect_file(&mut self, file_path: &PathBuf) {
        self.selected_files.retain(|f| f != file_path);
    }

    /// Loads the codebase data (source tree and file list) into the session.
    pub fn load_codebase(&mut self) -> Result<()> {
        let (tree, files_json) =
            traverse_directory(&self.config).with_context(|| "Failed to traverse directory")?;

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
        if self.config.user_variables.len() > 0 {
            if let Some(obj) = data.as_object_mut() {
                for (key, value) in &self.config.user_variables {
                    obj.insert(key.clone(), serde_json::Value::String(value.clone()));
                }
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
            directory_name: directory_name,
            token_count,
            model_info,
            files: files,
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
