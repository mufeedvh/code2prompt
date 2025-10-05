//! Settings state management for the TUI application.
//!
//! This module contains the settings state, settings groups, and related
//! functionality for managing configuration options in the TUI.

use code2prompt_core::session::Code2PromptSession;
use code2prompt_core::template::OutputFormat;
use code2prompt_core::tokenizer::TokenFormat;

/// Settings state containing cursor position and related data
#[derive(Default, Debug, Clone)]
pub struct SettingsState {
    pub settings_cursor: usize,
}

/// Settings group for organizing settings
#[derive(Debug, Clone)]
pub struct SettingsGroup {
    pub name: String,
    pub items: Vec<SettingsItem>,
}

/// Settings item for display and interaction
#[derive(Debug, Clone)]
pub struct SettingsItem {
    pub key: SettingKey,
    pub name: String,
    pub description: String,
    pub setting_type: SettingType,
}

#[derive(Debug, Clone)]
pub enum SettingType {
    Boolean(bool),
    Choice {
        options: Vec<String>,
        selected: usize,
    },
}

#[derive(Debug, Clone)]
pub enum SettingAction {
    Toggle,
    Cycle,
}

/// Unique identifier for each setting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SettingKey {
    LineNumbers,
    AbsolutePaths,
    NoCodeblock,
    OutputFormat,
    TokenFormat,
    FullDirectoryTree,
    SortMethod,
    TokenizerType,
    GitDiff,
    FollowSymlinks,
    HiddenFiles,
    NoIgnore,
}

impl SettingsState {
    /// Get flattened list of settings for display (uses format_settings_groups)
    pub fn get_settings_items(&self, session: &Code2PromptSession) -> Vec<SettingsItem> {
        crate::view::format_settings_groups(session)
            .into_iter()
            .flat_map(|group| group.items)
            .collect()
    }

    /// Update setting based on SettingKey and action
    pub fn update_setting_by_key(
        &self,
        session: &mut Code2PromptSession,
        key: SettingKey,
        action: SettingAction,
    ) -> &'static str {
        match (key, action) {
            (SettingKey::LineNumbers, SettingAction::Toggle | SettingAction::Cycle) => {
                session.config.line_numbers = !session.config.line_numbers;
                "Line Numbers"
            }
            (SettingKey::AbsolutePaths, SettingAction::Toggle | SettingAction::Cycle) => {
                session.config.absolute_path = !session.config.absolute_path;
                "Absolute Paths"
            }
            (SettingKey::NoCodeblock, SettingAction::Toggle | SettingAction::Cycle) => {
                session.config.no_codeblock = !session.config.no_codeblock;
                "No Codeblock"
            }
            (SettingKey::OutputFormat, SettingAction::Cycle) => {
                session.config.output_format = match session.config.output_format {
                    OutputFormat::Markdown => OutputFormat::Json,
                    OutputFormat::Json => OutputFormat::Xml,
                    OutputFormat::Xml => OutputFormat::Markdown,
                };
                "Output Format"
            }
            (SettingKey::TokenFormat, SettingAction::Cycle) => {
                session.config.token_format = match session.config.token_format {
                    TokenFormat::Raw => TokenFormat::Format,
                    TokenFormat::Format => TokenFormat::Raw,
                };
                "Token Format"
            }
            (SettingKey::FullDirectoryTree, SettingAction::Toggle | SettingAction::Cycle) => {
                session.config.full_directory_tree = !session.config.full_directory_tree;
                "Full Directory Tree"
            }
            (SettingKey::SortMethod, SettingAction::Cycle) => {
                session.config.sort_method = Some(match session.config.sort_method {
                    Some(code2prompt_core::sort::FileSortMethod::NameAsc) => {
                        code2prompt_core::sort::FileSortMethod::NameDesc
                    }
                    Some(code2prompt_core::sort::FileSortMethod::NameDesc) => {
                        code2prompt_core::sort::FileSortMethod::DateAsc
                    }
                    Some(code2prompt_core::sort::FileSortMethod::DateAsc) => {
                        code2prompt_core::sort::FileSortMethod::DateDesc
                    }
                    Some(code2prompt_core::sort::FileSortMethod::DateDesc) | None => {
                        code2prompt_core::sort::FileSortMethod::NameAsc
                    }
                });
                "Sort Method"
            }
            (SettingKey::TokenizerType, SettingAction::Cycle) => {
                session.config.encoding = match session.config.encoding {
                    code2prompt_core::tokenizer::TokenizerType::Cl100kBase => {
                        code2prompt_core::tokenizer::TokenizerType::O200kBase
                    }
                    code2prompt_core::tokenizer::TokenizerType::O200kBase => {
                        code2prompt_core::tokenizer::TokenizerType::P50kBase
                    }
                    code2prompt_core::tokenizer::TokenizerType::P50kBase => {
                        code2prompt_core::tokenizer::TokenizerType::P50kEdit
                    }
                    code2prompt_core::tokenizer::TokenizerType::P50kEdit => {
                        code2prompt_core::tokenizer::TokenizerType::R50kBase
                    }
                    code2prompt_core::tokenizer::TokenizerType::R50kBase
                    | code2prompt_core::tokenizer::TokenizerType::Gpt2 => {
                        code2prompt_core::tokenizer::TokenizerType::Cl100kBase
                    }
                };
                "Tokenizer Type"
            }
            (SettingKey::GitDiff, SettingAction::Toggle | SettingAction::Cycle) => {
                session.config.diff_enabled = !session.config.diff_enabled;
                "Git Diff"
            }
            (SettingKey::FollowSymlinks, SettingAction::Toggle | SettingAction::Cycle) => {
                session.config.follow_symlinks = !session.config.follow_symlinks;
                "Follow Symlinks"
            }
            (SettingKey::HiddenFiles, SettingAction::Toggle | SettingAction::Cycle) => {
                session.config.hidden = !session.config.hidden;
                "Hidden Files"
            }
            (SettingKey::NoIgnore, SettingAction::Toggle | SettingAction::Cycle) => {
                session.config.no_ignore = !session.config.no_ignore;
                "No Ignore"
            }
            _ => "Unknown Setting",
        }
    }
}
