//! This module defines the `Code2PromptSession` struct, which manages stateful operations on top of
//! a `Code2PromptConfig`. In a purely stateless CLI usage, a session may not be strictly necessary,
//! but for dynamic or incremental use cases (e.g., LSP, interactive agents), it provides a place to
//! store evolving state, such as which files have been added or removed, or other data that evolves over time.

use anyhow::{Context, Error, Result};
use std::path::PathBuf;

use crate::configuration::Code2PromptConfig;
use crate::git::{get_git_diff, get_git_diff_between_branches, get_git_log};
use crate::path::{label, traverse_directory};
use crate::template::{handlebars_setup, render_template};
use crate::tokenizer::{count_tokens, TokenFormat};
use num_format::{SystemLocale, ToFormattedString};

/// Represents a live session that holds stateful data about the user's codebase,
/// including which files have been added or removed, or other data that evolves over time.
pub struct Code2PromptSession {
    pub config: Code2PromptConfig,
    selected_files: Vec<PathBuf>,
}

impl Code2PromptSession {
    /// Creates a new session that can track additional state if needed.
    pub fn new(config: Code2PromptConfig) -> Self {
        Self {
            config,
            selected_files: Vec::new(),
        }
    }

    /// Allows adding a file to the included set manually.
    // pub fn select_file(&mut self, file_path: PathBuf) {
    //     self.selected_files.push(file_path);
    // }

    /// Allows removing a file from the included set manually.
    // pub fn deselect_file(&mut self, file_path: &PathBuf) {
    //     self.selected_files.retain(|f| f != file_path);
    // }

    pub fn git_diff(&self) -> Result<Option<String>, Error> {
        if self.config.diff_enabled {
            get_git_diff(&self.config.path).map(Some)
        } else {
            Ok(None)
        }
    }

    pub fn git_diff_between_branches(&self) -> Result<Option<String>, Error> {
        if let Some((b1, b2)) = &self.config.diff_branches {
            get_git_diff_between_branches(&self.config.path, b1, b2).map(Some)
        } else {
            Ok(None)
        }
    }

    pub fn git_log_between_branches(&self) -> Result<Option<String>, Error> {
        if let Some((b1, b2)) = &self.config.log_branches {
            get_git_log(&self.config.path, b1, b2).map(Some)
        } else {
            Ok(None)
        }
    }

    /// Generates the final prompt
    /// This method is the main entry point for generating the prompt.
    /// It traverses the directory, fetches git info, and renders the template.
    /// and returns the final prompt as a string.
    pub fn generate_prompt(&self) -> Result<String> {
        // ~~~ Traverse the directory ~~~
        let (tree, files_json) = traverse_directory(
            &self.config.path,
            &self.config.include_patterns,
            &self.config.exclude_patterns,
            self.config.include_priority,
            self.config.line_numbers,
            self.config.relative_paths,
            self.config.full_directory_tree,
            self.config.no_codeblock,
            self.config.follow_symlinks,
            self.config.hidden,
            self.config.no_ignore,
            self.config.sort_method,
        )
        .with_context(|| "Failed to traverse directory")?;

        // ~~~ Git Informations ~~~
        let git_diff = if self.config.diff_enabled {
            Some(get_git_diff(&self.config.path).unwrap_or_default())
        } else {
            None
        };

        let git_diff_branch = if let Some((b1, b2)) = &self.config.diff_branches {
            Some(get_git_diff_between_branches(&self.config.path, b1, b2).unwrap_or_default())
        } else {
            None
        };

        let git_log_branch = if let Some((b1, b2)) = &self.config.log_branches {
            Some(get_git_log(&self.config.path, b1, b2).unwrap_or_default())
        } else {
            None
        };

        // ~~~ Template ~~~
        // Prepare the data
        let template_data = serde_json::json!({
            "absolute_code_path": label(&self.config.path),
            "source_tree": tree,
            "files": files_json,
            "git_diff": git_diff,
            "git_diff_branch": git_diff_branch,
            "git_log_branch": git_log_branch,
        });

        // Render the template
        let (template_content, template_name) = match &self.config.custom_template {
            Some(tpl) => (tpl.clone(), "custom".to_string()),
            None => match self.config.output_format {
                crate::template::OutputFormat::Markdown | crate::template::OutputFormat::Json => (
                    include_str!("../../default_template_md.hbs").to_string(),
                    "default_md".into(),
                ),
                crate::template::OutputFormat::Xml => (
                    include_str!("../../default_template_xml.hbs").to_string(),
                    "default_xml".into(),
                ),
            },
        };

        let handlebars = handlebars_setup(&template_content, &template_name)?;
        let rendered = render_template(&handlebars, &template_name, &template_data)?;

        // ~~~ Token Count ~~~
        let tokenizer_type = &self.config.encoding;
        let token_count = count_tokens(&rendered, &tokenizer_type);
        let formatted_token_count: String = match &self.config.token_format {
            TokenFormat::Raw => token_count.to_string(),
            TokenFormat::Format => {
                token_count.to_formatted_string(&SystemLocale::default().unwrap())
            }
        };
        let model_info = tokenizer_type.description();
        println!("tokens: {}", formatted_token_count);
        println!("model: {}", model_info);

        Ok(rendered)
    }
}
