//! Widget components for the TUI interface.
//!
//! This module contains all the widget implementations using Ratatui's native widget system.
//! Each widget is responsible for rendering a specific part of the UI and managing its own state.

pub mod file_selection;
pub mod output;
pub mod settings;
pub mod statistics_by_extension;
pub mod statistics_overview;
pub mod statistics_token_map;
pub mod template;

pub use file_selection::FileSelectionWidget;
pub use output::OutputWidget;
pub use settings::SettingsWidget;
pub use statistics_by_extension::StatisticsByExtensionWidget;
pub use statistics_overview::StatisticsOverviewWidget;
pub use statistics_token_map::StatisticsTokenMapWidget;
pub use template::TemplateWidget;
// Re-export TemplateState from model
pub use crate::model::template::TemplateState;
