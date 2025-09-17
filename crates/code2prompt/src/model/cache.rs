//! Caching system for TUI performance optimization.
//!
//! This module provides intelligent caching mechanisms to avoid expensive
//! recalculations during UI updates, particularly for file tree rendering
//! and scroll operations.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::model::DisplayFileNode;
use crate::utils::DisplayNodeWithSelection;
use code2prompt_core::session::Code2PromptSession;

/// Cache for visible nodes with intelligent invalidation
#[derive(Debug, Clone)]
pub struct VisibleNodesCache {
    /// Cached visible nodes
    nodes: Vec<DisplayNodeWithSelection>,
    /// Last search query used for computation
    last_search_query: String,
    /// Hash of the file tree structure
    last_tree_hash: u64,
    /// Hash of the selection state
    last_selection_hash: u64,
    /// Whether the cache is valid
    is_valid: bool,
}

impl Default for VisibleNodesCache {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            last_search_query: String::new(),
            last_tree_hash: 0,
            last_selection_hash: 0,
            is_valid: false,
        }
    }
}

impl VisibleNodesCache {
    /// Get cached visible nodes or compute them if cache is invalid
    pub fn get_or_compute(
        &mut self,
        file_tree_nodes: &[DisplayFileNode],
        search_query: &str,
        session: &mut Code2PromptSession,
    ) -> &Vec<DisplayNodeWithSelection> {
        let current_tree_hash = self.compute_tree_hash(file_tree_nodes);
        let current_selection_hash = self.compute_selection_hash(session);

        // Check if cache needs invalidation
        let needs_update = !self.is_valid
            || search_query != self.last_search_query
            || current_tree_hash != self.last_tree_hash
            || current_selection_hash != self.last_selection_hash;

        if needs_update {
            // Recompute visible nodes
            self.nodes = crate::utils::get_visible_nodes(file_tree_nodes, search_query, session);

            // Update cache metadata
            self.last_search_query = search_query.to_string();
            self.last_tree_hash = current_tree_hash;
            self.last_selection_hash = current_selection_hash;
            self.is_valid = true;
        }

        &self.nodes
    }

    /// Invalidate the cache (force recomputation on next access)
    pub fn invalidate(&mut self) {
        self.is_valid = false;
    }

    /// Compute hash of the file tree structure
    fn compute_tree_hash(&self, nodes: &[DisplayFileNode]) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash_nodes_recursive(nodes, &mut hasher);
        hasher.finish()
    }

    /// Recursively hash file tree nodes
    fn hash_nodes_recursive(&self, nodes: &[DisplayFileNode], hasher: &mut DefaultHasher) {
        for node in nodes {
            node.path.hash(hasher);
            node.is_expanded.hash(hasher);
            node.children_loaded.hash(hasher);
            self.hash_nodes_recursive(&node.children, hasher);
        }
    }

    /// Compute hash of the selection state
    fn compute_selection_hash(&self, session: &mut Code2PromptSession) -> u64 {
        let mut hasher = DefaultHasher::new();

        // Hash selection engine state
        if let Ok(selected_files) = session.get_selected_files() {
            for file in selected_files {
                file.hash(&mut hasher);
            }
        }

        // Hash user actions count (indicates selection changes)
        session.has_user_actions().hash(&mut hasher);

        hasher.finish()
    }
}

/// Cache for scroll-related computations
#[derive(Debug, Clone)]
pub struct ScrollCache {
    /// Last computed viewport info
    last_viewport: Option<ViewportInfo>,
    /// Last scroll position
    last_scroll_position: u16,
    /// Last cursor position
    last_cursor_position: usize,
    /// Total items count
    last_total_items: usize,
}

#[derive(Debug, Clone)]
pub struct ViewportInfo {
    /// First visible item index
    pub start_index: usize,
    /// Last visible item index (exclusive)
    pub end_index: usize,
    /// Viewport height in lines
    pub viewport_height: usize,
    /// Whether cursor is visible in viewport
    pub cursor_visible: bool,
}

impl Default for ScrollCache {
    fn default() -> Self {
        Self {
            last_viewport: None,
            last_scroll_position: 0,
            last_cursor_position: 0,
            last_total_items: 0,
        }
    }
}

impl ScrollCache {
    /// Get viewport information with caching
    pub fn get_viewport_info(
        &mut self,
        scroll_position: u16,
        cursor_position: usize,
        total_items: usize,
        viewport_height: usize,
    ) -> ViewportInfo {
        // Check if we can reuse cached viewport
        if let Some(ref cached) = self.last_viewport {
            if scroll_position == self.last_scroll_position
                && cursor_position == self.last_cursor_position
                && total_items == self.last_total_items
                && viewport_height == cached.viewport_height
            {
                return cached.clone();
            }
        }

        // Compute new viewport info
        let start_index = scroll_position as usize;
        let end_index = (start_index + viewport_height).min(total_items);
        let cursor_visible = cursor_position >= start_index && cursor_position < end_index;

        let viewport = ViewportInfo {
            start_index,
            end_index,
            viewport_height,
            cursor_visible,
        };

        // Cache the result
        self.last_viewport = Some(viewport.clone());
        self.last_scroll_position = scroll_position;
        self.last_cursor_position = cursor_position;
        self.last_total_items = total_items;

        viewport
    }

    /// Check if scroll position needs adjustment to keep cursor visible
    pub fn compute_scroll_adjustment(
        &self,
        cursor_position: usize,
        current_scroll: u16,
        viewport_height: usize,
        total_items: usize,
    ) -> Option<u16> {
        let cursor_pos = cursor_position as u16;
        let current_scroll = current_scroll;
        let viewport_end = current_scroll + viewport_height as u16;

        // Cursor above visible area - scroll up
        if cursor_pos < current_scroll {
            return Some(cursor_pos);
        }

        // Cursor below visible area - scroll down
        if cursor_pos >= viewport_end {
            let new_scroll = cursor_pos.saturating_sub(viewport_height as u16 - 1);
            let max_scroll = (total_items as u16).saturating_sub(viewport_height as u16);
            return Some(new_scroll.min(max_scroll));
        }

        // Cursor is visible, no adjustment needed
        None
    }
}

/// Performance metrics for debugging
#[derive(Debug, Default, Clone)]
pub struct PerformanceMetrics {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub last_render_time_ms: u64,
    pub avg_render_time_ms: f64,
    pub render_count: u64,
}

impl PerformanceMetrics {
    pub fn record_cache_hit(&mut self) {
        self.cache_hits += 1;
    }

    pub fn record_cache_miss(&mut self) {
        self.cache_misses += 1;
    }

    pub fn record_render_time(&mut self, time_ms: u64) {
        self.last_render_time_ms = time_ms;
        self.render_count += 1;

        // Update rolling average
        let alpha = 0.1; // Smoothing factor
        if self.render_count == 1 {
            self.avg_render_time_ms = time_ms as f64;
        } else {
            self.avg_render_time_ms =
                alpha * time_ms as f64 + (1.0 - alpha) * self.avg_render_time_ms;
        }
    }

    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }
}
