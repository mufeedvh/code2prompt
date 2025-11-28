//! Tests for file processor module
//!
//! This file contains all tests for the file processor implementations,
//! organized by processor type.

use code2prompt_core::file_processor::*;
use std::path::PathBuf;

// ============================================================================
// CSV Processor Tests
// ============================================================================

mod csv_tests {
    use super::*;

    #[test]
    fn test_csv_with_headers_and_data() {
        let processor = CsvProcessor;
        let content = b"name,age,city\nAlice,30,NYC\nBob,25,LA\nCharlie,35,SF";
        let result = processor
            .process(content, &PathBuf::from("test.csv"))
            .unwrap();

        assert!(result.contains("Headers: name, age, city"));
        assert!(result.contains("Sample: \"Alice\", \"30\", \"NYC\""));
        assert!(result.contains("[2 more rows omitted]"));
    }

    #[test]
    fn test_csv_with_quoted_fields() {
        let processor = CsvProcessor;
        let content =
            b"name,description\n\"John Doe\",\"Software Engineer, Senior\"\n\"Jane\",\"Manager\"";
        let result = processor
            .process(content, &PathBuf::from("test.csv"))
            .unwrap();

        assert!(result.contains("Headers: name, description"));
        assert!(result.contains("Sample: \"John Doe\", \"Software Engineer, Senior\""));
    }

    #[test]
    fn test_csv_empty() {
        let processor = CsvProcessor;
        let content = b"name,age\n";
        let result = processor
            .process(content, &PathBuf::from("test.csv"))
            .unwrap();

        assert!(result.contains("Headers: name, age"));
        assert!(result.contains("(No data rows found)"));
    }

    #[test]
    fn test_csv_malformed_fallback() {
        let processor = CsvProcessor;
        let content = b"not a valid csv file\nwith random\ncontent";
        let result = processor
            .process(content, &PathBuf::from("test.csv"))
            .unwrap();

        // Should fallback to raw text
        assert!(result.contains("not a valid csv file"));
    }
}

// ============================================================================
// TSV Processor Tests
// ============================================================================

mod tsv_tests {
    use super::*;

    #[test]
    fn test_tsv_with_headers_and_data() {
        let processor = TsvProcessor;
        let content = b"name\tage\tcity\nAlice\t30\tNYC\nBob\t25\tLA\nCharlie\t35\tSF";
        let result = processor
            .process(content, &PathBuf::from("test.tsv"))
            .unwrap();

        assert!(result.contains("TSV Schema"));
        assert!(result.contains("Headers: name, age, city"));
        assert!(result.contains("Sample: \"Alice\", \"30\", \"NYC\""));
        assert!(result.contains("[2 more rows omitted]"));
    }

    #[test]
    fn test_tsv_with_spaces() {
        let processor = TsvProcessor;
        let content = b"name\tdescription\nJohn Doe\tSoftware Engineer\nJane\tManager";
        let result = processor
            .process(content, &PathBuf::from("test.tsv"))
            .unwrap();

        assert!(result.contains("TSV Schema"));
        assert!(result.contains("Headers: name, description"));
        assert!(result.contains("Sample: \"John Doe\", \"Software Engineer\""));
    }

    #[test]
    fn test_tsv_empty() {
        let processor = TsvProcessor;
        let content = b"name\tage\n";
        let result = processor
            .process(content, &PathBuf::from("test.tsv"))
            .unwrap();

        assert!(result.contains("Headers: name, age"));
        assert!(result.contains("(No data rows found)"));
    }
}

// ============================================================================
// JSONL Processor Tests
// ============================================================================

mod jsonl_tests {
    use super::*;

    #[test]
    fn test_jsonl_with_multiple_lines() {
        let processor = JsonLinesProcessor;
        let content = b"{\"id\":1,\"name\":\"Alice\",\"age\":30}\n{\"id\":2,\"name\":\"Bob\",\"age\":25}\n{\"id\":3,\"name\":\"Charlie\",\"age\":35}";
        let result = processor
            .process(content, &PathBuf::from("test.jsonl"))
            .unwrap();

        assert!(result.contains("JSONL Schema"));
        assert!(
            result.contains("Fields: id, name, age")
                || result.contains("Fields: name, id, age")
                || result.contains("Fields: age, id, name")
        );
        assert!(result.contains("Sample: {\"id\":1,\"name\":\"Alice\",\"age\":30}"));
        assert!(result.contains("[2 more lines omitted]"));
    }

    #[test]
    fn test_jsonl_single_line() {
        let processor = JsonLinesProcessor;
        let content = b"{\"user\":\"john\",\"action\":\"login\"}";
        let result = processor
            .process(content, &PathBuf::from("test.jsonl"))
            .unwrap();

        assert!(result.contains("JSONL Schema"));
        assert!(result.contains("user") && result.contains("action"));
        assert!(result.contains("Sample: {\"user\":\"john\",\"action\":\"login\"}"));
        assert!(!result.contains("more lines omitted"));
    }

