//! Data structures and application state management for the TUI.
//!
//! This module contains the core data structures that represent the application state,
//! including the main Model struct, tab definitions, message types for event handling,
//! file tree nodes, and settings management. It serves as the central state container
//! for the terminal user interface.

use code2prompt_core::session::Code2PromptSession;
use code2prompt_core::template::OutputFormat;
use code2prompt_core::tokenizer::TokenFormat;
use std::path::PathBuf;

/// Represents the overall state of the TUI application.
#[derive(Debug, Clone)]
pub struct Model {
    pub session: Code2PromptSession,
    pub current_tab: Tab,
    pub should_quit: bool,

    // File tree state (Tab 1)
    pub file_tree: Vec<FileNode>,
    pub search_query: String,
    pub tree_cursor: usize,

    // Settings state (Tab 2)
    pub settings_cursor: usize,

    // Statistics state (Tab 3)
    pub statistics_view: StatisticsView,
    pub statistics_scroll: u16,

    // Template state (Tab 4)
    pub template_content: String,
    pub template_name: String,
    pub template_is_editing: bool,
    pub template_scroll_offset: u16,
    pub template_cursor_position: usize,

    // Prompt output state (Tab 5)
    pub generated_prompt: Option<String>,
    pub token_count: Option<usize>,
    pub file_count: usize,
    pub analysis_in_progress: bool,
    pub analysis_error: Option<String>,
    pub output_scroll: u16,
    pub file_tree_scroll: u16,

    // Token Map data
    pub token_map_entries: Vec<crate::token_map::TokenMapEntry>,

    // Status messages
    pub status_message: String,
}

/// The five main tabs of the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    FileTree,     // Tab 1: File selection with search
    Settings,     // Tab 2: Configuration options
    Statistics,   // Tab 3: Analysis statistics and metrics
    Template,     // Tab 4: Template editor
    PromptOutput, // Tab 5: Generated prompt and copy
}

/// Different views available in the Statistics tab
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatisticsView {
    Overview,   // General statistics and summary
    TokenMap,   // Token distribution by directory/file
    Extensions, // Token distribution by file extension
}

impl StatisticsView {
    pub fn next(&self) -> Self {
        match self {
            StatisticsView::Overview => StatisticsView::TokenMap,
            StatisticsView::TokenMap => StatisticsView::Extensions,
            StatisticsView::Extensions => StatisticsView::Overview,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            StatisticsView::Overview => StatisticsView::Extensions,
            StatisticsView::TokenMap => StatisticsView::Overview,
            StatisticsView::Extensions => StatisticsView::TokenMap,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            StatisticsView::Overview => "Overview",
            StatisticsView::TokenMap => "Token Map",
            StatisticsView::Extensions => "Extensions",
        }
    }
}

/// File tree node with selection state
#[derive(Debug, Clone)]
pub struct FileNode {
    pub path: PathBuf,
    pub name: String,
    pub is_directory: bool,
    pub is_expanded: bool,
    pub is_selected: bool,
    pub children: Vec<FileNode>,
    pub level: usize,
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

/// Messages for updating the model
#[derive(Debug, Clone)]
pub enum Message {
    // Navigation
    SwitchTab(Tab),
    Quit,

    // File tree
    UpdateSearchQuery(String),
    ToggleFileSelection(usize),
    ExpandDirectory(usize),
    CollapseDirectory(usize),
    MoveTreeCursor(i32),
    RefreshFileTree,

    // Search mode
    EnterSearchMode,
    ExitSearchMode,

    // Settings
    MoveSettingsCursor(i32),
    ToggleSetting(usize),
    CycleSetting(usize),

    // Analysis
    RunAnalysis,
    AnalysisComplete(AnalysisResults), // Complete analysis results
    AnalysisError(String),

    // Prompt output
    CopyToClipboard,
    SaveToFile(String),
    ScrollOutput(i16),   // Scroll delta (positive = down, negative = up)
    ScrollFileTree(i16), // Scroll delta for file tree

