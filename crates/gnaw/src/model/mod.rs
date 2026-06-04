//! Data structures and application state management for the TUI.
//!
//! This module contains the core data structures that represent the application state,
//! including the main Model struct, tab definitions, message types for event handling,
//! and all state management submodules. It serves as the central state container
//! for the terminal user interface.

pub mod commands;
pub mod prompt_output;
pub mod settings;
pub mod statistics;
pub mod template;

pub use commands::*;
pub use prompt_output::*;
pub use settings::*;
pub use statistics::*;
pub use template::*;

use crate::utils::directory_contains_selected_files;
use gnaw_core::session::GnawSession;
use std::collections::HashMap;
use std::path::PathBuf;

/// The five main tabs of the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    FileTree,
    Settings,
    Statistics,
    Template,
    PromptOutput,
}

/// Input mode for the FileTree tab
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileTreeInputMode {
    Browsing,
    Search,
}

/// Top-level interaction mode (vim-style). Derived for display via `Model::mode()`;
/// only the command line is stored, the rest is computed from existing flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Normal,
    Insert,
    Command,
}

/// How the file tree orders siblings. Path is the default (stable spatial layout);
/// TokenWeight reorders by aggregate token count, heaviest first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeSortMode {
    Path,
    TokenWeight,
}

impl TreeSortMode {
    fn toggled(self) -> Self {
        match self {
            TreeSortMode::Path => TreeSortMode::TokenWeight,
            TreeSortMode::TokenWeight => TreeSortMode::Path,
        }
    }
}
/// Per-file token-counting state. Map presence doubles as the work queue
/// (Pending entries are what FlushTokenQueue drains) and as a cache.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenState {
    Pending,  // selected, queued, not yet counting
    Counting, // handed to a background task
    Done(usize),
    Failed, // binary/empty/unreadable
}

/// Active token-size filter on the tree. None = no size filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeFilter {
    GreaterThan(usize),
    LessThan(usize),
}

/// Hierarchical file node for TUI display with proper parent-child relationships
#[derive(Debug, Clone)]
pub struct DisplayFileNode {
    pub path: std::path::PathBuf,
    pub name: String,
    pub is_directory: bool,
    pub is_expanded: bool,
    pub level: usize,
    pub children_loaded: bool,
    pub children: Vec<DisplayFileNode>,
    /// Σ Done-tokens of selected leaves under this node. None until first aggregated.
    /// Maintained on count-quiescence; doubles as the weight-sort key.
    pub agg_tokens: Option<usize>,
}

impl DisplayFileNode {
    pub fn new(path: std::path::PathBuf, level: usize) -> Self {
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
            level,
            children_loaded: false,
            children: Vec::new(),
            agg_tokens: None,
        }
    }

    /// Find a node by path in the tree (recursive)
    pub fn find_node_mut(&mut self, target_path: &std::path::Path) -> Option<&mut DisplayFileNode> {
        if self.path == target_path {
            return Some(self);
        }

        for child in &mut self.children {
            if let Some(found) = child.find_node_mut(target_path) {
                return Some(found);
            }
        }

        None
    }

    /// Load children for this directory node
    pub fn load_children(
        &mut self,
        session: &mut gnaw_core::session::GnawSession,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !self.is_directory || self.children_loaded {
            return Ok(());
        }

        self.children.clear();

        // Use ignore crate to respect gitignore
        use ignore::WalkBuilder;
        let walker = WalkBuilder::new(&self.path).max_depth(Some(1)).build();

        for entry in walker {
            let entry = entry?;
            let path = entry.path();

            if path == self.path {
                continue; // Skip self
            }

            let mut child = DisplayFileNode::new(path.to_path_buf(), self.level + 1);

            // Auto-expand if contains selected files
            if child.is_directory && directory_contains_selected_files(&child.path, session) {
                child.is_expanded = true;
            }

            self.children.push(child);
        }

        // Sort children: directories first, then alphabetically
        self.children
            .sort_by(|a, b| match (a.is_directory, b.is_directory) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            });

        self.children_loaded = true;
        Ok(())
    }
}

/// Collapse every directory in the tree. Children stay loaded (children_loaded
/// untouched) so re-expanding is instant and doesn't re-walk the filesystem.
fn collapse_all(nodes: &mut [DisplayFileNode]) {
    for n in nodes {
        if n.is_directory {
            n.is_expanded = false;
            collapse_all(&mut n.children); // collapse descendants too, so re-expanding
            // one level doesn't reveal a still-expanded subtree
        }
    }
}
/// Recompute agg_tokens bottom-up: a file's weight is its Done count (else 0),
/// a directory's is the sum of its children. Fills every node on the way up.
fn recompute_agg(
    node: &mut DisplayFileNode,
    states: &HashMap<PathBuf, TokenState>,
    session: &mut GnawSession,
) -> usize {
    let total = if node.is_directory {
        node.children
            .iter_mut()
            .map(|c| recompute_agg(c, states, session))
            .sum()
    } else if session.is_file_selected(&node.path) {
        match states.get(&node.path) {
            Some(TokenState::Done(n)) => *n,
            _ => 0,
        }
    } else {
        0 // deselected leaves don't contribute, even if their count is still cached
    };
    node.agg_tokens = Some(total);
    total
}

