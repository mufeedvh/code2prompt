//! Configuration file loading and management.
//!
//! This module handles loading TOML configuration files from multiple locations
//! with proper priority handling and informational messages.

use anyhow::{Context, Result};
use code2prompt_core::configuration::{OutputDestination, TomlConfig};
use colored::*;
use log::{debug, info};
use std::path::PathBuf;

/// Configuration source information
#[derive(Debug, Clone)]
pub struct ConfigSource {
    pub config: TomlConfig,
}

/// Load configuration with proper priority handling
pub fn load_config(quiet: bool) -> Result<ConfigSource> {
    // Check for local config first (.c2pconfig in current directory)
    let local_config_path = std::env::current_dir()?.join(".c2pconfig");
    if local_config_path.exists() {
        match load_config_from_file(&local_config_path) {
            Ok(config) => {
                if !quiet {
                    eprintln!(
                        "{}{}{} Using config from: {}",
                        "[".bold().white(),
                        "i".bold().blue(),
                        "]".bold().white(),
                        local_config_path.display()
                    );
                }
                info!("Loaded local config from: {}", local_config_path.display());
                return Ok(ConfigSource { config });
            }
            Err(e) => {
                debug!("Failed to load local config: {}", e);
            }
        }
    }

    // Check for global config (~/.config/code2prompt/.c2pconfig)
    if let Some(config_dir) = dirs::config_dir() {
        let global_config_path = config_dir.join("code2prompt").join(".c2pconfig");
        if global_config_path.exists() {
            match load_config_from_file(&global_config_path) {
                Ok(config) => {
                    if !quiet {
                        eprintln!(
                            "{}{}{} Using config from: {}",
                            "[".bold().white(),
                            "i".bold().blue(),
                            "]".bold().white(),
                            global_config_path.display()
                        );
                    }
                    info!(
                        "Loaded global config from: {}",
                        global_config_path.display()
                    );
                    return Ok(ConfigSource { config });
                }
                Err(e) => {
                    debug!("Failed to load global config: {}", e);
                }
            }
        }
    }

    // Use default configuration
    if !quiet {
        eprintln!(
            "{}{}{} Using default configuration",
            "[".bold().white(),
            "i".bold().blue(),
            "]".bold().white(),
        );
    }
    info!("Using default configuration");

    Ok(ConfigSource {
        config: TomlConfig::default(),
    })
}

/// Load TOML configuration from a file
fn load_config_from_file(path: &PathBuf) -> Result<TomlConfig> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;

    TomlConfig::from_toml_str(&content)
        .with_context(|| format!("Failed to parse TOML config file: {}", path.display()))
}

/// Get the default output destination from config
pub fn get_default_output_destination(config_source: &ConfigSource) -> OutputDestination {
    config_source.config.default_output.clone()
}
