//! Formatting functions for display purposes.
//!
//! This module contains pure functions that format data for display in the TUI.
//! These functions were previously scattered in Model and widgets.

use code2prompt_core::tokenizer::TokenFormat;
use num_format::{SystemLocale, ToFormattedString};
use std::collections::HashMap;

use crate::model::{SettingType, SettingsGroup, SettingsItem};
use crate::token_map::TokenMapEntry;

/// Format a number according to the specified token format
#[allow(dead_code)] // Will be used when widgets are refactored
pub fn format_number(num: usize, token_format: &TokenFormat) -> String {
    match token_format {
        TokenFormat::Format => SystemLocale::default()
            .map(|locale| num.to_formatted_string(&locale))
            .unwrap_or_else(|_| num.to_string()),
        TokenFormat::Raw => num.to_string(),
    }
}

/// Aggregate tokens by file extension from token map entries
#[allow(dead_code)] // Will be used when widgets are refactored
pub fn aggregate_tokens_by_extension(
    token_map_entries: &[TokenMapEntry],
) -> Vec<(String, usize, usize)> {
    let mut extension_stats: HashMap<String, (usize, usize)> = HashMap::new();

    for entry in token_map_entries {
        if !entry.metadata.is_dir {
            let extension = entry
                .name
                .split('.')
                .next_back()
                .map(|ext| format!(".{}", ext))
                .unwrap_or_else(|| "(no extension)".to_string());

            let (tokens, count) = extension_stats.entry(extension).or_insert((0, 0));
            *tokens += entry.tokens;
            *count += 1;
        }
    }

    // Convert to sorted vec by tokens (descending)
    let mut ext_vec: Vec<(String, usize, usize)> = extension_stats
        .into_iter()
        .map(|(ext, (tokens, count))| (ext, tokens, count))
        .collect();
    ext_vec.sort_by(|a, b| b.1.cmp(&a.1));

    ext_vec
}

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

/// Format extension statistics for display with dynamic column widths
#[allow(dead_code)] // Will be used when widgets are refactored
pub fn format_extension_statistics(
    extension_stats: &[(String, usize, usize)],
    total_tokens: usize,
    token_format: &TokenFormat,
    available_width: usize,
) -> (Vec<String>, String) {
    if extension_stats.is_empty() {
        return (Vec::new(), String::new());
    }

    // Calculate maximum widths needed for each column
    let max_ext_width = extension_stats
        .iter()
        .map(|(ext, _, _)| ext.len())
        .max()
        .unwrap_or(12)
        .max(12); // Minimum 12 chars for "Extension"

    let max_tokens_width = extension_stats
        .iter()
        .map(|(_, tokens, _)| format_number(*tokens, token_format).len())
        .max()
        .unwrap_or(6)
        .max(6); // Minimum 6 chars for tokens

    let max_count_width = extension_stats
        .iter()
        .map(|(_, _, count)| count.to_string().len())
        .max()
        .unwrap_or(3)
        .max(3); // Minimum 3 chars for count

    // Fixed widths for percentage and separators
    let percentage_width = 7; // "(100.0%)"
    let separators_width = 8; // " │ │ " + " | " + " files"

    // Calculate remaining space for the progress bar
    let fixed_content_width = max_ext_width
        + max_tokens_width
        + percentage_width
        + max_count_width
        + separators_width
        + 5; // +5 for "files"
    let bar_width = if available_width > fixed_content_width {
        (available_width - fixed_content_width).clamp(10, 40) // Between 10 and 40 chars
    } else {
        15 // Fallback minimum bar width
    };

    // Create header
    let header = format!(
        "{:<width_ext$} │{:^width_bar$}│ {:>width_tokens$} {:>7} | {:>width_count$} Files",
        "Extension",
        "Usage",
        "Tokens",
        "Percent",
        "Count",
        width_ext = max_ext_width,
        width_bar = bar_width,
        width_tokens = max_tokens_width,
        width_count = max_count_width
    );

    // Create formatted lines
    let lines: Vec<String> = extension_stats
        .iter()
        .map(|(extension, tokens, count)| {
            let percentage = if total_tokens > 0 {
                (*tokens as f64 / total_tokens as f64) * 100.0
            } else {
                0.0
            };

            // Create visual bar with calculated width
            let filled_chars = ((percentage / 100.0) * bar_width as f64) as usize;
            let bar = format!(
                "{}{}",
                "█".repeat(filled_chars),
                "░".repeat(bar_width.saturating_sub(filled_chars))
            );

            // Format with dynamic column widths
            let formatted_tokens = format_number(*tokens, token_format);
            format!(
                "{:<width_ext$} │{}│ {:>width_tokens$} ({:>4.1}%) | {:>width_count$} files",
                extension,
                bar,
                formatted_tokens,
                percentage,
                count,
                width_ext = max_ext_width,
                width_tokens = max_tokens_width,
                width_count = max_count_width
            )
        })
        .collect();

    (lines, header)
}