/// Heaviest first; ties fall back to dirs-first-then-name so equal-weight nodes
/// (e.g. all-zero before counting) keep a stable, familiar order instead of jumping.
fn weight_cmp(a: &DisplayFileNode, b: &DisplayFileNode) -> std::cmp::Ordering {
    b.agg_tokens
        .unwrap_or(0)
        .cmp(&a.agg_tokens.unwrap_or(0))
        .then_with(|| path_cmp(a, b))
}

/// Dirs first, then alphabetical — the original tree ordering.
fn path_cmp(a: &DisplayFileNode, b: &DisplayFileNode) -> std::cmp::Ordering {
    match (a.is_directory, b.is_directory) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.cmp(&b.name),
    }
}

fn sort_recursive(nodes: &mut [DisplayFileNode], by_weight: bool) {
    let cmp = if by_weight { weight_cmp } else { path_cmp };
    nodes.sort_by(cmp);
    for n in nodes {
        sort_recursive(&mut n.children, by_weight);
    }
}

/// Parse a size-filter arg: ">N" / "<N", with k/m suffixes. "" or "0" clears.
fn parse_size_filter(arg: &str) -> Result<Option<SizeFilter>, String> {
    if arg.is_empty() || arg == "0" {
        return Ok(None);
    }
    let (ctor, num): (fn(usize) -> SizeFilter, &str) = match arg.chars().next() {
        Some('>') => (SizeFilter::GreaterThan, &arg[1..]),
        Some('<') => (SizeFilter::LessThan, &arg[1..]),
        _ => return Err("Usage: :size >N | <N | 0  (e.g. :size >500, :size <2k)".into()),
    };
    let n = parse_count(num.trim()).ok_or_else(|| format!("Bad size: {num} (try >500 or <2k)"))?;
    Ok(Some(ctor(n)))
}

/// "500" → 500, "2k" → 2000, "1m" → 1_000_000.
fn parse_count(s: &str) -> Option<usize> {
    let s = s.to_lowercase();
    if let Some(stripped) = s.strip_suffix('k') {
        stripped
            .trim()
            .parse::<f64>()
            .ok()
            .map(|v| (v * 1_000.0) as usize)
    } else if let Some(stripped) = s.strip_suffix('m') {
        stripped
            .trim()
            .parse::<f64>()
            .ok()
            .map(|v| (v * 1_000_000.0) as usize)
    } else {
        s.parse().ok()
    }
}

/// Messages for updating the model
#[derive(Debug, Clone)]
pub enum Message {
    SwitchTab(Tab),
    Quit,

    EnterCommandMode,
    ExitCommandMode,
    CommandInputChar(char),
    CommandInputBackspace,
    ExecuteCommand,

    SearchHistoryPrev,
    UpdateSearchQuery(String),
    ToggleFileSelection(usize),
    /// Select every visible (search-matched) leaf file.
    SelectMatches,
    /// Deselect every visible (search-matched) leaf file.
    DeselectMatches,
    ExpandDirectory(usize),
    CollapseDirectory(usize),
    MoveTreeCursor(i32),
    RefreshFileTree,

    /// A background count finished (tokens=None means binary/empty/failed).
    TokenCounted {
        path: PathBuf,
        tokens: Option<usize>,
    },
    /// Debounce fired for this generation; drain Pending if still current.
    FlushTokenQueue(u64),
    /// After the tree loads, enqueue files that were already selected.
    InitialTokenScan,

    EnterSearchMode,
    ExitSearchMode,

    MoveSettingsCursor(i32),
    ToggleSetting(usize),
    CycleSetting(usize),

    RunAnalysis,
    AnalysisComplete(AnalysisResults),
    AnalysisError(String),

    CopyToClipboard,
    SaveToFile(String),
    ScrollOutput(i16),

    CycleStatisticsView(i8),
    ScrollStatistics(i16),

    SaveTemplate(String),
    ReloadTemplate,
    LoadTemplate,
    RefreshTemplates,

    SetTemplateFocus(TemplateFocus, FocusMode),
    SetTemplateFocusMode(FocusMode),
    TemplateEditorInput(ratatui::crossterm::event::KeyEvent),
    TemplatePickerMove(i32),

    VariableStartEditing(String),
    VariableInputChar(char),
    VariableInputBackspace,
    VariableInputEnter,
    VariableInputCancel,
    VariableNavigateUp,
    VariableNavigateDown,
}

/// Represents the overall state of the TUI application.
#[derive(Debug, Clone)]
pub struct Model {
    pub session: GnawSession,
    pub current_tab: Tab,
    pub should_quit: bool,
    pub file_tree_input_mode: FileTreeInputMode,
    pub file_tree_nodes: Vec<DisplayFileNode>,
    pub search_query: String,
    pub tree_cursor: usize,
    pub file_tree_scroll: u16,
    pub settings: SettingsState,
    pub statistics: StatisticsState,
    pub template: TemplateState,
    pub prompt_output: PromptOutputState,
    pub status_message: String,
    /// `Some` while the `:` command line is open; holds the text typed after the colon.
    pub command_line: Option<String>,
    /// Committed search queries, oldest first, newest last. Capped, de-duplicated.
    pub search_history: Vec<String>,
    /// Cursor into history while cycling with Ctrl+P; None = editing live, not browsing history.
    pub search_history_pos: Option<usize>,
    /// Token-counting state per absolute file path. Only selected files appear
    /// here; deselected entries may linger as a cache and are filtered out of totals.
    pub token_states: HashMap<PathBuf, TokenState>,
    /// Debounce generation; bumped on every (re)schedule, checked on flush.
    pub token_debounce_gen: u64,
    /// Cached Σ tokens of currently-selected Done files (the % denominator).
    pub selected_token_total: usize,
    /// Current tree ordering. Path by default to preserve spatial memory.
    pub tree_sort_mode: TreeSortMode,
    pub size_filter: Option<SizeFilter>,
}

