//! Formatting functions for display purposes.
//!
//! This module contains pure functions that format data for display in the TUI.
//! These functions were previously scattered in Model and widgets.

use code2prompt_core::tokenizer::TokenFormat;

use crate::model::{SettingType, SettingsGroup, SettingsItem};

/// Format settings groups for display
pub fn format_settings_groups(
    session: &code2prompt_core::session::Code2PromptSession,
) -> Vec<SettingsGroup> {
    use code2prompt_core::template::OutputFormat;

    vec![
        SettingsGroup {
            name: "Output Format".to_string(),
            items: vec![
                SettingsItem {
                    name: "Line Numbers".to_string(),
                    description: "Show line numbers in output".to_string(),
                    setting_type: SettingType::Boolean(session.config.line_numbers),
                },
                SettingsItem {
                    name: "Absolute Paths".to_string(),
                    description: "Use absolute instead of relative paths".to_string(),
                    setting_type: SettingType::Boolean(session.config.absolute_path),
                },
                SettingsItem {
                    name: "No Codeblock".to_string(),
                    description: "Don't wrap code in markdown blocks".to_string(),
                    setting_type: SettingType::Boolean(session.config.no_codeblock),
                },
                SettingsItem {
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
                    name: "Token Format".to_string(),
                    description: "How to display token counts".to_string(),
                    setting_type: SettingType::Choice {
                        options: vec!["Raw".to_string(), "Formatted".to_string()],
                        selected: match session.config.token_format {
                            TokenFormat::Raw => 0,
                            TokenFormat::Format => 1,
                        },
                    },
                },
                SettingsItem {
                    name: "Full Directory Tree".to_string(),
                    description: "Show complete directory structure".to_string(),
                    setting_type: SettingType::Boolean(session.config.full_directory_tree),
                },
            ],
        },
        SettingsGroup {
            name: "Sorting & Organization".to_string(),
            items: vec![SettingsItem {
                name: "Sort Method".to_string(),
                description: "How to sort files in output".to_string(),
                setting_type: SettingType::Choice {
                    options: vec![
                        "Name (A→Z)".to_string(),
                        "Name (Z→A)".to_string(),
                        "Date (Old→New)".to_string(),
                        "Date (New→Old)".to_string(),
                    ],
                    selected: match session.config.sort_method {
                        Some(code2prompt_core::sort::FileSortMethod::NameAsc) => 0,
                        Some(code2prompt_core::sort::FileSortMethod::NameDesc) => 1,
                        Some(code2prompt_core::sort::FileSortMethod::DateAsc) => 2,
                        Some(code2prompt_core::sort::FileSortMethod::DateDesc) => 3,
                        None => 0,
                    },
                },
            }],
        },
        SettingsGroup {
            name: "Tokenizer & Encoding".to_string(),
            items: vec![SettingsItem {
                name: "Tokenizer Type".to_string(),
                description: "Encoding method for token counting".to_string(),
                setting_type: SettingType::Choice {
                    options: vec![
                        "cl100k (ChatGPT)".to_string(),
                        "o200k (GPT-4o)".to_string(),
                        "p50k (Code models)".to_string(),
                        "p50k_edit (Edit models)".to_string(),
                        "r50k (GPT-3)".to_string(),
                    ],
                    selected: match session.config.encoding {
                        code2prompt_core::tokenizer::TokenizerType::Cl100kBase => 0,
                        code2prompt_core::tokenizer::TokenizerType::O200kBase => 1,
                        code2prompt_core::tokenizer::TokenizerType::P50kBase => 2,
                        code2prompt_core::tokenizer::TokenizerType::P50kEdit => 3,
                        code2prompt_core::tokenizer::TokenizerType::R50kBase
                        | code2prompt_core::tokenizer::TokenizerType::Gpt2 => 4,
                    },
                },
            }],
        },
        SettingsGroup {
            name: "Git Integration".to_string(),
            items: vec![SettingsItem {
                name: "Git Diff".to_string(),
                description: "Include git diff in output".to_string(),
                setting_type: SettingType::Boolean(session.config.diff_enabled),
            }],
        },
        SettingsGroup {
            name: "File Selection".to_string(),
            items: vec![
                SettingsItem {
                    name: "Follow Symlinks".to_string(),
                    description: "Follow symbolic links".to_string(),
                    setting_type: SettingType::Boolean(session.config.follow_symlinks),
                },
                SettingsItem {
                    name: "Hidden Files".to_string(),
                    description: "Include hidden files and directories".to_string(),
                    setting_type: SettingType::Boolean(session.config.hidden),
                },
                SettingsItem {
                    name: "No Ignore".to_string(),
                    description: "Ignore .gitignore rules".to_string(),
                    setting_type: SettingType::Boolean(session.config.no_ignore),
                },
            ],
        },
    ]
}