    #[test]
    fn test_jsonl_with_nested_objects() {
        let processor = JsonLinesProcessor;
        let content = b"{\"id\":1,\"user\":{\"name\":\"Alice\",\"email\":\"alice@example.com\"}}\n{\"id\":2,\"user\":{\"name\":\"Bob\",\"email\":\"bob@example.com\"}}";
        let result = processor
            .process(content, &PathBuf::from("test.jsonl"))
            .unwrap();

        assert!(result.contains("JSONL Schema"));
        assert!(result.contains("id") && result.contains("user"));
    }

    #[test]
    fn test_jsonl_empty_file() {
        let processor = JsonLinesProcessor;
        let content = b"";
        let result = processor.process(content, &PathBuf::from("test.jsonl"));

        assert!(result.is_err());
    }

    #[test]
    fn test_jsonl_invalid_json() {
        let processor = JsonLinesProcessor;
        let content = b"not a valid json\nanother line";
        let result = processor.process(content, &PathBuf::from("test.jsonl"));

        assert!(result.is_err());
    }

    #[test]
    fn test_jsonl_with_fallback() {
        let processor = JsonLinesProcessor;
        let content = b"invalid json content";
        let result = processor
            .process_with_fallback(content, &PathBuf::from("test.jsonl"))
            .unwrap();

        // Should fallback to raw text
        assert!(result.contains("invalid json content"));
    }
}

// ============================================================================
// Jupyter Notebook Processor Tests
// ============================================================================

mod ipynb_tests {
    use super::*;

    #[test]
    fn test_ipynb_with_code_cells() {
        let processor = JupyterNotebookProcessor;
        let content = r##"{
            "cells": [
                {
                    "cell_type": "code",
                    "source": ["import pandas as pd\n", "df = pd.read_csv(\"data.csv\")"]
                },
                {
                    "cell_type": "markdown",
                    "source": ["# This is a title"]
                },
                {
                    "cell_type": "code",
                    "source": "df.head()"
                }
            ]
        }"##;

        let result = processor
            .process(content.as_bytes(), &PathBuf::from("test.ipynb"))
            .unwrap();

        assert!(result.contains("Jupyter Notebook Summary"));
        assert!(result.contains("Total cells: 3 (2 code, 1 markdown, 0 raw)"));
        assert!(result.contains("Code Cell #1:"));
        assert!(result.contains("import pandas as pd"));
        assert!(result.contains("Code Cell #2:"));
        assert!(result.contains("df.head()"));
    }

    #[test]
    fn test_ipynb_with_many_code_cells() {
        let processor = JupyterNotebookProcessor;
        let content = r#"{
            "cells": [
                {"cell_type": "code", "source": "cell1"},
                {"cell_type": "code", "source": "cell2"},
                {"cell_type": "code", "source": "cell3"},
                {"cell_type": "code", "source": "cell4"},
                {"cell_type": "code", "source": "cell5"}
            ]
        }"#;

        let result = processor
            .process(content.as_bytes(), &PathBuf::from("test.ipynb"))
            .unwrap();

        assert!(result.contains("Total cells: 5 (5 code, 0 markdown, 0 raw)"));
        assert!(result.contains("Code Cell #1:"));
        assert!(result.contains("Code Cell #2:"));
        assert!(result.contains("Code Cell #3:"));
        assert!(result.contains("[2 more code cells omitted]"));
        assert!(!result.contains("Code Cell #4:"));
    }

    #[test]
    fn test_ipynb_no_code_cells() {
        let processor = JupyterNotebookProcessor;
        let content = r##"{
            "cells": [
                {"cell_type": "markdown", "source": "# Title"},
                {"cell_type": "markdown", "source": "Some text"}
            ]
        }"##;

        let result = processor
            .process(content.as_bytes(), &PathBuf::from("test.ipynb"))
            .unwrap();

        assert!(result.contains("Total cells: 2 (0 code, 2 markdown, 0 raw)"));
        assert!(result.contains("(No code cells found)"));
    }

    #[test]
    fn test_ipynb_invalid_json() {
        let processor = JupyterNotebookProcessor;
        let content = b"not a valid json";

        let result = processor.process(content, &PathBuf::from("test.ipynb"));
        assert!(result.is_err());
    }

    #[test]
    fn test_ipynb_with_fallback() {
        let processor = JupyterNotebookProcessor;
        let content = b"invalid notebook content";

        let result = processor
            .process_with_fallback(content, &PathBuf::from("test.ipynb"))
            .unwrap();

        // Should fallback to raw text
        assert!(result.contains("invalid notebook content"));
    }
}

// ============================================================================
// Default Text Processor Tests
// ============================================================================

mod default_tests {
    use super::*;

    #[test]
    fn test_valid_utf8() {
        let processor = DefaultTextProcessor;
        let content = b"Hello, world!";
        let result = processor
            .process(content, &PathBuf::from("test.txt"))
            .unwrap();
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn test_invalid_utf8() {
        let processor = DefaultTextProcessor;
        let content = b"Hello\xFF\xFEworld";
        let result = processor
            .process(content, &PathBuf::from("test.txt"))
            .unwrap();
        assert!(result.contains("Hello"));
        assert!(result.contains("world"));
    }
}
