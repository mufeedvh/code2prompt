//! # Code2Prompt Core Library
//!
//! Pure Rust library for consolidating multi-file repositories
//! and source trees into a single, structured, LLM-optimized prompt payload.
//! Provides file filtering, template processing, and git integration.
//!
//! ---
//!
//! ## 📘 Documentation & Integration Guides
//!
//! For high-level concepts (Diataxis framework), step-by-step onboarding, and workflows:
//! * **Main Portal:** [code2prompt.dev](https://code2prompt.dev)
//! * **Tutorials & Guides:** [code2prompt.dev/docs/welcome](https://code2prompt.dev/docs/welcome)
//!
//! ## Quick Start
//!
//! ```
//! use code2prompt_core::{Code2PromptConfig, Code2PromptSession};
//! 
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # // Create a temporary directory for testing
//! # let temp_dir = std::env::temp_dir().join("code2prompt_test");
//! # std::fs::create_dir_all(&temp_dir)?;
//! # std::fs::write(temp_dir.join("test.rs"), "fn main() {}")?;
//! let config = Code2PromptConfig::builder()
//!     .path(&temp_dir)
//!     .build()?;
//! 
//! let mut session = Code2PromptSession::new(config);
//! let output = session.generate_prompt()?;
//! println!("Generated {} tokens", output.token_count);
//! # // Cleanup
//! # std::fs::remove_dir_all(&temp_dir).ok();
//! # Ok(())
//! # }
//! ```
//!
//! ## Main Components
//!
//! - [`Code2PromptConfig`] - Configuration and builder
//! - [`Code2PromptSession`] - Main workflow orchestrator
//! - [`template`] - Template processing and rendering
//! - [`filter`] - File filtering and selection
//! - [`git`] - Git repository integration

pub mod analysis;
pub mod builtin_templates;
pub mod configuration;
pub mod entity_map;
pub mod file_processor;
pub mod filter;
pub mod git;
pub mod path;
pub mod selection;
pub mod session;
pub mod sort;
pub mod template;
pub mod tokenizer;
pub mod util;

pub use configuration::{Code2PromptConfig, Code2PromptConfigBuilder};
pub use session::Code2PromptSession;
