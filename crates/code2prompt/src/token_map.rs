use colored::*;
use num_format::{SystemLocale, ToFormattedString};
use std::collections::HashMap;

#[derive(Debug)]
pub struct TokenMapEntry {
    pub path: String,
    pub tokens: usize,
    pub percentage: f64,
    pub is_directory: bool,
    pub depth: usize,
}

pub fn generate_token_map(files: &[serde_json::Value], total_tokens: usize) -> Vec<TokenMapEntry> {
    let mut entries = Vec::new();
    let mut dir_tokens: HashMap<String, usize> = HashMap::new();
    
    // Collect token counts per file
    for file in files {
        if let (Some(path), Some(tokens)) = (
            file.get("path").and_then(|p| p.as_str()),
            file.get("token_count").and_then(|t| t.as_u64()),
        ) {
            let tokens = tokens as usize;
            let percentage = (tokens as f64 / total_tokens as f64) * 100.0;
            let depth = path.matches('/').count();
            
            entries.push(TokenMapEntry {
                path: path.to_string(),
                tokens,
                percentage,
                is_directory: false,
                depth,
            });
            
            // Aggregate by parent directories
            let path_parts: Vec<&str> = path.split('/').collect();
            for i in 1..path_parts.len() {
                let dir_path = path_parts[..i].join("/");
                *dir_tokens.entry(dir_path).or_insert(0) += tokens;
            }
        }
    }
    
    // Add directory entries
    for (dir_path, tokens) in dir_tokens {
        let percentage = (tokens as f64 / total_tokens as f64) * 100.0;
        let depth = dir_path.matches('/').count();
        entries.push(TokenMapEntry {
            path: dir_path,
            tokens,
            percentage,
            is_directory: true,
            depth,
        });
    }
    
    // Sort by token count descending
    entries.sort_by(|a, b| b.tokens.cmp(&a.tokens));
    
    entries
}

pub fn display_token_map(entries: &[TokenMapEntry], total_tokens: usize) {
    let locale = SystemLocale::default().unwrap();
    let terminal_width = 120; // Conservative terminal width
    
    println!("\n{}", "╔═ Token Distribution Map ═╗".bold().cyan());
    println!("{}", "═".repeat(terminal_width).cyan());
    
    // Display total tokens
    println!(
        "\n{}: {} tokens\n",
        "Total".bold().white(),
        total_tokens.to_formatted_string(&locale).green().bold()
    );
    
    // Group entries by depth for better visualization
    let mut displayed_paths = std::collections::HashSet::new();
    let mut display_entries = Vec::new();
    
    // First, add top-level directories and significant files
    for entry in entries.iter() {
        if entry.percentage >= 0.5 || (entry.is_directory && entry.depth <= 2) {
            display_entries.push(entry);
            displayed_paths.insert(&entry.path);
            
            if display_entries.len() >= 40 {
                break;
            }
        }
    }
    
    // Sort display entries by path for hierarchical display
    display_entries.sort_by(|a, b| a.path.cmp(&b.path));
    
    // Calculate maximum path length for display
    let max_path_len = display_entries
        .iter()
        .map(|e| e.path.len() + e.depth * 2) // Account for indentation
        .max()
        .unwrap_or(20)
        .min(70);
    
    // Calculate bar width
    let bar_width = terminal_width.saturating_sub(max_path_len + 30); // Reserve space for formatting
    
    for entry in display_entries {
        // Create indentation based on depth
        let indent = "  ".repeat(entry.depth);
        let path_parts: Vec<&str> = entry.path.split('/').collect();
        let display_name = path_parts.last().copied().unwrap_or(&entry.path);
        
        // Format the path with indentation
        let indented_path = format!("{}{}", indent, display_name);
        let truncated_path = if indented_path.len() > max_path_len {
            format!("...{}", &indented_path[indented_path.len() - max_path_len + 3..])
        } else {
            indented_path
        };
        
        // Calculate bar length (with minimum of 1 if percentage > 0)
        let bar_length = if entry.percentage > 0.0 {
            ((entry.percentage / 100.0) * bar_width as f64).max(1.0) as usize
        } else {
            0
        };
        
        // Create gradient bar based on file size
        let bar_char = if entry.percentage > 50.0 {
            "█"
        } else if entry.percentage > 25.0 {
            "▓"
        } else if entry.percentage > 10.0 {
            "▒"
        } else {
            "░"
        };
        let bar = bar_char.repeat(bar_length);
        
        // Color based on percentage
        let colored_bar = if entry.percentage > 30.0 {
            bar.bright_red()
        } else if entry.percentage > 15.0 {
            bar.red()
        } else if entry.percentage > 8.0 {
            bar.yellow()
        } else if entry.percentage > 4.0 {
            bar.green()
        } else if entry.percentage > 1.0 {
            bar.cyan()
        } else {
            bar.blue()
        };
        
        // Format tokens with locale
        let formatted_tokens = if entry.tokens > 1_000_000 {
            format!("{:.1}M", entry.tokens as f64 / 1_000_000.0)
        } else if entry.tokens > 1_000 {
            format!("{:.1}K", entry.tokens as f64 / 1_000.0)
        } else {
            entry.tokens.to_string()
        };
        
        // Directory or file indicator
        let path_display = if entry.is_directory {
            format!("{}/", truncated_path).bold().white()
        } else {
            truncated_path.normal()
        };
        
        // Size indicator
        let size_indicator = if entry.percentage > 10.0 {
            " ⚠".red()
        } else if entry.percentage > 5.0 {
            " ▲".yellow() 
        } else {
            "".normal()
        };
        
        println!(
            "{:<width$} │{:>8} {:>6.1}%{} │ {}",
            path_display,
            formatted_tokens,
            entry.percentage,
            size_indicator,
            colored_bar,
            width = max_path_len
        );
    }
    
    println!("\n{}", "═".repeat(terminal_width).cyan());
    println!("{}", "Legend: ".bold());
    println!("  {} Large files (>30%)", "█".bright_red());
    println!("  {} Medium files (8-30%)", "▓".yellow());
    println!("  {} Small files (<8%)", "░".cyan());
    println!("  {} Warning - Large token count", "⚠".red());
    println!("{}", "═".repeat(terminal_width).cyan());
}