    // Statistics
    CycleStatisticsView(i8), // +1 = next view, -1 = previous view
    ScrollStatistics(i16),   // Scroll delta for statistics

    // Template
    ToggleTemplateEdit,
    UpdateTemplateContent(String),
    LoadTemplateFromFile(String),
    SaveTemplateToFile(String),
    ResetTemplateToDefault,
    ScrollTemplate(i16),
}

impl Default for Model {
    fn default() -> Self {
        let config = code2prompt_core::configuration::Code2PromptConfig::default();
        let session = Code2PromptSession::new(config);

        // Load default template based on output format
        let template_content = match session.config.output_format {
            OutputFormat::Xml => {
                include_str!("../../code2prompt-core/src/default_template_xml.hbs").to_string()
            }
            _ => include_str!("../../code2prompt-core/src/default_template_md.hbs").to_string(),
        };

        Model {
            session,
            current_tab: Tab::FileTree,
            should_quit: false,
            file_tree: Vec::new(),
            search_query: String::new(),
            tree_cursor: 0,
            settings_cursor: 0,
            generated_prompt: None,
            token_count: None,
            file_count: 0,
            analysis_in_progress: false,
            analysis_error: None,
            output_scroll: 0,
            file_tree_scroll: 0,
            token_map_entries: Vec::new(),
            statistics_view: StatisticsView::Overview,
            statistics_scroll: 0,
            status_message: String::new(),
            template_content,
            template_name: "Default Template".to_string(),
            template_is_editing: false,
            template_scroll_offset: 0,
            template_cursor_position: 0,
        }
    }
}

impl Model {
    pub fn new_with_cli_args(session: Code2PromptSession) -> Self {
        // Load default template based on output format
        let template_content = match session.config.output_format {
            OutputFormat::Xml => {
                include_str!("../../code2prompt-core/src/default_template_xml.hbs").to_string()
            }
            _ => include_str!("../../code2prompt-core/src/default_template_md.hbs").to_string(),
        };

        Model {
            session,
            current_tab: Tab::FileTree,
            should_quit: false,
            file_tree: Vec::new(),
            search_query: String::new(),
            tree_cursor: 0,
            settings_cursor: 0,
            generated_prompt: None,
            token_count: None,
            file_count: 0,
            analysis_in_progress: false,
            analysis_error: None,
            output_scroll: 0,
            file_tree_scroll: 0,
            token_map_entries: Vec::new(),
            statistics_view: StatisticsView::Overview,
            statistics_scroll: 0,
            status_message: String::new(),
            template_content,
            template_name: "Default Template".to_string(),
            template_is_editing: false,
            template_scroll_offset: 0,
            template_cursor_position: 0,
        }
    }

    /// Get flattened list of visible file nodes for display
    pub fn get_visible_nodes(&self) -> Vec<&FileNode> {
        let mut visible = Vec::new();
        self.collect_visible_nodes(&self.file_tree, &mut visible);
        visible
    }

    fn collect_visible_nodes<'a>(&'a self, nodes: &'a [FileNode], visible: &mut Vec<&'a FileNode>) {
        for node in nodes {
            // Apply search filter - support both simple text and glob patterns
            let matches_search = if self.search_query.is_empty() {
                true
            } else if self.search_query.contains('*') || self.search_query.contains("**") {
                // Treat as glob pattern
                self.glob_match_search(&self.search_query, &node.name)
                    || self.glob_match_search(&self.search_query, &node.path.to_string_lossy())
            } else {
                // Simple text search (case insensitive)
                node.name
                    .to_lowercase()
                    .contains(&self.search_query.to_lowercase())
                    || node
                        .path
                        .to_string_lossy()
                        .to_lowercase()
                        .contains(&self.search_query.to_lowercase())
            };

            if matches_search {
                visible.push(node);
            }

            // Add children if expanded and node matches search or has matching children
            if node.is_expanded && (matches_search || node.is_directory) {
                self.collect_visible_nodes(&node.children, visible);
            }
        }
    }

