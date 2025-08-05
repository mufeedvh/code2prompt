use code2prompt_core::configuration::Code2PromptConfig;
use code2prompt_core::template::OutputFormat;
use code2prompt_core::tokenizer::TokenFormat;
use std::collections::HashMap;
use std::path::PathBuf;

/// Represents the overall state of the TUI application.
#[derive(Debug, Clone)]
pub struct Model {
    pub config: Code2PromptConfig,
    pub current_tab: Tab,
    pub should_quit: bool,
    
    // File tree state (Tab 1)
    pub file_tree: Vec<FileNode>,
    pub selected_files: HashMap<String, bool>,
    pub search_query: String,
    pub tree_cursor: usize,
    
    // Settings state (Tab 2)
    pub settings_cursor: usize,
    
    // Prompt output state (Tab 3)
    pub generated_prompt: Option<String>,
    pub token_count: Option<usize>,
    pub file_count: usize,
    pub analysis_in_progress: bool,
    pub analysis_error: Option<String>,
    pub output_scroll: u16,
    
    // Status messages
    pub status_message: String,
}

/// The three main tabs of the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    FileTree,      // Tab 1: File selection with search
    Settings,      // Tab 2: Configuration options
    PromptOutput,  // Tab 3: Generated prompt and copy
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
    Choice { options: Vec<String>, selected: usize },
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
    AnalysisComplete(String, usize, usize), // prompt, tokens, files
    AnalysisError(String),
    
    // Prompt output
    CopyToClipboard,
    SaveToFile(String),
    ScrollOutput(i16), // Scroll delta (positive = down, negative = up)
    
}

