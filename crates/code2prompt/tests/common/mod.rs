//! Common test utilities and fixtures for code2prompt integration tests
//!
//! This module provides reusable fixtures and utilities to reduce code duplication
//! across integration tests using rstest.

pub mod fixtures;
pub mod test_env;

pub use test_env::*;

use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize logger for tests (called once)
pub fn init_logger() {
    INIT.call_once(|| {
        env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init()
            .expect("Failed to initialize logger");
    });
}