impl Default for Model {
    fn default() -> Self {
        let config = gnaw_core::configuration::GnawConfig::default();
        let session = GnawSession::new(config);

        Model {
            session,
            current_tab: Tab::FileTree,
            should_quit: false,
            file_tree_input_mode: FileTreeInputMode::Browsing,
            file_tree_nodes: Vec::new(),
            search_query: String::new(),
            tree_cursor: 0,
            file_tree_scroll: 0,
            settings: SettingsState::default(),
            statistics: StatisticsState::default(),
            template: TemplateState::default(),
            prompt_output: PromptOutputState::default(),
            status_message: String::new(),
            command_line: None,
            search_history: Vec::new(),
            search_history_pos: None,
            token_states: HashMap::new(),
            token_debounce_gen: 0,
            selected_token_total: 0,
            tree_sort_mode: TreeSortMode::Path,
            size_filter: None,
        }
    }
}

impl Model {
    pub fn new(session: GnawSession) -> Self {
        Model {
            session,
            current_tab: Tab::FileTree,
            should_quit: false,
            file_tree_input_mode: FileTreeInputMode::Browsing,
            file_tree_nodes: Vec::new(),
            search_query: String::new(),
            tree_cursor: 0,
            file_tree_scroll: 0,
            settings: SettingsState::default(),
            statistics: StatisticsState::default(),
            template: TemplateState::default(),
            prompt_output: PromptOutputState::default(),
            status_message: String::new(),
            command_line: None,
            search_history: Vec::new(),
            search_history_pos: None,
            token_states: HashMap::new(),
            token_debounce_gen: 0,
            selected_token_total: 0,
            tree_sort_mode: TreeSortMode::Path,
            size_filter: None,
        }
    }

    /// Get grouped settings for display
    pub fn get_settings_groups(&self) -> Vec<SettingsGroup> {
        crate::view::format_settings_groups(&self.session)
    }

    /// The current interaction mode, derived from existing state. Command mode is
    /// explicit (command_line is open); insert is computed from search/template flags.
    pub fn mode(&self) -> AppMode {
        if self.command_line.is_some() {
            AppMode::Command
        } else if self.is_in_insert_context() {
            AppMode::Insert
        } else {
            AppMode::Normal
        }
    }

    /// Recompute the % denominator: Σ Done tokens over currently-selected files.
    /// Clones keys first to avoid borrowing token_states while mutating session.
    fn recompute_selected_token_total(&mut self) {
        let done: Vec<(PathBuf, usize)> = self
            .token_states
            .iter()
            .filter_map(|(p, s)| match s {
                TokenState::Done(n) => Some((p.clone(), *n)),
                _ => None,
            })
            .collect();
        let mut total = 0;
        for (p, n) in done {
            if self.session.is_file_selected(&p) {
                total += n;
            }
        }
        self.selected_token_total = total;
    }

    /// Recompute every node's agg_tokens, then re-order siblings per the current
    /// sort mode. Preserves the cursor by path across the reorder. Cheap parts
    /// (aggregate walk) always run; the reorder only matters in TokenWeight mode
    /// but Path mode re-applies the canonical order too (harmless, keeps it consistent).
    fn refresh_tree_aggregates_and_sort(&mut self) {
        // 1. Aggregates — disjoint field borrows (file_tree_nodes mut, token_states immut).
        {
            let states = &self.token_states;
            let session = &mut self.session;
            for n in &mut self.file_tree_nodes {
                recompute_agg(n, states, session);
            }
        }

        // 2. Capture the path under the cursor before the order changes.
        let cursor_path = {
            let vis = crate::utils::get_visible_nodes(
                &self.file_tree_nodes,
                &self.search_query,
                self.size_filter,
                &self.token_states,
                &mut self.session,
            );
            vis.get(self.tree_cursor).map(|d| d.node.path.clone())
        };

        // 3. Reorder.
        let by_weight = self.tree_sort_mode == TreeSortMode::TokenWeight;
        sort_recursive(&mut self.file_tree_nodes, by_weight);

        // 4. Restore the cursor to wherever that path landed.
        if let Some(path) = cursor_path {
            let vis = crate::utils::get_visible_nodes(
                &self.file_tree_nodes,
                &self.search_query,
                self.size_filter,
                &self.token_states,
                &mut self.session,
            );
            if let Some(idx) = vis.iter().position(|d| d.node.path == path) {
                self.tree_cursor = idx;
            }
        }
    }

    fn is_in_insert_context(&self) -> bool {
        let searching = self.current_tab == Tab::FileTree
            && self.file_tree_input_mode == FileTreeInputMode::Search;
        let editing_template =
            self.current_tab == Tab::Template && self.template.is_in_editing_mode();
        searching || editing_template
    }

