//! Startup banner for the gnaw CLI/TUI.
//!
//! The banner art is shared between two rendering paths that use *different*
//! color systems:
//!
//! * **CLI** — written to stderr, colored with the `colored` crate
//!   (`.truecolor`). stderr only, never stdout: stdout is reserved for the
//!   generated prompt payload, so `gnaw . | pbcopy` and the byte-stable stdout
//!   snapshot tests stay clean.
//! * **TUI** — drawn by ratatui as a one-time splash, colored with
//!   `ratatui::style::Color::Rgb`.
//!
//! Because the two color systems are incompatible, the art itself is a plain
//! `&str` const and each path applies its own color.

/// The shared ASCII art mark (`ansi_shadow` figlet font). No color codes —
/// each renderer colors it for its own backend.
pub const BANNER_ART: &str = r#" ██████╗ ███╗   ██╗ █████╗ ██╗    ██╗
██╔════╝ ████╗  ██║██╔══██╗██║    ██║
██║  ███╗██╔██╗ ██║███████║██║ █╗ ██║
██║   ██║██║╚██╗██║██╔══██║██║███╗██║
╚██████╔╝██║ ╚████║██║  ██║╚███╔███╔╝
 ╚═════╝ ╚═╝  ╚═══╝╚═╝  ╚═╝ ╚══╝╚══╝ "#;

/// Brand ember orange (rust-lang `#FF6700`). Shared so CLI and TUI match.
pub const EMBER: (u8, u8, u8) = (0xFF, 0x67, 0x00);

/// Subtitle shown under the art: version + tagline.
pub fn subtitle() -> String {
    format!("gnaw v{} · codebase → prompt", env!("CARGO_PKG_VERSION"))
}

// ----------------------------------------------------------------------------
// CLI path
// ----------------------------------------------------------------------------

use std::io::IsTerminal;

/// Whether the CLI startup banner should be shown.
///
/// `quiet` is the `--quiet` flag. We check **stderr** (not stdout): the banner
/// is diagnostic chrome that must never appear when the prompt is piped from
/// stdout, but stderr being a TTY is the right signal that a human is watching.
pub fn should_show(quiet: bool) -> bool {
    if quiet {
        return false;
    }
    if std::env::var_os("GNAW_NO_BANNER").is_some() {
        return false;
    }
    std::io::stderr().is_terminal()
}

/// Print the banner to stderr if `should_show` allows it.
///
/// `colored` already honors `NO_COLOR`, so users with it set get plain art.
pub fn print_cli(quiet: bool) {
    if !should_show(quiet) {
        return;
    }
    eprintln!("{}", render_cli());
}

/// Build the colored CLI banner string (art + subtitle). Separated from
/// `print_cli` so it is unit-testable without touching stderr.
pub fn render_cli() -> String {
    use colored::Colorize;
    let art = BANNER_ART.truecolor(EMBER.0, EMBER.1, EMBER.2);
    format!("{art}\n  {}\n", subtitle().dimmed())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quiet_suppresses_banner() {
        assert!(!should_show(true));
    }

    #[test]
    fn env_var_suppresses_banner() {
        // SAFETY: single-threaded test; set then immediately read & unset.
        unsafe { std::env::set_var("GNAW_NO_BANNER", "1") };
        assert!(!should_show(false));
        unsafe { std::env::remove_var("GNAW_NO_BANNER") };
    }

    #[test]
    fn cli_render_includes_version_and_tagline() {
        let out = render_cli();
        assert!(out.contains(env!("CARGO_PKG_VERSION")));
        assert!(out.contains("codebase"));
    }

    #[test]
    fn art_is_six_lines() {
        assert_eq!(BANNER_ART.lines().count(), 6);
    }
}
