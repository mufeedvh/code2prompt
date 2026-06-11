//! View layer for the TUI application.
//!
//! This module contains all the formatting and display logic that was previously
//! mixed into the Model and widgets. It provides pure functions that take data
//! and return formatted strings or display structures.

pub mod formatters;

pub use formatters::*;
