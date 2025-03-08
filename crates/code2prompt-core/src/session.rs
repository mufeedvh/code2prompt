//! This module defines a Code2promptSession struct that provide a stateful interface to code2prompt-core.
//! It allows you to load codebase data, Git info, and render prompts using a template.

use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::configuration::Code2PromptConfig;
use crate::git::{get_git_diff, get_git_diff_between_branches, get_git_log};
use crate::path::{label, traverse_directory};
use crate::template::{handlebars_setup, render_template};
use crate::tokenizer::{count_tokens, TokenizerType};

/// Represents a live session that holds stateful data about the user's codebase,
/// including which files have been added or removed, or other data that evolves over time.
pub struct Code2PromptSession {
    pub config: Code2PromptConfig,
    pub selected_files: Vec<PathBuf>,
    pub data: SessionData,
}

/// Represents the collected data about the code (tree + files) and optional Git info.
/// The session loads these pieces separately, so you can manage them step by step.
#[derive(Debug, Default)]
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
        serde_json::json!({
            "absolute_code_path": label(&self.config.path),
            "source_tree": self.data.source_tree,
            "files": self.data.files,
            "git_diff": self.data.git_diff,
            "git_diff_branch": self.data.git_diff_branch,
            "git_log_branch": self.data.git_log_branch
        })
    }

    /// Renders the final prompt given a template-data JSON object. Returns both
    /// the rendered prompt and the token count information. The session
    /// does not do any printing or user prompting — that’s up to the caller.
    pub fn render_prompt(&self, template_data: &serde_json::Value) -> Result<RenderedPrompt> {
        // ~~~ Rendering ~~~
        let handlebars = handlebars_setup(&self.config.template_str, &self.config.template_name)?;
        let rendered_prompt =
            render_template(&handlebars, &self.config.template_name, template_data)?;

        // ~~~ Informations ~~~
        let tokenizer_type: TokenizerType = self.config.encoding;
        let token_count = count_tokens(&rendered_prompt, &tokenizer_type);
        let model_info = tokenizer_type.description();
        let directory_name = label(&self.config.path);

        Ok(RenderedPrompt {
            prompt: rendered_prompt,
            directory_name: directory_name,
            token_count,
            model_info,
            files: self
                .data
                .files
                .as_ref()
                .unwrap()
                .as_array()
                .unwrap()
                .iter()
                .map(|f| f.as_str().unwrap().to_string())
                .collect(),
        })
    }

    pub fn generate_prompt(&mut self) -> Result<RenderedPrompt> {
        self.load_codebase()?;
        self.load_git_diff()?;
        self.load_git_diff_between_branches()?;
        self.load_git_log_between_branches()?;
        let template_data = self.build_template_data();
        let rendered = self.render_prompt(&template_data)?;
        Ok(rendered)
    }
}