    /// Simple glob matching for search (similar to utils but accessible from model)
    fn glob_match_search(&self, pattern: &str, text: &str) -> bool {
        // Handle ** for recursive directory matching
        if pattern.contains("**") {
            let parts: Vec<&str> = pattern.split("**").collect();
            if parts.len() == 2 {
                let prefix = parts[0].trim_end_matches('/');
                let suffix = parts[1].trim_start_matches('/');

                if prefix.is_empty() && suffix.is_empty() {
                    return true; // "**" matches everything
                }

                let prefix_match = prefix.is_empty() || text.contains(prefix);
                let suffix_match = suffix.is_empty() || text.contains(suffix);

                return prefix_match && suffix_match;
            }
        }

        // Handle single * wildcard
        if pattern.contains('*') && !pattern.contains("**") {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                return text.contains(parts[0]) && text.contains(parts[1]);
            }
        }

        // Fallback to contains
        text.to_lowercase().contains(&pattern.to_lowercase())
    }

    /// Get grouped settings for display
    pub fn get_settings_groups(&self) -> Vec<SettingsGroup> {
        vec![
            SettingsGroup {
                name: "Output Format".to_string(),
                items: vec![
                    SettingsItem {
                        name: "Line Numbers".to_string(),
                        description: "Show line numbers in output".to_string(),
                        setting_type: SettingType::Boolean(self.session.config.line_numbers),
                    },
                    SettingsItem {
                        name: "Absolute Paths".to_string(),
                        description: "Use absolute instead of relative paths".to_string(),
                        setting_type: SettingType::Boolean(self.session.config.absolute_path),
                    },
                    SettingsItem {
                        name: "No Codeblock".to_string(),
                        description: "Don't wrap code in markdown blocks".to_string(),
                        setting_type: SettingType::Boolean(self.session.config.no_codeblock),
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
                            selected: match self.session.config.output_format {
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
                            selected: match self.session.config.token_format {
                                TokenFormat::Raw => 0,
                                TokenFormat::Format => 1,
                            },
                        },
                    },
                    SettingsItem {
                        name: "Full Directory Tree".to_string(),
                        description: "Show complete directory structure".to_string(),
                        setting_type: SettingType::Boolean(self.session.config.full_directory_tree),
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
                        selected: match self.session.config.sort_method {
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
                        selected: match self.session.config.encoding {
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
                    setting_type: SettingType::Boolean(self.session.config.diff_enabled),
                }],
            },
            SettingsGroup {
                name: "File Selection".to_string(),
                items: vec![
                    SettingsItem {
                        name: "Follow Symlinks".to_string(),
                        description: "Follow symbolic links".to_string(),
                        setting_type: SettingType::Boolean(self.session.config.follow_symlinks),
                    },
                    SettingsItem {
                        name: "Hidden Files".to_string(),
                        description: "Include hidden files and directories".to_string(),
                        setting_type: SettingType::Boolean(self.session.config.hidden),
                    },
                    SettingsItem {
                        name: "No Ignore".to_string(),
                        description: "Ignore .gitignore rules".to_string(),
                        setting_type: SettingType::Boolean(self.session.config.no_ignore),
                    },
                ],
            },
        ]
    }

    /// Get flattened list of settings for display (keeping for backward compatibility)
    pub fn get_settings_items(&self) -> Vec<SettingsItem> {
        vec![
            // Output Format section
            SettingsItem {
                name: "Line Numbers".to_string(),
                description: "Show line numbers in output".to_string(),
                setting_type: SettingType::Boolean(self.session.config.line_numbers),
            },
            SettingsItem {
                name: "Absolute Paths".to_string(),
                description: "Use absolute instead of relative paths".to_string(),
                setting_type: SettingType::Boolean(self.session.config.absolute_path),
            },
            SettingsItem {
                name: "No Codeblock".to_string(),
                description: "Don't wrap code in markdown blocks".to_string(),
                setting_type: SettingType::Boolean(self.session.config.no_codeblock),
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
                    selected: match self.session.config.output_format {
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
                    selected: match self.session.config.token_format {
                        TokenFormat::Raw => 0,
                        TokenFormat::Format => 1,
                    },
                },
            },
            SettingsItem {
                name: "Full Directory Tree".to_string(),
                description: "Show complete directory structure".to_string(),
                setting_type: SettingType::Boolean(self.session.config.full_directory_tree),
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
                    selected: match self.session.config.sort_method {
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
                    selected: match self.session.config.encoding {
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
                setting_type: SettingType::Boolean(self.session.config.diff_enabled),
            },
            // File Selection section
            SettingsItem {
                name: "Follow Symlinks".to_string(),
                description: "Follow symbolic links".to_string(),
                setting_type: SettingType::Boolean(self.session.config.follow_symlinks),
            },
            SettingsItem {
                name: "Hidden Files".to_string(),
                description: "Include hidden files and directories".to_string(),
                setting_type: SettingType::Boolean(self.session.config.hidden),
            },
            SettingsItem {
                name: "No Ignore".to_string(),
                description: "Ignore .gitignore rules".to_string(),
                setting_type: SettingType::Boolean(self.session.config.no_ignore),
            },
        ]
    }

    /// Update setting based on index and action (works with grouped settings)
    pub fn update_setting(&mut self, index: usize, action: SettingAction) {
        // Map flat index to actual setting based on grouped structure
        match index {
            // Output Format section (0-5)
            0 => self.session.config.line_numbers = !self.session.config.line_numbers, // Line Numbers
            1 => self.session.config.absolute_path = !self.session.config.absolute_path, // Absolute Paths
            2 => self.session.config.no_codeblock = !self.session.config.no_codeblock, // No Codeblock
            3 => {
                // Output Format
                if let SettingAction::Cycle = action {
                    self.session.config.output_format = match self.session.config.output_format {
                        OutputFormat::Markdown => OutputFormat::Json,
                        OutputFormat::Json => OutputFormat::Xml,
                        OutputFormat::Xml => OutputFormat::Markdown,
                    };
                }
            }
            4 => {
                // Token Format
                if let SettingAction::Cycle = action {
                    self.session.config.token_format = match self.session.config.token_format {
                        TokenFormat::Raw => TokenFormat::Format,
                        TokenFormat::Format => TokenFormat::Raw,
                    };
                }
            }
            5 => self.session.config.full_directory_tree = !self.session.config.full_directory_tree, // Full Directory Tree

            // Sorting & Organization section (6)
            6 => {
                // Sort Method
                if let SettingAction::Cycle = action {
                    self.session.config.sort_method = Some(match self.session.config.sort_method {
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
                    self.session.config.encoding = match self.session.config.encoding {
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

            // Git Integration section (9)
            8 => self.session.config.diff_enabled = !self.session.config.diff_enabled, // Git Diff

            // File Selection section (10-12)
            9 => self.session.config.follow_symlinks = !self.session.config.follow_symlinks, // Follow Symlinks
            10 => self.session.config.hidden = !self.session.config.hidden, // Hidden Files
            11 => self.session.config.no_ignore = !self.session.config.no_ignore, // No Ignore

            _ => {}
        }
    }
}

#[derive(Debug, Clone)]
pub enum SettingAction {
    Toggle,
    Cycle,
}

/// Results from code2prompt analysis
#[derive(Debug, Clone)]
pub struct AnalysisResults {
    pub file_count: usize,
    pub token_count: Option<usize>,
    pub generated_prompt: String,
    pub token_map_entries: Vec<crate::token_map::TokenMapEntry>,
}

impl FileNode {
    pub fn new(path: PathBuf, level: usize) -> Self {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        let is_directory = path.is_dir();

        Self {
            path,
            name,
            is_directory,
            is_expanded: false,
            is_selected: false,
            children: Vec::new(),
            level,
        }
    }
}
