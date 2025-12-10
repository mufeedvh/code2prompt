//! Tests for binary file detection using content_inspector

use code2prompt_core::configuration::Code2PromptConfig;
use code2prompt_core::path::traverse_directory;
use std::fs;
use tempfile::TempDir;

/// Helper to create a test directory with mixed binary and text files
fn create_test_directory_with_binary() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    // Create text files
    fs::write(base_path.join("text.txt"), "This is a text file").unwrap();
    fs::write(
        base_path.join("code.rs"),
        "fn main() { println!(\"Hello\"); }",
    )
    .unwrap();
    fs::write(base_path.join("data.json"), r#"{"key": "value"}"#).unwrap();

    // Create text file with non-UTF8 encoding (GB2312)
    let mut gb2312_data = b"GB2312 test: ".to_vec();
    // Append "你好" encoded in GB2312
    // '你' is 0xC4 0xE3
    // '好' is 0xBA 0xC3
    gb2312_data.extend_from_slice(&[0xC4, 0xE3, 0xBA, 0xC3]);
    fs::write(base_path.join("chinese_gb2312.txt"), gb2312_data).unwrap();

    // Create binary files (simulated)
    // PNG header signature
    let mut png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    // Append some zeros and random high bytes to ensure it hits the binary heuristic
    png_data.extend_from_slice(&[0x00, 0x00, 0x00, 0xFF, 0xFE]);
    fs::write(base_path.join("image.png"), png_data).unwrap();

    // Random binary data
    let binary_data: Vec<u8> = (0..100).map(|i| (i * 7) as u8).collect();
    fs::write(base_path.join("binary.bin"), binary_data).unwrap();

    // JPEG header
    let mut jpeg_data = vec![0xFF, 0xD8, 0xFF, 0xE0];
    jpeg_data.extend_from_slice(&[0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00]);
    fs::write(base_path.join("photo.jpg"), jpeg_data).unwrap();

    // Compiled object file simulation (ELF header with more data)
    let mut elf_data = vec![0x7F, b'E', b'L', b'F']; // ELF magic
    // Add more binary data to make it clearly binary
    elf_data.extend_from_slice(&[0x02, 0x01, 0x01, 0x00]); // 64-bit, little endian, etc
    elf_data.extend((0..50).map(|i| (i * 13) as u8)); // More binary content
    fs::write(base_path.join("compiled.o"), elf_data).unwrap();

    temp_dir
}

#[test]
fn test_binary_files_are_skipped() {
    let temp_dir = create_test_directory_with_binary();
    let config = Code2PromptConfig::builder()
        .path(temp_dir.path().to_path_buf())
        .build()
        .unwrap();

    let (_, files) = traverse_directory(&config, None).unwrap();

    // Should only include text files, not binary files
    let file_paths: Vec<String> = files.iter().map(|f| f.path.clone()).collect();

    // Text files should be included
    assert!(file_paths.iter().any(|p| p.contains("text.txt")));
    assert!(file_paths.iter().any(|p| p.contains("code.rs")));
    assert!(file_paths.iter().any(|p| p.contains("data.json")));
    assert!(file_paths.iter().any(|p| p.contains("chinese_gb2312.txt")));

    // Binary files should be excluded
    assert!(!file_paths.iter().any(|p| p.contains("image.png")));
    assert!(!file_paths.iter().any(|p| p.contains("binary.bin")));
    assert!(!file_paths.iter().any(|p| p.contains("photo.jpg")));
    assert!(!file_paths.iter().any(|p| p.contains("compiled.o")));

    // Should have exactly 3 text files
    assert_eq!(files.len(), 4);
}

#[test]
fn test_empty_file_handling() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    // Create an empty file
    fs::write(base_path.join("empty.txt"), "").unwrap();

    let config = Code2PromptConfig::builder()
        .path(base_path.to_path_buf())
        .build()
        .unwrap();

    let (_, files) = traverse_directory(&config, None).unwrap();

    // Empty files should be excluded (existing behavior)
    assert_eq!(files.len(), 0);
}

