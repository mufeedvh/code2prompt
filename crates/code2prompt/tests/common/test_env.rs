//! Test environment types and utilities

#![allow(dead_code)]

use assert_cmd::Command;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tempfile::TempDir;

/// Basic test environment with temporary directory and output file
pub struct BasicTestEnv {
    pub dir: TempDir,
    output_file: String,
}

impl BasicTestEnv {
    pub fn new() -> Self {
        super::init_logger();
        let dir = tempfile::tempdir().unwrap();
        let output_file = dir.path().join("output.txt").to_str().unwrap().to_string();
        BasicTestEnv { dir, output_file }
    }

    pub fn command(&self) -> Command {
        let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("code2prompt");
        cmd.arg(self.dir.path().to_str().unwrap())
            .arg("--output-file")
            .arg(&self.output_file)
            .arg("--no-clipboard");
        cmd
    }

    pub fn read_output(&self) -> String {
        let file_path = self.dir.path().join("output.txt");
        std::fs::read_to_string(&file_path)
            .unwrap_or_else(|_| panic!("Failed to read output file: {:?}", file_path))
    }
}

/// Git-enabled test environment
pub struct GitTestEnv {
    pub dir: TempDir,
    output_file: String,
}

impl GitTestEnv {
    pub fn new() -> Self {
        super::init_logger();
        let dir = tempfile::tempdir().unwrap();
        let _repo = git2::Repository::init(dir.path()).expect("Failed to initialize repository");
        let output_file = dir.path().join("output.txt").to_str().unwrap().to_string();
        GitTestEnv { dir, output_file }
    }

    pub fn command(&self) -> Command {
        let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("code2prompt");
        cmd.arg(self.dir.path().to_str().unwrap())
            .arg("--output-file")
            .arg(&self.output_file)
            .arg("--no-clipboard");
        cmd
    }

    pub fn read_output(&self) -> String {
        let file_path = self.dir.path().join("output.txt");
        std::fs::read_to_string(&file_path)
            .unwrap_or_else(|_| panic!("Failed to read output file: {:?}", file_path))
    }
}

/// Simple test environment for stdout tests
pub struct StdoutTestEnv {
    pub dir: TempDir,
}

impl StdoutTestEnv {
    pub fn new() -> Self {
        super::init_logger();
        let dir = tempfile::tempdir().unwrap();
        StdoutTestEnv { dir }
    }

    pub fn path(&self) -> &str {
        self.dir.path().to_str().unwrap()
    }
}

/// Template test environment
pub struct TemplateTestEnv {
    pub dir: TempDir,
    output_file: std::path::PathBuf,
}

impl TemplateTestEnv {
    pub fn new() -> Self {
        super::init_logger();
        let dir = tempfile::tempdir().unwrap();
        let output_file = dir.path().join("output.txt");
        TemplateTestEnv { dir, output_file }
    }

    pub fn command(&self) -> Command {
        let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("code2prompt");
        cmd.arg(self.dir.path().to_str().unwrap())
            .arg("--output-file")
            .arg(self.output_file.to_str().unwrap())
            .arg("--no-clipboard");
        cmd
    }

    pub fn read_output(&self) -> String {
        std::fs::read_to_string(&self.output_file)
            .unwrap_or_else(|_| panic!("Failed to read output file: {:?}", self.output_file))
    }

    pub fn output_file_exists(&self) -> bool {
        self.output_file.exists()
    }
}

/// Utility functions
pub fn create_temp_file(dir: &Path, name: &str, content: &str) -> std::path::PathBuf {
    let file_path = dir.join(name);
    let parent_dir = file_path.parent().unwrap();
    fs::create_dir_all(parent_dir)
        .unwrap_or_else(|_| panic!("Failed to create directory: {:?}", parent_dir));
    let mut file = File::create(&file_path)
        .unwrap_or_else(|_| panic!("Failed to create temp file: {:?}", file_path));
    writeln!(file, "{}", content)
        .unwrap_or_else(|_| panic!("Failed to write to temp file: {:?}", file_path));
    file_path
}
