// code2prompt/crates/code2prompt/tests/template_integration_test.rs

use assert_cmd::Command;
use log::{debug, info};
use predicates::prelude::*;
use predicates::str::contains;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::sync::Once;
use tempfile::tempdir;

static INIT: Once = Once::new();

fn init_logger() {
    INIT.call_once(|| {
        env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init()
            .expect("Failed to initialize logger");
    });
}

fn create_temp_file(dir: &Path, name: &str, content: &str) -> std::path::PathBuf {
    let file_path = dir.join(name);
    let parent_dir = file_path.parent().unwrap();
    fs::create_dir_all(parent_dir).expect(&format!("Failed to create directory: {:?}", parent_dir));
    let mut file =
        File::create(&file_path).expect(&format!("Failed to create temp file: {:?}", file_path));
    writeln!(file, "{}", content).expect(&format!("Failed to write to temp file: {:?}", file_path));
    file_path
}

fn create_test_codebase(base_path: &Path) {
    // Create a simple code structure for testing templates
    let files = vec![
        (
            "src/main.rs",
            "fn main() {\n    println!(\"Hello, world!\");\n}",
        ),
        (
            "src/lib.rs",
            "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}",
        ),
        (
            "tests/test.rs",
            "#[test]\nfn test_add() {\n    assert_eq!(3, add(1, 2));\n}",
        ),
    ];

    for (file_path, content) in files {
        create_temp_file(base_path, file_path, content);
    }
    info!("Test codebase created");
}

fn read_output_file(file_path: &Path) -> String {
    fs::read_to_string(file_path).expect(&format!("Failed to read output file: {:?}", file_path))
}

mod template_tests {
    use super::*;
    use predicates::str::{ends_with, starts_with};
    use tempfile::TempDir;

    struct TestEnv {
        dir: TempDir,
        output_file: std::path::PathBuf,
    }

    impl TestEnv {
        fn new() -> Self {
            init_logger();
            let dir = tempdir().unwrap();
            create_test_codebase(dir.path());
            let output_file = dir.path().join("output.txt");
            TestEnv { dir, output_file }
        }

        fn command(&self) -> Command {
            let mut cmd =
                Command::cargo_bin("code2prompt").expect("Failed to find code2prompt binary");
            cmd.arg(&self.dir.path().to_str().unwrap())
                .arg("--output-file")
                .arg(&self.output_file.to_str().unwrap())
                .arg("--no-clipboard");
            cmd
        }

        fn read_output(&self) -> String {
            read_output_file(&self.output_file)
        }
    }

    #[test]
    fn test_markdown_template() {
        let env = TestEnv::new();
        let mut cmd = env.command();
        cmd.arg("--output-format=markdown").assert().success();

        let output = env.read_output();
        debug!("Markdown template output:\n{}", output);

        // Check markdown-specific formatting
        assert!(contains("Source Tree:").eval(&output));
        assert!(contains("```rs").eval(&output));
        assert!(contains("fn main()").eval(&output));
        assert!(contains("Hello, world!").eval(&output));
    }

    #[test]
    fn test_xml_template() {
        let env = TestEnv::new();
        let mut cmd = env.command();
        cmd.arg("--output-format=xml").assert().success();

        let output = env.read_output();
        debug!("XML template output:\n{}", output);

        // Check XML-specific formatting
        assert!(contains("<directory>").eval(&output));
        assert!(contains("</file>").eval(&output));
        assert!(contains(".rs\"").eval(&output));
        assert!(contains("fn main()").eval(&output));
        assert!(contains("Hello, world!").eval(&output));
    }

    //     #[test]
    //     fn test_custom_template_with_variables() {
    //         let env = TestEnv::new();

    //         // Create a custom template with variables
    //         let template_content = r#"
    // # {{project_name}} Code Review

    // Author: {{author_name}}
    // Purpose: {{purpose}}

    // ## Files

    // {{#each files}}
    // ### {{path}}

    // ```{{extension}}
    // {{code}}
    // {{/each}}
    // "#;
    //         let template_path =
    //             create_temp_file(&env.dir.path(), "custom_template.hbs", template_content);

    //         // Use command-line arguments to provide variables instead of stdin
    //         let mut cmd = env.command();
    //         cmd.arg("--template")
    //             .arg(template_path.to_str().unwrap())
    //             // Add user variables as command-line arguments
    //             .arg("--var")
    //             .arg("project_name=Test Project")
    //             .arg("--var")
    //             .arg("author_name=John Doe")
    //             .arg("--var")
    //             .arg("purpose=Demonstrating custom templates")
    //             .assert()
    //             .success();

    //         let output = env.read_output();
    //         debug!("Custom template output:\n{}", output);

    //         // Check custom template formatting and variables
    //         assert!(contains("# Test Project Code Review").eval(&output));
    //         assert!(contains("Author: John Doe").eval(&output));
    //         assert!(contains("Purpose: Demonstrating custom templates").eval(&output));
    //         assert!(contains("### src/main.rs").eval(&output));
    //         assert!(contains("fn main()").eval(&output));
    //         assert!(contains("Hello, world!").eval(&output));
    //     }

    #[test]
    fn test_json_output_format() {
        let env = TestEnv::new();
        let mut cmd = env.command();
        cmd.arg("--output-format=json").assert().success();

        let output = env.read_output();
        debug!("JSON output format:\n{}", output);

        // Even though JSON is an output format flag, the content is still markdown
        // But the content should be structured to be machine-parseable
        assert!(starts_with("{").eval(&output));
        assert!(contains("\"directory_name\":").eval(&output));
        assert!(contains("\"prompt\": \"<directory>").eval(&output));
        assert!(ends_with("}").eval(&output));
    }

    //     #[test]
    //     fn test_template_with_empty_variables() {
    //         let env = TestEnv::new();

    //         // Create a custom template with optional variables
    //         let template_content = r#"
    //     Code Review {{#if project_name}}for {{project_name}}{{/if}}
    //     {{#if notes}}
    //     Notes: {{notes}}
    //     {{else}}
    //     No additional notes provided.
    //     {{/if}}

    //     Files
    //     {{#each files}}

    //     {{path}}
    //     {{code}}
    //     {{/each}}
    //     "#;

    //         let template_path = create_temp_file(
    //             &env.dir.path(),
    //             "optional_vars_template.hbs",
    //             template_content,
    //         );

    //         // Use command-line arguments with empty values instead of stdin
    //         let mut cmd = env.command();
    //         cmd.arg("--template")
    //             .arg(template_path.to_str().unwrap())
    //             .arg("--var")
    //             .arg("project_name=")
    //             .arg("--var")
    //             .arg("notes=")
    //             .assert()
    //             .success();

    //         let output = env.read_output();
    //         debug!("Template with empty variables output:\n{}", output);

    //         // Fix assertions to match the actual template format
    //         assert!(contains("Code Review").eval(&output));
    //         assert!(contains("No additional notes provided.").eval(&output));
    //         assert!(contains("src/main.rs").eval(&output));
    //     }
}
