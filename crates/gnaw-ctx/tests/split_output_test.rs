//! Integration tests for --split-output (auto-splitting by token budget).

mod common;

use common::fixtures::*;
use common::*;
use rstest::*;
use std::path::PathBuf;

/// Collect every `output.partN.txt` the binary produced, in order, stopping at
/// the first gap. Returns (path, contents) pairs.
fn collect_parts(dir: &std::path::Path) -> Vec<(PathBuf, String)> {
    let mut parts = Vec::new();
    let mut n = 1;
    loop {
        let path = dir.join(format!("output.part{}.txt", n));
        match std::fs::read_to_string(&path) {
            Ok(contents) => {
                parts.push((path, contents));
                n += 1;
            }
            Err(_) => break,
        }
    }
    parts
}

/// A small budget over 12 files must yield multiple parts, every part must be
/// non-empty, and the combined single-file output must NOT be written.
#[rstest]
fn split_produces_multiple_nonempty_parts(basic_test_env: BasicTestEnv) {
    basic_test_env
        .command()
        .arg("--split-size")
        .arg("1") // forces each file into its own part regardless of tokenizer
        .assert()
        .success();

    let parts = collect_parts(basic_test_env.dir.path());
    assert!(
        parts.len() >= 2,
        "expected the output to be split into multiple parts, got {}",
        parts.len()
    );
    for (path, contents) in &parts {
        assert!(!contents.trim().is_empty(), "part {:?} was empty", path);
    }

    // Split mode is exclusive: the combined file must never be written.
    assert!(
        !basic_test_env.dir.path().join("output.txt").exists(),
        "combined output.txt should not exist in split mode"
    );
}

/// No content is lost: every source file's content appears across the parts.
#[rstest]
fn split_preserves_all_content(basic_test_env: BasicTestEnv) {
    basic_test_env
        .command()
        .arg("--split-size")
        .arg("1")
        .assert()
        .success();

    let combined: String = collect_parts(basic_test_env.dir.path())
        .into_iter()
        .map(|(_, c)| c)
        .collect::<Vec<_>>()
        .join("\n");

    for needle in [
        "content foo.py",
        "content bar.py",
        "content baz.py",
        "CONTENT FOO.PY",
        "CONTENT BAR.PY",
        "CONTENT BAZ.PY",
    ] {
        assert!(
            combined.contains(needle),
            "expected '{}' somewhere across the split parts",
            needle
        );
    }
}

/// --split-size combined with --clipboard is rejected at parse time
/// (clap `conflicts_with`).
#[rstest]
fn split_conflicts_with_clipboard(basic_test_env: BasicTestEnv) {
    basic_test_env
        .command()
        .arg("--split-size")
        .arg("100")
        .arg("--clipboard")
        .assert()
        .failure();
}
