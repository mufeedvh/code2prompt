//! Formatting functions for display purposes.
//!
//! This module contains pure functions that format data for display in the TUI.
//! These functions were previously scattered in Model and widgets.

use code2prompt_core::sort::FileSortMethod;
use code2prompt_core::template::OutputFormat;
use code2prompt_core::tokenizer::TokenFormat;
use code2prompt_core::{session::Code2PromptSession, tokenizer::TokenizerType};

use crate::model::{SettingKey, SettingType, SettingsGroup, SettingsItem};

/// Format settings groups for display
pub fn format_settings_groups(session: &Code2PromptSession) -> Vec<SettingsGroup> {
    vec![
        SettingsGroup {
            name: "Output Format".to_string(),
            items: vec![
                SettingsItem {
                    key: SettingKey::LineNumbers,
                    name: "Line Numbers".to_string(),
                    description: "Show line numbers in output".to_string(),
                    setting_type: SettingType::Boolean(session.config.line_numbers),
                },
                SettingsItem {
                    key: SettingKey::AbsolutePaths,
                    name: "Absolute Paths".to_string(),
                    description: "Use absolute instead of relative paths".to_string(),
                    setting_type: SettingType::Boolean(session.config.absolute_path),
                },
                SettingsItem {
                    key: SettingKey::NoCodeblock,
                    name: "No Codeblock".to_string(),
                    description: "Don't wrap code in markdown blocks".to_string(),
                    setting_type: SettingType::Boolean(session.config.no_codeblock),
                },
                SettingsItem {
                    key: SettingKey::OutputFormat,
                    name: "Output Format".to_string(),
                    description: "Format for generated output".to_string(),
                    setting_type: SettingType::Choice {
                        options: vec![
                            "Markdown".to_string(),
                            "JSON".to_string(),
                            "XML".to_string(),
                        ],
                        selected: match session.config.output_format {
                            OutputFormat::Markdown => 0,
                            OutputFormat::Json => 1,
                            OutputFormat::Xml => 2,
                        },
                    },
                },
                SettingsItem {
                    key: SettingKey::TokenFormat,
                    name: "Token Format".to_string(),
                    description: "How to display token counts".to_string(),
                    setting_type: SettingType::Choice {
                        options: vec![
                            TokenFormat::Raw.to_string(),
                            TokenFormat::Format.to_string(),
                        ],
                        selected: match session.config.token_format {
                            TokenFormat::Raw => 0,
                            TokenFormat::Format => 1,
                        },
                    },
                },
                SettingsItem {
                    key: SettingKey::FullDirectoryTree,
                    name: "Full Directory Tree".to_string(),
                    description: "Show complete directory structure".to_string(),
                    setting_type: SettingType::Boolean(session.config.full_directory_tree),
                },
            ],
        },
        SettingsGroup {
            name: "Sorting & Organization".to_string(),
            items: vec![SettingsItem {
                key: SettingKey::SortMethod,
                name: "Sort Method".to_string(),
                description: "How to sort files in output".to_string(),
                setting_type: SettingType::Choice {
                    options: vec![
                        FileSortMethod::NameAsc.to_string(),
                        FileSortMethod::NameDesc.to_string(),
                        FileSortMethod::DateAsc.to_string(),
                        FileSortMethod::DateDesc.to_string(),
                    ],
                    selected: match session.config.sort_method {
                        Some(FileSortMethod::NameAsc) => 0,
                        Some(FileSortMethod::NameDesc) => 1,
                        Some(FileSortMethod::DateAsc) => 2,
                        Some(FileSortMethod::DateDesc) => 3,
                        None => 0,
                    },
                },
            }],
        },
        SettingsGroup {
            name: "Tokenizer & Encoding".to_string(),
            items: vec![SettingsItem {
                key: SettingKey::TokenizerType,
                name: "Tokenizer Type".to_string(),
                description: "Encoding method for token counting".to_string(),
                setting_type: SettingType::Choice {
                    options: vec![
                        TokenizerType::Cl100kBase.to_string(),
                        TokenizerType::O200kBase.to_string(),
                        TokenizerType::P50kBase.to_string(),
                        TokenizerType::P50kEdit.to_string(),
                        TokenizerType::R50kBase.to_string(),
                    ],
                    selected: match session.config.encoding {
                        TokenizerType::Cl100kBase => 0,
                        TokenizerType::O200kBase => 1,
                        TokenizerType::P50kBase => 2,
                        TokenizerType::P50kEdit => 3,
                        TokenizerType::R50kBase | TokenizerType::Gpt2 => 4,
                    },
                },
            }],
        },
        SettingsGroup {
            name: "Git Integration".to_string(),
            items: vec![SettingsItem {
                key: SettingKey::GitDiff,
                name: "Git Diff".to_string(),
                description: "Include git diff in output".to_string(),
                setting_type: SettingType::Boolean(session.config.diff_enabled),
            }],
        },
        SettingsGroup {
            name: "File Selection".to_string(),
            items: vec![
                SettingsItem {
                    key: SettingKey::FollowSymlinks,
                    name: "Follow Symlinks".to_string(),
                    description: "Follow symbolic links".to_string(),
                    setting_type: SettingType::Boolean(session.config.follow_symlinks),
                },
                SettingsItem {
                    key: SettingKey::HiddenFiles,
                    name: "Hidden Files".to_string(),
                    description: "Include hidden files and directories".to_string(),
                    setting_type: SettingType::Boolean(session.config.hidden),
                },
                SettingsItem {
                    key: SettingKey::NoIgnore,
                    name: "No Ignore".to_string(),
                    description: "Ignore .gitignore rules".to_string(),
                    setting_type: SettingType::Boolean(session.config.no_ignore),
                },
            ],
        },
    ]
}
