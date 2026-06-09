use code2prompt_core::path::FileEntry;
use std::path::Path;
use code2prompt_core::analysis::{CodebaseAnalysis, TokenMapOptions};
use code2prompt_core::path::EntryMetadata;

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_file(path: &str, tokens: usize, is_dir: bool) -> FileEntry {
        FileEntry {
            path: path.to_string(),
            extension: if is_dir {
                String::new()
            } else {
                Path::new(path)
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string()
            },
            code: String::new(),
            token_count: tokens,
            metadata: EntryMetadata {
                is_dir,
                is_symlink: false,
            },
            mod_time: None,
        }
    }

    #[test]
    fn test_simple_tree_building() {
        let files = vec![
            mock_file("src/main.rs", 100, false),
            mock_file("src/lib.rs", 200, false),
        ];

        let analysis = CodebaseAnalysis::new(&files, 300);
        let entries = analysis.token_map(TokenMapOptions {
            max_lines: 100,
            min_percent: 0.0,
        });

        // Should have: src (dir), src/main.rs, src/lib.rs
        assert!(entries.len() >= 3, "Expected at least 3 entries, got {}", entries.len());
        
        // Find the src directory entry
        let src_entry = entries.iter().find(|e| e.name == "src");
        assert!(src_entry.is_some(), "Should have 'src' directory");
        assert_eq!(src_entry.unwrap().tokens, 300, "src should have 300 tokens");
    }

    #[test]
    fn test_nested_directory_structure() {
        let files = vec![
            mock_file("website/public/assets/logo.svg", 100, false),
            mock_file("website/public/favicon.ico", 50, false),
        ];

        let analysis = CodebaseAnalysis::new(&files, 150);
        let entries = analysis.token_map(TokenMapOptions {
            max_lines: 100,
            min_percent: 0.0,
        });

        // Verify hierarchy: website -> public -> assets -> logo.svg
        let website = entries.iter().find(|e| e.name == "website");
        assert!(website.is_some(), "Should have 'website' at root");
        assert_eq!(website.unwrap().depth, 0, "website should be at depth 0");

        let public = entries.iter().find(|e| e.name == "public");
        assert!(public.is_some(), "Should have 'public'");
        assert_eq!(public.unwrap().depth, 1, "public should be at depth 1");

        let assets = entries.iter().find(|e| e.name == "assets");
        assert!(assets.is_some(), "Should have 'assets'");
        assert_eq!(assets.unwrap().depth, 2, "assets should be at depth 2");

        let logo = entries.iter().find(|e| e.name == "logo.svg");
        assert!(logo.is_some(), "Should have 'logo.svg'");
        assert_eq!(logo.unwrap().depth, 3, "logo.svg should be at depth 3");
    }

    #[test]
    fn test_token_accumulation() {
        let files = vec![
            mock_file("src/a.rs", 100, false),
            mock_file("src/b.rs", 200, false),
            mock_file("src/sub/c.rs", 50, false),
        ];

        let analysis = CodebaseAnalysis::new(&files, 350);
        let entries = analysis.token_map(TokenMapOptions {
            max_lines: 100,
            min_percent: 0.0,
        });

        let src = entries.iter().find(|e| e.name == "src");
        assert!(src.is_some());
        assert_eq!(src.unwrap().tokens, 350, "src should accumulate all tokens");

        let sub = entries.iter().find(|e| e.name == "sub");
        assert!(sub.is_some());
        assert_eq!(sub.unwrap().tokens, 50, "sub should have 50 tokens");
    }

    #[test]
    fn test_by_extension() {
        let files = vec![
            mock_file("a.rs", 100, false),
            mock_file("b.rs", 200, false),
            mock_file("c.js", 50, false),
            mock_file("d.js", 75, false),
        ];

        let analysis = CodebaseAnalysis::new(&files, 425);
        let ext_stats = analysis.by_extension();

        assert_eq!(ext_stats.len(), 2, "Should have 2 extensions");
        
        // Should be sorted by tokens descending
        assert_eq!(ext_stats[0].extension, "rs");
        assert_eq!(ext_stats[0].tokens, 300);
        assert_eq!(ext_stats[0].file_count, 2);

        assert_eq!(ext_stats[1].extension, "js");
        assert_eq!(ext_stats[1].tokens, 125);
        assert_eq!(ext_stats[1].file_count, 2);
    }

    #[test]
    fn test_filtering_by_max_lines() {
        let files = vec![
            mock_file("a.rs", 1000, false),
            mock_file("b.rs", 500, false),
            mock_file("c.rs", 200, false),
            mock_file("d.rs", 100, false),
        ];

        let analysis = CodebaseAnalysis::new(&files, 1800);
        let entries = analysis.token_map(TokenMapOptions {
            max_lines: 2, // Only show top 2
            min_percent: 0.0,
        });

        // Should have top 2 files + possibly "other files"
        let file_entries: Vec<_> = entries.iter().filter(|e| !e.metadata.is_dir).collect();
        assert!(file_entries.len() <= 3, "Should have at most 3 file entries (2 + other)");
    }

    #[test]
    fn test_filtering_by_min_percent() {
        let files = vec![
            mock_file("big.rs", 900, false),
            mock_file("small.rs", 50, false), // Only 5.5% - should be filtered
        ];

        let analysis = CodebaseAnalysis::new(&files, 950);
        let entries = analysis.token_map(TokenMapOptions {
            max_lines: 100,
            min_percent: 10.0, // 10% minimum
        });

        let big = entries.iter().find(|e| e.name == "big.rs");
        assert!(big.is_some(), "big.rs should be included");

        let small = entries.iter().find(|e| e.name == "small.rs");
        assert!(small.is_none(), "small.rs should be filtered out");
    }

    #[test]
    fn test_realistic_project_structure() {
        let files = vec![
            mock_file("crates/code2prompt/src/main.rs", 5000, false),
            mock_file("crates/code2prompt/src/args.rs", 3000, false),
            mock_file("crates/code2prompt-core/src/analysis.rs", 2000, false),
            mock_file("website/src/index.html", 1000, false),
            mock_file("website/public/favicon.ico", 500, false),
            mock_file("README.md", 200, false),
        ];

        let total = files.iter().map(|f| f.token_count).sum();
        let analysis = CodebaseAnalysis::new(&files, total);
        
        // Test with permissive settings
        let entries = analysis.token_map(TokenMapOptions {
            max_lines: 50,
            min_percent: 0.0,
        });

        println!("\n=== Token Map Entries ===");
        for entry in &entries {
            println!("Depth {}: {} ({} tokens, {}%)", 
                entry.depth, entry.name, entry.tokens, entry.percentage);
        }

        // Should have the directory structure
        assert!(!entries.is_empty(), "Should have entries");
        
        // Check that we have the crates directory
        let crates_dir = entries.iter().find(|e| e.name == "crates");
        assert!(crates_dir.is_some(), "Should have 'crates' directory");
        
        // Check that we have nested structure
        let has_depth_2 = entries.iter().any(|e| e.depth >= 2);
        assert!(has_depth_2, "Should have nested structure with depth >= 2");
    }
}
