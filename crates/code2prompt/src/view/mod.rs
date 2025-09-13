//! View layer for the TUI application.
//!
//! This module contains all the formatting and display logic that was previously
//! mixed into the Model and widgets. It provides pure functions that take data
//! and return formatted strings or display structures.

pub mod formatters;
pub mod utils;

pub use formatters::*;
// Note: utils functions will be used in future refactoring steps
#[allow(unused_imports)]
pub use utils::*;
