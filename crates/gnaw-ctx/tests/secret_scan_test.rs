//! End-to-end tests for the secret scrubber (--secret-scan).

mod common;

use common::*;
use predicates::prelude::*;
use predicates::str::contains;
use rstest::*;

/// Exactly 36 trailing chars → matches the github-pat rule; all-distinct → high entropy.
const FAKE_PAT: &str = "ghp_abcdefghijklmnopqrstuvwxyz0123456789";
/// AWS's documentation key — on the builtin allowlist, must never be redacted.
const AWS_EXAMPLE: &str = "AKIAIOSFODNN7EXAMPLE";

fn env_with_secret(content: &str) -> StdoutTestEnv {
    let env = StdoutTestEnv::new();
    create_temp_file(env.dir.path(), "config.py", content);
    env
}

#[rstest]
fn redact_replaces_the_secret() {
    let env = env_with_secret(&format!("API_KEY = \"{FAKE_PAT}\""));
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("gnaw");
    cmd.arg(env.path())
        .args(["-O", "-", "--no-clipboard", "--secret-scan", "redact"])
        .assert()
        .success()
        .stdout(contains("[REDACTED: github-pat]"))
        .stdout(contains(FAKE_PAT).not()); // raw secret must be gone
}

#[rstest]
fn off_passes_the_secret_through() {
    let env = env_with_secret(&format!("API_KEY = \"{FAKE_PAT}\""));
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("gnaw");
    cmd.arg(env.path())
        .args(["-O", "-", "--no-clipboard", "--secret-scan", "off"])
        .assert()
        .success()
        .stdout(contains(FAKE_PAT));
}

#[rstest]
fn warn_keeps_content_but_reports_on_stderr() {
    let env = env_with_secret(&format!("API_KEY = \"{FAKE_PAT}\""));
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("gnaw");
    cmd.arg(env.path())
        .args(["-O", "-", "--no-clipboard", "--secret-scan", "warn"])
        .assert()
        .success()
        .stdout(contains(FAKE_PAT)) // content untouched
        .stderr(contains("github-pat")); // but flagged — NEEDS findings plumbing
}

#[rstest]
fn block_fails_loudly() {
    let env = env_with_secret(&format!("API_KEY = \"{FAKE_PAT}\""));
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("gnaw");
    cmd.arg(env.path())
        .args(["-O", "-", "--no-clipboard", "--secret-scan", "block"])
        .assert()
        .failure() // NEEDS findings plumbing + bail in generate_prompt
        .stderr(contains("github-pat"));
}

#[rstest]
fn allowlisted_example_key_is_not_redacted() {
    let env = env_with_secret(&format!("aws_key = \"{AWS_EXAMPLE}\""));
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("gnaw");
    cmd.arg(env.path())
        .args(["-O", "-", "--no-clipboard", "--secret-scan", "redact"])
        .assert()
        .success()
        .stdout(contains(AWS_EXAMPLE))
        .stdout(contains("[REDACTED").not());
}