impl Default for Model {
    fn default() -> Self {
        let config = Code2PromptConfig::default();
        
        Model {
            config,
            current_tab: Tab::FileTree,
            should_quit: false,
            file_tree: Vec::new(),
            selected_files: HashMap::new(),
            search_query: String::new(),
            tree_cursor: 0,
            settings_cursor: 0,
            generated_prompt: None,
            token_count: None,
            file_count: 0,
            analysis_in_progress: false,
            analysis_error: None,
            output_scroll: 0,
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
        let mut model = Self::default();
        model.config.path = path;
        model.config.include_patterns = include_patterns;
        model.config.exclude_patterns = exclude_patterns;
        model
    }
    
    /// Get flattened list of visible file nodes for display
    pub fn get_visible_nodes(&self) -> Vec<&FileNode> {
        let mut visible = Vec::new();
        self.collect_visible_nodes(&self.file_tree, &mut visible);
        visible
    }
    
    fn collect_visible_nodes<'a>(&'a self, nodes: &'a [FileNode], visible: &mut Vec<&'a FileNode>) {
        for node in nodes {
            // Apply search filter
            if self.search_query.is_empty() || 
               node.name.to_lowercase().contains(&self.search_query.to_lowercase()) {
                visible.push(node);
            }
            
            // Add children if expanded and node matches search or has matching children
            if node.is_expanded && (!self.search_query.is_empty() || node.is_directory) {
                self.collect_visible_nodes(&node.children, visible);
            }
        }
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
                        setting_type: SettingType::Boolean(self.config.line_numbers),
                    },
                    SettingsItem {
                        name: "Absolute Paths".to_string(),
                        description: "Use absolute instead of relative paths".to_string(),
                        setting_type: SettingType::Boolean(self.config.absolute_path),
                    },
                    SettingsItem {
                        name: "No Codeblock".to_string(),
                        description: "Don't wrap code in markdown blocks".to_string(),
                        setting_type: SettingType::Boolean(self.config.no_codeblock),
                    },
                    SettingsItem {
                        name: "Output Format".to_string(),
                        description: "Format for generated output".to_string(),
                        setting_type: SettingType::Choice {
                            options: vec!["Markdown".to_string(), "JSON".to_string(), "XML".to_string()],
                            selected: match self.config.output_format {
                                OutputFormat::Markdown => 0,
                                OutputFormat::Json => 1,
                                OutputFormat::Xml => 2,
                            }
                        },
                    },
                    SettingsItem {
                        name: "Token Format".to_string(),
                        description: "How to display token counts".to_string(),
                        setting_type: SettingType::Choice {
                            options: vec!["Raw".to_string(), "Formatted".to_string()],
                            selected: match self.config.token_format {
                                TokenFormat::Raw => 0,
                                TokenFormat::Format => 1,
                            }
                        },
                    },
                ],
            },
            SettingsGroup {
                name: "Git Integration".to_string(),
                items: vec![
                    SettingsItem {
                        name: "Git Diff".to_string(),
                        description: "Include git diff in output".to_string(),
                        setting_type: SettingType::Boolean(self.config.diff_enabled),
                    },
                ],
            },
            SettingsGroup {
                name: "File Selection".to_string(),
                items: vec![
                    SettingsItem {
                        name: "Follow Symlinks".to_string(),
                        description: "Follow symbolic links".to_string(),
                        setting_type: SettingType::Boolean(self.config.follow_symlinks),
                    },
                    SettingsItem {
                        name: "Hidden Files".to_string(),
                        description: "Include hidden files and directories".to_string(),
                        setting_type: SettingType::Boolean(self.config.hidden),
                    },
                    SettingsItem {
                        name: "No Ignore".to_string(),
                        description: "Ignore .gitignore rules".to_string(),
                        setting_type: SettingType::Boolean(self.config.no_ignore),
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
                setting_type: SettingType::Boolean(self.config.line_numbers),
            },
            SettingsItem {
                name: "Absolute Paths".to_string(),
                description: "Use absolute instead of relative paths".to_string(),
                setting_type: SettingType::Boolean(self.config.absolute_path),
            },
            SettingsItem {
                name: "No Codeblock".to_string(),
                description: "Don't wrap code in markdown blocks".to_string(),
                setting_type: SettingType::Boolean(self.config.no_codeblock),
            },
            SettingsItem {
                name: "Output Format".to_string(),
                description: "Format for generated output".to_string(),
                setting_type: SettingType::Choice {
                    options: vec!["Markdown".to_string(), "JSON".to_string(), "XML".to_string()],
                    selected: match self.config.output_format {
                        OutputFormat::Markdown => 0,
                        OutputFormat::Json => 1,
                        OutputFormat::Xml => 2,
                    }
                },
            },
            // Tokenizer section
            SettingsItem {
                name: "Token Format".to_string(),
                description: "How to display token counts".to_string(),
                setting_type: SettingType::Choice {
                    options: vec!["Raw".to_string(), "Formatted".to_string()],
                    selected: match self.config.token_format {
                        TokenFormat::Raw => 0,
                        TokenFormat::Format => 1,
                    }
                },
            },
            // Git section
            SettingsItem {
                name: "Git Diff".to_string(),
                description: "Include git diff in output".to_string(),
                setting_type: SettingType::Boolean(self.config.diff_enabled),
            },
            // File selection section
            SettingsItem {
                name: "Follow Symlinks".to_string(),
                description: "Follow symbolic links".to_string(),
                setting_type: SettingType::Boolean(self.config.follow_symlinks),
            },
            SettingsItem {
                name: "Hidden Files".to_string(),
                description: "Include hidden files and directories".to_string(),
                setting_type: SettingType::Boolean(self.config.hidden),
            },
            SettingsItem {
                name: "No Ignore".to_string(),
                description: "Ignore .gitignore rules".to_string(),
                setting_type: SettingType::Boolean(self.config.no_ignore),
            },
        ]
    }
    
    /// Update setting based on index and action (works with grouped settings)
    pub fn update_setting(&mut self, index: usize, action: SettingAction) {
        // Map flat index to actual setting based on grouped structure
        match index {
            0 => self.config.line_numbers = !self.config.line_numbers, // Line Numbers
            1 => self.config.absolute_path = !self.config.absolute_path, // Absolute Paths  
            2 => self.config.no_codeblock = !self.config.no_codeblock, // No Codeblock
            3 => { // Output Format
                if let SettingAction::Cycle = action {
                    self.config.output_format = match self.config.output_format {
                        OutputFormat::Markdown => OutputFormat::Json,
                        OutputFormat::Json => OutputFormat::Xml,
                        OutputFormat::Xml => OutputFormat::Markdown,
                    };
                }
            },
            4 => { // Token Format
                if let SettingAction::Cycle = action {
                    self.config.token_format = match self.config.token_format {
                        TokenFormat::Raw => TokenFormat::Format,
                        TokenFormat::Format => TokenFormat::Raw,
                    };
                }
            },
            5 => self.config.diff_enabled = !self.config.diff_enabled, // Git Diff
            6 => self.config.follow_symlinks = !self.config.follow_symlinks, // Follow Symlinks
            7 => self.config.hidden = !self.config.hidden, // Hidden Files
            8 => self.config.no_ignore = !self.config.no_ignore, // No Ignore
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
}

impl FileNode {
    pub fn new(path: PathBuf, level: usize) -> Self {
        let name = path.file_name()
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
    
    
    pub fn sort_children(&mut self) {
        self.children.sort_by(|a, b| {
            // Directories first, then files
            match (a.is_directory, b.is_directory) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });
        
        // Recursively sort children
        for child in &mut self.children {
            child.sort_children();
        }
    }
}