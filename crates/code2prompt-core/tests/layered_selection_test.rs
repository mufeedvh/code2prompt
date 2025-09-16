//! Tests for the layered selection system (explicit overrides + glob patterns)

use code2prompt_core::configuration::Code2PromptConfig;
use code2prompt_core::filter::{build_globset, should_include_path};
use std::path::PathBuf;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explicit_include_overrides_exclude_pattern() {
        let mut config = Code2PromptConfig::default();
        config.exclude_patterns = vec!["*.rs".to_string()];
        config
            .explicit_includes
            .insert(PathBuf::from("src/main.rs"));

        let include_gs = build_globset(&config.include_patterns);
        let exclude_gs = build_globset(&config.exclude_patterns);

        // main.rs should be included despite exclude pattern
        assert!(should_include_path(
            &PathBuf::from("src/main.rs"),
            &include_gs,
            &exclude_gs,
            &config.explicit_includes,
            &config.explicit_excludes,
        ));
    }

    #[test]
    fn test_explicit_exclude_overrides_include_pattern() {
        let mut config = Code2PromptConfig::default();
        config.include_patterns = vec!["src/**".to_string()];
        config
            .explicit_excludes
            .insert(PathBuf::from("src/utils.rs"));

        let include_gs = build_globset(&config.include_patterns);
        let exclude_gs = build_globset(&config.exclude_patterns);

        // utils.rs should be excluded despite include pattern
        assert!(!should_include_path(
            &PathBuf::from("src/utils.rs"),
            &include_gs,
            &exclude_gs,
            &config.explicit_includes,
            &config.explicit_excludes,
        ));
    }

    #[test]
    fn test_ancestor_propagation_include() {
        let mut config = Code2PromptConfig::default();
        config.exclude_patterns = vec!["src/**".to_string()];
        config.explicit_includes.insert(PathBuf::from("src"));

        let include_gs = build_globset(&config.include_patterns);
        let exclude_gs = build_globset(&config.exclude_patterns);

        // Child files should be included due to ancestor explicit include
        assert!(should_include_path(
            &PathBuf::from("src/main.rs"),
            &include_gs,
            &exclude_gs,
            &config.explicit_includes,
            &config.explicit_excludes,
        ));
        assert!(should_include_path(
            &PathBuf::from("src/lib/mod.rs"),
            &include_gs,
            &exclude_gs,
            &config.explicit_includes,
            &config.explicit_excludes,
        ));
    }

    #[test]
    fn test_ancestor_propagation_exclude() {
        let mut config = Code2PromptConfig::default();
        config.include_patterns = vec!["**/*.rs".to_string()];
        config.explicit_excludes.insert(PathBuf::from("src"));

        let include_gs = build_globset(&config.include_patterns);
        let exclude_gs = build_globset(&config.exclude_patterns);

        // Child files should be excluded due to ancestor explicit exclude
        assert!(!should_include_path(
            &PathBuf::from("src/main.rs"),
            &include_gs,
            &exclude_gs,
            &config.explicit_includes,
            &config.explicit_excludes,
        ));
        assert!(!should_include_path(
            &PathBuf::from("src/lib/mod.rs"),
            &include_gs,
            &exclude_gs,
            &config.explicit_includes,
            &config.explicit_excludes,
        ));
    }

    #[test]
    fn test_explicit_exclude_takes_precedence_over_explicit_include() {
        let mut config = Code2PromptConfig::default();
        config
            .explicit_includes
            .insert(PathBuf::from("src/main.rs"));
        config
            .explicit_excludes
            .insert(PathBuf::from("src/main.rs"));

        let include_gs = build_globset(&config.include_patterns);
        let exclude_gs = build_globset(&config.exclude_patterns);

        // Explicit exclude should win over explicit include
        assert!(!should_include_path(
            &PathBuf::from("src/main.rs"),
            &include_gs,
            &exclude_gs,
            &config.explicit_includes,
            &config.explicit_excludes,
        ));
    }

    #[test]
    fn test_fallback_to_patterns_when_no_explicit() {
        let mut config = Code2PromptConfig::default();
        config.include_patterns = vec!["*.rs".to_string()];
        config.exclude_patterns = vec!["test_*.rs".to_string()];

        let include_gs = build_globset(&config.include_patterns);
        let exclude_gs = build_globset(&config.exclude_patterns);

        // Should follow normal pattern logic when no explicit overrides
        assert!(should_include_path(
            &PathBuf::from("main.rs"),
            &include_gs,
            &exclude_gs,
            &config.explicit_includes,
            &config.explicit_excludes,
        ));
        assert!(!should_include_path(
            &PathBuf::from("test_main.rs"),
            &include_gs,
            &exclude_gs,
            &config.explicit_includes,
            &config.explicit_excludes,
        ));
    }

    #[test]
    fn test_empty_patterns_include_all_when_no_explicit() {
        let config = Code2PromptConfig::default();

        let include_gs = build_globset(&config.include_patterns);
        let exclude_gs = build_globset(&config.exclude_patterns);

        // Should include everything when no patterns and no explicit overrides
        assert!(should_include_path(
            &PathBuf::from("any/file.rs"),
            &include_gs,
            &exclude_gs,
            &config.explicit_includes,
            &config.explicit_excludes,
        ));
    }
}
