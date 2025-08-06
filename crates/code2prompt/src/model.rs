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

    // Prompt output state (Tab 4)
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

/// The four main tabs of the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    FileTree,     // Tab 1: File selection with search
    Settings,     // Tab 2: Configuration options
    Statistics,   // Tab 3: Analysis statistics and metrics
    PromptOutput, // Tab 4: Generated prompt and copy
}

/// Different views available in the Statistics tab
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatisticsView {
    Overview,   // General statistics and summary
    TokenMap,   // Token distribution by directory/file
    Extensions, // Token distribution by file extension
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

}

impl Default for Model {
    fn default() -> Self {
        let config = code2prompt_core::configuration::Code2PromptConfig::default();
        let session = Code2PromptSession::new(config);

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
        }
    }
}

impl Model {

    pub fn new_with_cli_args(
        path: PathBuf,
        include_patterns: Vec<String>,
        exclude_patterns: Vec<String>,
    ) -> Self {
        let mut config = code2prompt_core::configuration::Code2PromptConfig::default();
        config.path = path;
        config.include_patterns = include_patterns;
        config.exclude_patterns = exclude_patterns;
        // Enable token map for TUI
        config.token_map_enabled = true;
        
        let session = Code2PromptSession::new(config);
        
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
                ],
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
            // Format section
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
            // Tokenizer section
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
            // Git section
            SettingsItem {
                name: "Git Diff".to_string(),
                description: "Include git diff in output".to_string(),
                setting_type: SettingType::Boolean(self.session.config.diff_enabled),
            },
            // File selection section
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
            5 => self.session.config.diff_enabled = !self.session.config.diff_enabled, // Git Diff
            6 => self.session.config.follow_symlinks = !self.session.config.follow_symlinks, // Follow Symlinks
            7 => self.session.config.hidden = !self.session.config.hidden,             // Hidden Files
            8 => self.session.config.no_ignore = !self.session.config.no_ignore,       // No Ignore
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