    pub fn update(&self, message: Message) -> (Self, Cmd) {
        let mut new_model = self.clone();

        match message {
            Message::Quit => {
                new_model.should_quit = true;
                new_model.status_message = "Goodbye!".to_string();
                (new_model, Cmd::None)
            }

            Message::EnterCommandMode => {
                new_model.command_line = Some(String::new());
                new_model.status_message.clear();
                (new_model, Cmd::None)
            }

            Message::ExitCommandMode => {
                new_model.command_line = None;
                (new_model, Cmd::None)
            }

            Message::CommandInputChar(c) => {
                if let Some(buf) = new_model.command_line.as_mut() {
                    buf.push(c);
                }
                (new_model, Cmd::None)
            }

            Message::CommandInputBackspace => {
                // Backspacing past the ':' closes the command line (vim behavior).
                match new_model.command_line.as_mut() {
                    Some(buf) if !buf.is_empty() => {
                        buf.pop();
                    }
                    _ => new_model.command_line = None,
                }
                (new_model, Cmd::None)
            }

            Message::ExecuteCommand => {
                // take() closes the command line regardless of outcome.
                let cmd = new_model
                    .command_line
                    .take()
                    .unwrap_or_default()
                    .trim()
                    .to_string();
                match cmd.as_str() {
                    "q" | "q!" | "quit" => {
                        new_model.should_quit = true;
                        new_model.status_message = "Goodbye!".to_string();
                    }
                    // "w" | "wq" => /* wire to SaveToFile later */,
                    "" => {} // bare ":" — no-op
                    other => {
                        if let Some(rest) = other.strip_prefix("sort") {
                            let arg = rest.trim();
                            let new_mode = match arg {
                                "tokens" | "weight" => Some(TreeSortMode::TokenWeight),
                                "path" | "name" => Some(TreeSortMode::Path),
                                "" => Some(new_model.tree_sort_mode.toggled()),
                                _ => None,
                            };
                            match new_mode {
                                Some(mode) => {
                                    new_model.tree_sort_mode = mode;
                                    new_model.refresh_tree_aggregates_and_sort();
                                    new_model.status_message = match mode {
                                        TreeSortMode::TokenWeight => {
                                            "Tree sorted by token weight".to_string()
                                        }
                                        TreeSortMode::Path => "Tree sorted by path".to_string(),
                                    };
                                }
                                None => {
                                    new_model.status_message =
                                        format!("Unknown sort mode: {arg} (try tokens|path)");
                                }
                            }
                        } else if other == "collapse" {
                            collapse_all(&mut new_model.file_tree_nodes);
                            new_model.tree_cursor = 0;
                            new_model.file_tree_scroll = 0;
                            new_model.status_message = "Collapsed all folders".to_string();
                        } else if let Some(rest) = other.strip_prefix("size") {
                            let arg = rest.trim();
                            match parse_size_filter(arg) {
                                Ok(filter) => {
                                    new_model.size_filter = filter;
                                    new_model.tree_cursor = 0;
                                    new_model.file_tree_scroll = 0;
                                    new_model.status_message = match filter {
                                        Some(SizeFilter::GreaterThan(n)) => {
                                            format!("Filtering: files over {n} tokens")
                                        }
                                        Some(SizeFilter::LessThan(n)) => {
                                            format!("Filtering: files under {n} tokens")
                                        }
                                        None => "Size filter cleared".to_string(),
                                    };
                                }
                                Err(e) => new_model.status_message = e,
                            }
                        } else {
                            new_model.status_message = format!("Not a command: :{other}");
                        }
                    }
                }
                (new_model, Cmd::None)
            }

            Message::SwitchTab(tab) => {
                new_model.current_tab = tab;
                new_model.status_message = format!("Switched to {:?} tab", tab);
                (new_model, Cmd::None)
            }

            Message::RefreshFileTree => {
                new_model.status_message = "Refreshing file tree...".to_string();
                (new_model, Cmd::RefreshFileTree)
            }

            Message::UpdateSearchQuery(query) => {
                new_model.search_query = query;
                new_model.search_history_pos = None; // live typing leaves history-browsing
                new_model.tree_cursor = 0;
                new_model.file_tree_scroll = 0;
                (new_model, Cmd::None)
            }

            Message::EnterSearchMode => {
                new_model.search_query.clear();
                new_model.search_history_pos = None;
                new_model.tree_cursor = 0;
                new_model.file_tree_scroll = 0;
                new_model.file_tree_input_mode = FileTreeInputMode::Search;
                new_model.status_message = "Search mode - Type to search, Esc to exit".to_string();
                (new_model, Cmd::None)
            }

            Message::ExitSearchMode => {
                let q = new_model.search_query.trim().to_string();
                if !q.is_empty() {
                    // De-dup: if it's already in history, lift it to newest rather than dupe.
                    new_model.search_history.retain(|h| h != &q);
                    new_model.search_history.push(q);
                    const MAX_HISTORY: usize = 50;
                    let len = new_model.search_history.len();
                    if len > MAX_HISTORY {
                        new_model.search_history.drain(0..len - MAX_HISTORY);
                    }
                }
                new_model.search_history_pos = None;
                new_model.file_tree_input_mode = FileTreeInputMode::Browsing;
                new_model.status_message = "Exited search mode".to_string();
                (new_model, Cmd::None)
            }
            Message::SearchHistoryPrev => {
                if !new_model.search_history.is_empty() {
                    let next = match new_model.search_history_pos {
                        None => new_model.search_history.len() - 1, // first press: newest
                        Some(0) => 0,                               // already oldest, stay
                        Some(i) => i - 1,
                    };
                    new_model.search_history_pos = Some(next);
                    new_model.search_query = new_model.search_history[next].clone();
                    new_model.tree_cursor = 0;
                    new_model.file_tree_scroll = 0;
                }
                (new_model, Cmd::None)
            }
            Message::MoveTreeCursor(delta) => {
                let visible_nodes = crate::utils::get_visible_nodes(
                    &new_model.file_tree_nodes,
                    &new_model.search_query,
                    new_model.size_filter,
                    &new_model.token_states,
                    &mut new_model.session,
                );
                let visible_count = visible_nodes.len();

                if visible_count > 0 {
                    let new_cursor = if delta > 0 {
                        (new_model.tree_cursor + delta as usize).min(visible_count - 1)
                    } else {
                        new_model.tree_cursor.saturating_sub((-delta) as usize)
                    };
                    new_model.tree_cursor = new_cursor;
                }
                (new_model, Cmd::None)
            }

            Message::MoveSettingsCursor(delta) => {
                let settings_count = new_model
                    .settings
                    .get_settings_items(&new_model.session)
                    .len();
                if settings_count > 0 {
                    let new_cursor = if delta > 0 {
                        (new_model.settings.settings_cursor + delta as usize)
                            .min(settings_count - 1)
                    } else {
                        new_model
                            .settings
                            .settings_cursor
                            .saturating_sub((-delta) as usize)
                    };
                    new_model.settings.settings_cursor = new_cursor;
                }
                (new_model, Cmd::None)
            }

            Message::ToggleFileSelection(index) => {
                let visible_nodes = crate::utils::get_visible_nodes(
                    &new_model.file_tree_nodes,
                    &new_model.search_query,
                    new_model.size_filter,
                    &new_model.token_states,
                    &mut new_model.session,
                );

                if let Some(display_node) = visible_nodes.get(index) {
                    let node_path = display_node.node.path.clone();
                    let name = display_node.node.name.clone();
                    let is_directory = display_node.node.is_directory;
                    let current = display_node.is_selected;

                    // Convert to relative path for session
                    let relative_path =
                        if let Ok(rel) = node_path.strip_prefix(&new_model.session.config.path) {
                            rel.to_path_buf()
                        } else {
                            node_path.clone()
                        };

                    // Update session selection state (single source of truth)
                    new_model.session.toggle_file_selection(relative_path);

                    let action = if current { "Deselected" } else { "Selected" };
                    let extra = if is_directory { " (and contents)" } else { "" };
                    new_model.status_message = format!("{} {}{}", action, name, extra);

                    // Selected-only counting. On select, mark newly-selected leaves
                    // Pending and (re)arm the debounce. On deselect, just refresh the %.
                    let mut scheduled = false;
                    if !current {
                        let leaves = crate::utils::collect_selected_files_under(
                            &node_path,
                            &mut new_model.session,
                        );
                        for p in leaves {
                            // Skip anything already counted/counting/queued (dedup).
                            if !matches!(
                                new_model.token_states.get(&p),
                                Some(TokenState::Done(_))
                                    | Some(TokenState::Counting)
                                    | Some(TokenState::Pending)
                            ) {
                                new_model.token_states.insert(p, TokenState::Pending);
                                scheduled = true;
                            }
                        }
                    }

                    new_model.recompute_selected_token_total();

                    if scheduled {
                        new_model.token_debounce_gen += 1;
                        let debounce_gen = new_model.token_debounce_gen;
                        return (new_model, Cmd::ScheduleTokenCount(debounce_gen));
                    }

                    // Deselect or all-already-counted: if nothing is in flight, the
                    // aggregates/totals just changed, so refresh now.
                    let busy = new_model
                        .token_states
                        .values()
                        .any(|s| matches!(s, TokenState::Pending | TokenState::Counting));
                    if !busy {
                        new_model.refresh_tree_aggregates_and_sort();
                    }
                }
                (new_model, Cmd::None)
            }

            Message::SelectMatches | Message::DeselectMatches => {
                let select = matches!(message, Message::SelectMatches);

                let visible_nodes = crate::utils::get_visible_nodes(
                    &new_model.file_tree_nodes,
                    &new_model.search_query,
                    new_model.size_filter,
                    &new_model.token_states,
                    &mut new_model.session,
                );

                // Visible *leaves* only. Never bulk-toggle a directory node — that would
                // pull in its hidden, non-matching children, which is exactly the surprise
                // we're trying to remove.
                let leaves: Vec<PathBuf> = visible_nodes
                    .iter()
                    .filter(|d| !d.node.is_directory)
                    .map(|d| d.node.path.clone())
                    .collect();

                let mut changed = 0usize;
                let mut scheduled = false;

                for path in &leaves {
                    // session normalizes abs→rel internally, so the absolute path is fine.
                    if new_model.session.is_file_selected(path) == select {
                        continue; // already in the desired state
                    }
                    new_model.session.toggle_file_selection(path.clone());
                    changed += 1;

                    // Same dedup guard as ToggleFileSelection: only enqueue counts for
                    // leaves not already done/counting/queued.
                    if select
                        && !matches!(
                            new_model.token_states.get(path),
                            Some(TokenState::Done(_))
                                | Some(TokenState::Counting)
                                | Some(TokenState::Pending)
                        )
                    {
                        new_model
                            .token_states
                            .insert(path.clone(), TokenState::Pending);
                        scheduled = true;
                    }
                }

                new_model.status_message = if changed == 0 {
                    format!(
                        "No matching files to {}",
                        if select { "select" } else { "deselect" }
                    )
                } else {
                    let verb = if select { "Selected" } else { "Deselected" };
                    format!("{verb} {changed} matching file(s)")
                };

                new_model.recompute_selected_token_total();

                if scheduled {
                    new_model.token_debounce_gen += 1;
                    let debounce_gen = new_model.token_debounce_gen;
                    return (new_model, Cmd::ScheduleTokenCount(debounce_gen));
                }

                // Deselect, or everything already counted: refresh now if nothing's in flight.
                let busy = new_model
                    .token_states
                    .values()
                    .any(|s| matches!(s, TokenState::Pending | TokenState::Counting));
                if !busy {
                    new_model.refresh_tree_aggregates_and_sort();
                }
                (new_model, Cmd::None)
            }

            Message::FlushTokenQueue(debounce_gen) => {
                // Stale flush from a superseded debounce — drop it.
                if debounce_gen != new_model.token_debounce_gen {
                    return (new_model, Cmd::None);
                }
                let pending: Vec<PathBuf> = new_model
                    .token_states
                    .iter()
                    .filter(|(_, s)| matches!(s, TokenState::Pending))
                    .map(|(p, _)| p.clone())
                    .collect();

                let mut to_count = Vec::new();
                for p in pending {
                    if new_model.session.is_file_selected(&p) {
                        new_model
                            .token_states
                            .insert(p.clone(), TokenState::Counting);
                        to_count.push(p);
                    } else {
                        // Deselected during the debounce window — forget it.
                        new_model.token_states.remove(&p);
                    }
                }

                if to_count.is_empty() {
                    (new_model, Cmd::None)
                } else {
                    (new_model, Cmd::CountTokens { paths: to_count })
                }
            }

            Message::TokenCounted { path, tokens } => {
                // Ignore results invalidated by an encoding change (key removed).
                if new_model.token_states.contains_key(&path) {
                    let state = match tokens {
                        Some(n) => TokenState::Done(n),
                        None => TokenState::Failed,
                    };
                    new_model.token_states.insert(path, state);
                    new_model.recompute_selected_token_total();

                    // Quiescent (no Pending/Counting left)? The count set is now stable,
                    // so aggregates are complete — refresh and sort exactly once.
                    let busy = new_model
                        .token_states
                        .values()
                        .any(|s| matches!(s, TokenState::Pending | TokenState::Counting));
                    if !busy {
                        new_model.refresh_tree_aggregates_and_sort();
                    }
                }
                (new_model, Cmd::None)
            }

            Message::InitialTokenScan => {
                let leaves = crate::utils::collect_selected_files_in_tree(
                    &new_model.file_tree_nodes,
                    &mut new_model.session,
                );
                let mut scheduled = false;
                for p in leaves {
                    if !new_model.token_states.contains_key(&p) {
                        new_model.token_states.insert(p, TokenState::Pending);
                        scheduled = true;
                    }
                }
                new_model.recompute_selected_token_total();
                if scheduled {
                    new_model.token_debounce_gen += 1;
                    let debounce_gen = new_model.token_debounce_gen;
                    (new_model, Cmd::ScheduleTokenCount(debounce_gen))
                } else {
                    (new_model, Cmd::None)
                }
            }
            Message::ExpandDirectory(index) => {
                let visible_nodes = crate::utils::get_visible_nodes(
                    &new_model.file_tree_nodes,
                    &new_model.search_query,
                    new_model.size_filter,
                    &new_model.token_states,
                    &mut new_model.session,
                );

                if let Some(display_node) = visible_nodes.get(index)
                    && display_node.node.is_directory
                {
                    let node_path = display_node.node.path.clone();
                    let name = display_node.node.name.clone();

                    // Ensure the path exists in the tree first
                    if let Err(e) = crate::utils::ensure_path_exists_in_tree(
                        &mut new_model.file_tree_nodes,
                        &node_path,
                        &mut new_model.session,
                    ) {
                        new_model.status_message =
                            format!("Failed to ensure path exists for {}: {}", name, e);
                        return (new_model, Cmd::None);
                    }

                    // Find and expand the node in the tree
                    let mut found = false;
                    for root_node in &mut new_model.file_tree_nodes {
                        if let Some(node) = root_node.find_node_mut(&node_path) {
                            if !node.is_expanded {
                                node.is_expanded = true;
                                // Load children if not already loaded
                                if !node.children_loaded
                                    && let Err(e) = node.load_children(&mut new_model.session)
                                {
                                    new_model.status_message =
                                        format!("Failed to load children for {}: {}", name, e);
                                    return (new_model, Cmd::None);
                                }
                                new_model.status_message = format!("Expanded {}", name);
                            } else {
                                new_model.status_message = format!("{} is already expanded", name);
                            }
                            found = true;
                            break;
                        }
                    }

                    if !found {
                        new_model.status_message = format!("Could not find directory {}", name);
                    }
                }
                (new_model, Cmd::None)
            }

            Message::CollapseDirectory(index) => {
                let visible_nodes = crate::utils::get_visible_nodes(
                    &new_model.file_tree_nodes,
                    &new_model.search_query,
                    new_model.size_filter,
                    &new_model.token_states,
                    &mut new_model.session,
                );

                if let Some(display_node) = visible_nodes.get(index)
                    && display_node.node.is_directory
                {
                    let node_path = display_node.node.path.clone();
                    let name = display_node.node.name.clone();

                    // Find and collapse the node in the tree
                    let mut found = false;
                    for root_node in &mut new_model.file_tree_nodes {
                        if let Some(node) = root_node.find_node_mut(&node_path)
                            && node.is_expanded
                        {
                            node.is_expanded = false;
                            new_model.status_message = format!("Collapsed {}", name);
                            found = true;
                            break;
                        }
                    }

                    if !found {
                        new_model.status_message = format!("Could not find directory {}", name);
                    }
                }
                (new_model, Cmd::None)
            }

            Message::ToggleSetting(index) => {
                let items = new_model.settings.get_settings_items(&new_model.session);
                if let Some(item) = items.get(index) {
                    let setting_name = new_model.settings.update_setting_by_key(
                        &mut new_model.session,
                        item.key,
                        SettingAction::Toggle,
                    );
                    new_model.status_message = format!("Toggled {}", setting_name);
                } else {
                    new_model.status_message = format!("Invalid setting index: {}", index);
                }
                (new_model, Cmd::None)
            }

            Message::CycleSetting(index) => {
                let items = new_model.settings.get_settings_items(&new_model.session);
                if let Some(item) = items.get(index) {
                    let key = item.key;
                    let setting_name = new_model.settings.update_setting_by_key(
                        &mut new_model.session,
                        key,
                        SettingAction::Cycle,
                    );
                    new_model.status_message = format!("Cycled {}", setting_name);

                    // Token counts are encoding-specific — invalidate and re-enqueue
                    // the still-selected files when the tokenizer changes.
                    if key == SettingKey::TokenizerType {
                        let previously: Vec<PathBuf> =
                            new_model.token_states.keys().cloned().collect();
                        new_model.token_states.clear();
                        new_model.selected_token_total = 0;
                        for p in previously {
                            if new_model.session.is_file_selected(&p) {
                                new_model.token_states.insert(p, TokenState::Pending);
                            }
                        }
                        new_model.token_debounce_gen += 1;
                        let debounce_gen = new_model.token_debounce_gen;
                        return (new_model, Cmd::ScheduleTokenCount(debounce_gen));
                    }
                } else {
                    new_model.status_message = format!("Invalid setting index: {}", index);
                }
                (new_model, Cmd::None)
            }

            Message::RunAnalysis => {
                if !new_model.prompt_output.analysis_in_progress {
                    new_model.prompt_output.analysis_in_progress = true;
                    new_model.prompt_output.analysis_error = None;
                    new_model.status_message = "Running analysis...".to_string();
                    new_model.current_tab = Tab::PromptOutput; // Switch to output tab

                    let cmd = Cmd::RunAnalysis {
                        template_content: new_model.template.get_template_content().to_string(),
                        user_variables: new_model.template.variables.user_variables.clone(),
                    };
                    (new_model, cmd)
                } else {
                    new_model.status_message = "Analysis already in progress...".to_string();
                    (new_model, Cmd::None)
                }
            }

            Message::AnalysisComplete(results) => {
                new_model.prompt_output.analysis_in_progress = false;
                new_model.prompt_output.generated_prompt = Some(results.generated_prompt);
                new_model.prompt_output.token_count = results.token_count;
                new_model.prompt_output.file_count = results.file_count;
                // Reset output scroll so the new content starts at the top.
                new_model.prompt_output.output_scroll = 0;
                new_model.statistics.token_map_entries = results.token_map_entries;
                let tokens = results.token_count.unwrap_or(0);
                new_model.status_message = format!(
                    "Analysis complete! {} tokens, {} files",
                    tokens, results.file_count
                );
                (new_model, Cmd::None)
            }

            Message::AnalysisError(error) => {
                new_model.prompt_output.analysis_in_progress = false;
                new_model.prompt_output.analysis_error = Some(error.clone());
                new_model.status_message = format!("Analysis failed: {}", error);
                (new_model, Cmd::None)
            }

            Message::CopyToClipboard => {
                if let Some(prompt) = &new_model.prompt_output.generated_prompt {
                    let cmd = Cmd::CopyToClipboard(prompt.clone());
                    (new_model, cmd)
                } else {
                    new_model.status_message = "No prompt to copy".to_string();
                    (new_model, Cmd::None)
                }
            }

            Message::SaveToFile(filename) => {
                if let Some(prompt) = &new_model.prompt_output.generated_prompt {
                    let cmd = Cmd::SaveToFile {
                        filename,
                        content: prompt.clone(),
                    };
                    (new_model, cmd)
                } else {
                    new_model.status_message = "No prompt to save".to_string();
                    (new_model, Cmd::None)
                }
            }

            Message::ScrollOutput(delta) => {
                // Apply delta only; widgets will clamp based on actual viewport.
                let new_scroll = if delta < 0 {
                    new_model
                        .prompt_output
                        .output_scroll
                        .saturating_sub((-delta) as u16)
                } else {
                    new_model
                        .prompt_output
                        .output_scroll
                        .saturating_add(delta as u16)
                };
                new_model.prompt_output.output_scroll = new_scroll;
                (new_model, Cmd::None)
            }

            Message::CycleStatisticsView(direction) => {
                new_model.statistics.view = if direction > 0 {
                    new_model.statistics.view.next()
                } else {
                    new_model.statistics.view.prev()
                };
                new_model.statistics.scroll = 0;
                new_model.status_message =
                    format!("Switched to {} view", new_model.statistics.view.as_str());
                (new_model, Cmd::None)
            }

            Message::ScrollStatistics(delta) => {
                let new_scroll = if delta < 0 {
                    new_model.statistics.scroll.saturating_sub((-delta) as u16)
                } else {
                    new_model.statistics.scroll.saturating_add(delta as u16)
                };
                new_model.statistics.scroll = new_scroll;
                (new_model, Cmd::None)
            }

            Message::SaveTemplate(filename) => {
                let content = new_model.template.get_template_content().to_string();
                let cmd = Cmd::SaveTemplate {
                    filename: filename.clone(),
                    content,
                };
                new_model.status_message = "Saving template...".to_string();
                (new_model, cmd)
            }

            Message::ReloadTemplate => {
                new_model.template.editor = crate::model::template::EditorState::default();
                new_model.template.sync_variables_with_template();
                new_model.status_message = "Reloaded template".to_string();
                (new_model, Cmd::None)
            }

            Message::LoadTemplate => {
                let result = new_model.template.load_selected_template();
                match result {
                    Ok(template_name) => {
                        new_model.template.sync_variables_with_template();
                        new_model.status_message = format!("Loaded template: {}", template_name);
                    }
                    Err(e) => {
                        new_model.status_message = format!("Failed to load template: {}", e);
                    }
                }
                (new_model, Cmd::None)
            }

            Message::RefreshTemplates => {
                new_model.template.picker.refresh();
                new_model.status_message = "Templates refreshed".to_string();
                (new_model, Cmd::None)
            }

            Message::SetTemplateFocus(focus, mode) => {
                new_model.template.set_focus(focus);
                new_model.template.set_focus_mode(mode);
                if mode == crate::model::template::FocusMode::EditingVariable {
                    new_model
                        .template
                        .variables
                        .move_to_first_missing_variable();
                }
                new_model.status_message = format!("Template focus: {:?} ({:?})", focus, mode);
                (new_model, Cmd::None)
            }

            Message::SetTemplateFocusMode(mode) => {
                new_model.template.set_focus_mode(mode);
                new_model.status_message = format!("Template mode: {:?}", mode);
                (new_model, Cmd::None)
            }

            Message::TemplateEditorInput(key) => {
                new_model.template.editor.editor.input(key);
                new_model.template.editor.sync_content_from_textarea();
                new_model.template.editor.validate_template();
                new_model.template.sync_variables_with_template();
                (new_model, Cmd::None)
            }

            Message::TemplatePickerMove(delta) => {
                if delta > 0 {
                    new_model.template.picker.move_cursor_down();
                } else {
                    new_model.template.picker.move_cursor_up();
                }
                (new_model, Cmd::None)
            }

            Message::VariableStartEditing(var_name) => {
                new_model.template.variables.editing_variable = Some(var_name.clone());
                new_model.template.variables.show_variable_input = true;
                new_model.template.variables.variable_input_content.clear();
                new_model.status_message = format!("Editing variable: {}", var_name);
                (new_model, Cmd::None)
            }

            Message::VariableInputChar(c) => {
                new_model.template.variables.add_char_to_input(c);
                (new_model, Cmd::None)
            }

            Message::VariableInputBackspace => {
                new_model.template.variables.remove_char_from_input();
                (new_model, Cmd::None)
            }

            Message::VariableInputEnter => {
                if let Some((var_name, value)) = new_model.template.variables.finish_editing() {
                    new_model.status_message = format!("Set {} = {}", var_name, value);
                    new_model.template.sync_variables_with_template();
                }
                (new_model, Cmd::None)
            }

            Message::VariableInputCancel => {
                new_model.template.variables.cancel_editing();
                new_model.status_message = "Cancelled variable editing".to_string();
                (new_model, Cmd::None)
            }

            Message::VariableNavigateUp => {
                if new_model.template.variables.cursor > 0 {
                    new_model.template.variables.cursor -= 1;
                }
                (new_model, Cmd::None)
            }

            Message::VariableNavigateDown => {
                let variables = new_model.template.get_organized_variables();
                if new_model.template.variables.cursor < variables.len().saturating_sub(1) {
                    new_model.template.variables.cursor += 1;
                }
                (new_model, Cmd::None)
            }
        }
    }
}
