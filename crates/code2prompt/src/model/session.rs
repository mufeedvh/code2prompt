//! Session state management for the TUI application.
//!
//! This module contains the session state and related functionality,
//! including the Code2PromptSession and configuration management.

use code2prompt_core::configuration::Code2PromptConfig;
use code2prompt_core::session::Code2PromptSession;

/// Session state containing the core Code2PromptSession
#[derive(Debug, Clone)]
pub struct SessionState {
    pub session: Code2PromptSession,
}

impl Default for SessionState {
    fn default() -> Self {
        let config = Code2PromptConfig::default();
        let session = Code2PromptSession::new(config);
        SessionState { session }
    }
}

impl SessionState {
    /// Create a new session state with the provided session
    pub fn new(session: Code2PromptSession) -> Self {
        SessionState { session }
    }
}
