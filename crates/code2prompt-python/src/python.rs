use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::path::PathBuf;

use code2prompt_core::{
    git::{get_git_diff, get_git_diff_between_branches, get_git_log},
    path::traverse_directory,
    template::{handlebars_setup, render_template},
    tokenizer::{count_tokens, TokenizerType},
};

/// Python module for code2prompt
#[pymodule(name = "code2prompt_rs")]
fn code2prompt_rs(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Code2Prompt>()?;
    Ok(())
}

/// Main class for generating prompts from code
#[pyclass]
struct Code2Prompt {
    path: PathBuf,
    include_patterns: Vec<String>,
    exclude_patterns: Vec<String>,
    include_priority: bool,
    line_numbers: bool,
    relative_paths: bool,
    exclude_from_tree: bool,
    no_codeblock: bool,
    follow_symlinks: bool,
    hidden: bool,
    no_ignore: bool,
}

#[pymethods]
impl Code2Prompt {
    /// Create a new Code2Prompt instance
    ///
    /// Args:
    ///     path (str): Path to the codebase directory
    ///     include_patterns (List[str], optional): Patterns to include. Defaults to [].
    ///     exclude_patterns (List[str], optional): Patterns to exclude. Defaults to [].
    ///     include_priority (bool, optional): Give priority to include patterns. Defaults to False.
    ///     line_numbers (bool, optional): Add line numbers to code. Defaults to False.
    ///     relative_paths (bool, optional): Use relative paths. Defaults to False.
    ///     exclude_from_tree (bool, optional): Exclude files from tree based on patterns. Defaults to False.
    ///     no_codeblock (bool, optional): Don't wrap code in markdown blocks. Defaults to False.
    ///     follow_symlinks (bool, optional): Follow symbolic links. Defaults to False.
    ///     hidden (bool, optional): Include hidden directories and files. Defaults to False.
    ///     no_ignore (bool, optional): Skip .gitignore rules. Defaults to False.
    #[new]
    #[pyo3(signature = (
        path,
        include_patterns = vec![],
        exclude_patterns = vec![],
        include_priority = false,
        line_numbers = false,
        relative_paths = false,
        exclude_from_tree = false,
        no_codeblock = false,
        follow_symlinks = false,
        hidden = false,
        no_ignore = false,
    ))]
    fn new(
        path: String,
        include_patterns: Vec<String>,
        exclude_patterns: Vec<String>,
        include_priority: bool,
        line_numbers: bool,
        relative_paths: bool,
        exclude_from_tree: bool,
        no_codeblock: bool,
        follow_symlinks: bool,
        hidden: bool,
        no_ignore: bool,
    ) -> Self {
        Self {
            path: PathBuf::from(path),
            include_patterns,
            exclude_patterns,
            include_priority,
            line_numbers,
            relative_paths,
            exclude_from_tree,
            no_codeblock,
            follow_symlinks,
            hidden,
            no_ignore,
        }
    }

    /// Generate a prompt from the codebase
    ///
    /// Args:
    ///     template (str, optional): Custom Handlebars template. Defaults to None.
    ///     encoding (str, optional): Token encoding to use. Defaults to "cl100k".
    ///
    /// Returns:
    ///     dict: Dictionary containing the rendered prompt and metadata
    #[pyo3(signature = (template=None, encoding=None))]
    fn generate(&self, template: Option<String>, encoding: Option<String>) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            // Traverse directory
            let (tree, files) = traverse_directory(
                &self.path,
                &self.include_patterns,
                &self.exclude_patterns,
                self.include_priority,
                self.line_numbers,
                self.relative_paths,
                self.exclude_from_tree,
                self.no_codeblock,
                self.follow_symlinks,
                self.hidden,
                self.no_ignore,
                None,
            )
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

            // Setup template
            let template_content = template
                .unwrap_or_else(|| include_str!("../../default_template_md.hbs").to_string());
            let handlebars = handlebars_setup(&template_content, "template")
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

            // Prepare data
            let data = serde_json::json!({
                "absolute_code_path": self.path.display().to_string(),
                "source_tree": tree,
                "files": files,
            });

            // Render template
            let rendered = render_template(&handlebars, "template", &data)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

            // Select tokenizer type
            let tokenizer_type = encoding
                .as_deref()
                .unwrap_or("cl100k")
                .parse::<TokenizerType>()
                .unwrap_or(TokenizerType::Cl100kBase); // Fallback to `cl100k`

            let model_info = tokenizer_type.description();

            // Count tokens
            let token_count = count_tokens(&rendered, &tokenizer_type);

            // Create return dictionary
            let result = PyDict::new(py);
            result.set_item("prompt", rendered)?;
            result.set_item("directory", self.path.display().to_string())?;
            result.set_item("token_count", token_count)?;
            result.set_item("model_info", model_info)?;

            Ok(result.into())
        })
    }

    /// Get git diff for the repository
    ///
    /// Returns:
    ///     str: Git diff output
    fn get_git_diff(&self) -> PyResult<String> {
        get_git_diff(&self.path)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Get git diff between two branches
    ///
    /// Args:
    ///     branch1 (str): First branch name
    ///     branch2 (str): Second branch name
    ///
    /// Returns:
    ///     str: Git diff output
    fn get_git_diff_between_branches(&self, branch1: &str, branch2: &str) -> PyResult<String> {
        get_git_diff_between_branches(&self.path, branch1, branch2)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }

    /// Get git log between two branches
    ///
    /// Args:
    ///     branch1 (str): First branch name
    ///     branch2 (str): Second branch name
    ///
    /// Returns:
    ///     str: Git log output
    fn get_git_log(&self, branch1: &str, branch2: &str) -> PyResult<String> {
        get_git_log(&self.path, branch1, branch2)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }
}