#[test]
fn test_small_binary_file() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    // Create a very small binary file (less than 8KB)
    let small_binary: Vec<u8> = vec![0x00, 0xFF, 0x00, 0xFF, 0xFE, 0xED];
    fs::write(base_path.join("small.bin"), small_binary).unwrap();

    let config = Code2PromptConfig::builder()
        .path(base_path.to_path_buf())
        .build()
        .unwrap();

    let (_, files) = traverse_directory(&config, None).unwrap();

    // Small binary file should still be detected and excluded
    assert_eq!(files.len(), 0);
}

#[test]
fn test_text_file_with_unicode() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    // Create text file with Unicode characters
    fs::write(
        base_path.join("unicode.txt"),
        "Hello 世界 🌍 Здравствуй мир",
    )
    .unwrap();

    let config = Code2PromptConfig::builder()
        .path(base_path.to_path_buf())
        .build()
        .unwrap();

    let (_, files) = traverse_directory(&config, None).unwrap();

    // Unicode text should be detected as text and included
    assert_eq!(files.len(), 1);
    let file_path = &files[0].path;
    assert!(file_path.contains("unicode.txt"));
}

#[test]
fn test_mixed_directory_structure() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    // Create nested structure with mixed files
    fs::create_dir(base_path.join("src")).unwrap();
    fs::create_dir(base_path.join("assets")).unwrap();

    // Text files in src/
    fs::write(base_path.join("src/main.rs"), "fn main() {}").unwrap();
    fs::write(base_path.join("src/lib.rs"), "pub mod test {}").unwrap();

    // Binary files in assets/
    let png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    fs::write(base_path.join("assets/logo.png"), png_data).unwrap();

    let config = Code2PromptConfig::builder()
        .path(base_path.to_path_buf())
        .build()
        .unwrap();

    let (_, files) = traverse_directory(&config, None).unwrap();

    // Should only have 2 text files from src/
    assert_eq!(files.len(), 2);

    let file_paths: Vec<String> = files.iter().map(|f| f.path.clone()).collect();

    assert!(file_paths.iter().any(|p| p.contains("main.rs")));
    assert!(file_paths.iter().any(|p| p.contains("lib.rs")));
    assert!(!file_paths.iter().any(|p| p.contains("logo.png")));
}

#[test]
fn test_large_text_file() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    // Create a large text file (> 8KB) to test that full file is read after sample
    let large_text = "Lorem ipsum dolor sit amet. ".repeat(1000); // ~28KB
    fs::write(base_path.join("large.txt"), &large_text).unwrap();

    let config = Code2PromptConfig::builder()
        .path(base_path.to_path_buf())
        .build()
        .unwrap();

    let (_, files) = traverse_directory(&config, None).unwrap();

    // Large text file should be detected and included
    assert_eq!(files.len(), 1);

    // Verify the entire content was read (not just the sample)
    let code = &files[0].code;
    assert!(code.contains(&large_text));
}

#[test]
fn test_pdf_detection() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    // PDF file header
    let pdf_header = b"%PDF-1.4\n";
    fs::write(base_path.join("document.pdf"), pdf_header).unwrap();

    let config = Code2PromptConfig::builder()
        .path(base_path.to_path_buf())
        .build()
        .unwrap();

    let (_, files) = traverse_directory(&config, None).unwrap();

    // PDF should be detected as binary and excluded
    assert_eq!(files.len(), 0);
}

#[test]
fn test_various_text_formats() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    // Various text file formats
    fs::write(base_path.join("config.yaml"), "key: value\n").unwrap();
    fs::write(base_path.join("data.xml"), "<root><item/></root>").unwrap();
    fs::write(base_path.join("script.sh"), "#!/bin/bash\necho 'test'").unwrap();
    fs::write(base_path.join("style.css"), "body { margin: 0; }").unwrap();
    fs::write(base_path.join("page.html"), "<html><body></body></html>").unwrap();

    let config = Code2PromptConfig::builder()
        .path(base_path.to_path_buf())
        .build()
        .unwrap();

    let (_, files) = traverse_directory(&config, None).unwrap();

    // All text formats should be included
    assert_eq!(files.len(), 5);
}
