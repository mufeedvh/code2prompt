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

impl SettingsState {
    /// Get grouped settings for display
    #[allow(dead_code)] // Will be used when widgets are refactored
    pub fn get_settings_groups(&self, session: &Code2PromptSession) -> Vec<SettingsGroup> {
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

    /// Get flattened list of settings for display (keeping for backward compatibility)
    pub fn get_settings_items(&self, session: &Code2PromptSession) -> Vec<SettingsItem> {
        vec![
            // Output Format section
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
            // Sorting & Organization section
            SettingsItem {
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
            },
            // Tokenizer & Encoding section
            SettingsItem {
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
            },
            // Git Integration section
            SettingsItem {
                name: "Git Diff".to_string(),
                description: "Include git diff in output".to_string(),
                setting_type: SettingType::Boolean(session.config.diff_enabled),
            },
            // File Selection section
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
        ]
    }

    /// Update setting based on index and action (works with grouped settings)
    pub fn update_setting(
        &self,
        session: &mut Code2PromptSession,
        index: usize,
        action: SettingAction,
    ) {
        // Map flat index to actual setting based on grouped structure
        match index {
            // Output Format section (0-5)
            0 => session.config.line_numbers = !session.config.line_numbers, // Line Numbers
            1 => session.config.absolute_path = !session.config.absolute_path, // Absolute Paths
            2 => session.config.no_codeblock = !session.config.no_codeblock, // No Codeblock
            3 => {
                // Output Format
                if let SettingAction::Cycle = action {
                    session.config.output_format = match session.config.output_format {
                        OutputFormat::Markdown => OutputFormat::Json,
                        OutputFormat::Json => OutputFormat::Xml,
                        OutputFormat::Xml => OutputFormat::Markdown,
                    };
                }
            }
            4 => {
                // Token Format
                if let SettingAction::Cycle = action {
                    session.config.token_format = match session.config.token_format {
                        TokenFormat::Raw => TokenFormat::Format,
                        TokenFormat::Format => TokenFormat::Raw,
                    };
                }
            }
            5 => session.config.full_directory_tree = !session.config.full_directory_tree, // Full Directory Tree

            // Sorting & Organization section (6)
            6 => {
                // Sort Method
                if let SettingAction::Cycle = action {
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
                }
            }

            // Tokenizer & Encoding section
            7 => {
                // Tokenizer Type
                if let SettingAction::Cycle = action {
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
                }
            }

            // Git Integration section (8)
            8 => session.config.diff_enabled = !session.config.diff_enabled, // Git Diff

            // File Selection section (9-11)
            9 => session.config.follow_symlinks = !session.config.follow_symlinks, // Follow Symlinks
            10 => session.config.hidden = !session.config.hidden,                  // Hidden Files
            11 => session.config.no_ignore = !session.config.no_ignore,            // No Ignore

            _ => {}
        }
    }
}
