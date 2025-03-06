use anyhow::{Context, Result};

#[cfg(not(target_os = "linux"))]
/// Copies the provided text to the system clipboard.
///
/// This is a simple, one-shot copy operation suitable for non-Linux platforms
/// or scenarios where maintaining the clipboard content is not required.
///
/// # Arguments
///
/// * `text` - The text content to be copied.
///
/// # Returns
///
/// * `Result<()>` - Returns Ok on success, or an error if the clipboard could not be accessed.
pub fn copy_text_to_clipboard(rendered: &str) -> Result<()> {
    use arboard::Clipboard;
    match Clipboard::new() {
        Ok(mut clipboard) => {
            clipboard
                .set_text(rendered.to_string())
                .context("Failed to copy to clipboard")?;
            Ok(())
        }
        Err(e) => Err(anyhow::anyhow!("Failed to initialize clipboard: {}", e)),
    }
}

#[cfg(target_os = "linux")]
/// Entry point for the clipboard daemon process on Linux.
///
/// This function reads clipboard content from its standard input, sets it as the system clipboard,
/// and then waits to serve clipboard requests. This ensures that the clipboard content remains available
/// even after the main application exits. The daemon will exit automatically once the clipboard is overwritten.
///
/// # Returns
///
/// * `Result<()>` - Returns Ok on success or an error if clipboard operations fail.
pub fn serve_clipboard_daemon() -> Result<()> {
    use arboard::{Clipboard, LinuxClipboardKind, SetExtLinux};
    use std::io::Read;
    // Read content from stdin
    let mut content_from_stdin = String::new();
    std::io::stdin()
        .read_to_string(&mut content_from_stdin)
        .context("Failed to read from stdin")?;
    // Initialize the clipboard
    let mut clipboard = Clipboard::new().context("Failed to initialize clipboard")?;
    // Explicitly set the clipboard selection to Clipboard (not Primary)
    clipboard
        .set()
        .clipboard(LinuxClipboardKind::Clipboard)
        .wait()
        .text(content_from_stdin)
        .context("Failed to set clipboard content")?;
    Ok(())
}

#[cfg(target_os = "linux")]
/// Spawns a daemon process to maintain clipboard content on Linux.
///
/// On Linux (Wayland/X11), the clipboard content is owned by the process that defined it.
/// If the main application exits, the clipboard would be cleared.
/// To avoid this, this function spawns a new process that will run in the background
/// (daemon) and maintain the clipboard content until it is overwritten by a new copy.
///
/// # Arguments
///
/// * `text` - The text to be served by the daemon process.
///
/// # Returns
///
/// * `Result<()>` - Returns Ok if the daemon process was spawned and the content was sent successfully,
///   or an error if the process could not be launched or written to.
pub fn spawn_clipboard_daemon(content: &str) -> Result<()> {
    use colored::*;
    use log::info;
    use std::process::{Command, Stdio};

    // ~~~ Setting up the command to run the daemon ~~~
    let current_exe: std::path::PathBuf =
        std::env::current_exe().context("Failed to get current executable path")?;
    let mut args: Vec<String> = std::env::args().collect();
    args.push("--clipboard-daemon".to_string());

    // ~~~ Spawn the clipboard daemon process ~~~
    let mut child = Command::new(current_exe)
        .args(&args[1..])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to launch clipboard daemon process")?;

    // ~~~ Write the content to the daemon's standard input ~~~
    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin
            .write_all(content.as_bytes())
            .context("Failed to write content to clipboard daemon process")?;
    }
    info!("Clipboard daemon launched successfully");
    println!(
        "{}{}{} {}",
        "[".bold().white(),
        "✓".bold().green(),
        "]".bold().white(),
        "Copied to clipboard successfully.".green()
    );
    Ok(())
}
