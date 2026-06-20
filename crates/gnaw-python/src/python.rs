use pyo3::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;

use gnaw_core::configuration::GnawConfig;
use gnaw_core::configuration::GnawConfigBuilder;
use gnaw_core::path::display_name;
use gnaw_core::pipeline::adapters::{
    FullWalkTree, HandlebarsRenderer, IdentityChunker, ItemsTree, PatternSelector, RendererConfig,
    SecretScrubber, TakeUntilBudget, TiktokenCounter, Uniform, WorkingTreeSource,
};
use gnaw_core::pipeline::ports::TreeBuilder;
use gnaw_core::pipeline::{PipelineSpec, Rendered, SourceOpts, run};
use gnaw_core::session::SelectionState;
use gnaw_core::sort::FileSortMethod;
use gnaw_core::template::OutputFormat;
use gnaw_core::tokenizer::{TokenFormat, TokenizerType};
#[pyclass(from_py_object)]
#[derive(Clone)]
struct PyGnawSession {
    inner: SelectionState,
}

/// Build and run the whole-repo extraction pipeline for `config`.
///
/// Python's API is whole-repo only (no --diff / git-narrative templates), so
/// this mirrors the default arm of the CLI's `build_spec`. The duplication is
/// deliberate and temporary: when `build_spec` moves into `gnaw-core` (REST,
/// MCP, and now Python all need it — the second use case that justifies the
/// move), delete this and call the shared builder. Errors are stringified at
/// the boundary so the crate needs no anyhow/PipelineError import.
fn run_pipeline(config: &GnawConfig) -> Result<Rendered, String> {
    let tree_builder: Box<dyn TreeBuilder> = if config.full_directory_tree {
        Box::new(FullWalkTree::new(config.clone()))
    } else {
        Box::new(ItemsTree)
    };

    let spec = PipelineSpec {
        source: Box::new(WorkingTreeSource::new(config.clone())),
        selector: Box::new(PatternSelector::new(
            &config.include_patterns,
            &config.exclude_patterns,
        )),
        chunker: Box::new(IdentityChunker),
        scrubber: Box::new(SecretScrubber::new(config)),
        ranker: Box::new(Uniform),
        budgeter: Box::new(TakeUntilBudget::new(Box::new(TiktokenCounter::new(
            config.encoding,
        )))),
        renderer: Box::new(HandlebarsRenderer::new(RendererConfig {
            no_codeblock: config.no_codeblock,
            line_numbers: config.line_numbers,
            git_diff: None,
            git_diff_branch: None,
            git_log_branch: None,
            template_str: config.template_str.clone(),
            template_name: config.template_name.clone(),
            output_format: config.output_format,
            user_variables: config.user_variables.clone(),
        })),
        tree_builder,
        budget: 0,
        root_label: display_name(&config.path),
        sort_method: config.sort_method,
    };

    run(&spec, &SourceOpts).map_err(|e| e.to_string())
}

#[pymethods]
impl PyGnawSession {
    #[new]
    fn new(path: &str) -> PyResult<Self> {
        let config = GnawConfigBuilder::default()
            .path(PathBuf::from(path))
            .build()
            .map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                    "Failed to create config: {}",
                    e
                ))
            })?;

        Ok(Self {
            inner: SelectionState::new(config),
        })
    }

    // Configure methods that modify the config
    fn include(&mut self, patterns: Vec<String>) -> PyResult<Py<Self>> {
        let mut config = self.inner.config.clone();
        config.include_patterns = patterns;
        self.inner = SelectionState::new(config);

        Python::attach(|py| {
            Py::new(
                py,
                Self {
                    inner: self.inner.clone(),
                },
            )
        })
    }

    fn exclude(&mut self, patterns: Vec<String>) -> PyResult<Py<Self>> {
        let mut config = self.inner.config.clone();
        config.exclude_patterns = patterns;
        self.inner = SelectionState::new(config);

        Python::attach(|py| {
            Py::new(
                py,
                Self {
                    inner: self.inner.clone(),
                },
            )
        })
    }

    fn with_line_numbers(&mut self, value: bool) -> PyResult<Py<Self>> {
        let mut config = self.inner.config.clone();
        config.line_numbers = value;
        self.inner = SelectionState::new(config);

        Python::attach(|py| {
            Py::new(
                py,
                Self {
                    inner: self.inner.clone(),
                },
            )
        })
    }

    fn with_absolute_paths(&mut self, value: bool) -> PyResult<Py<Self>> {
        let mut config = self.inner.config.clone();
        config.absolute_path = value;
        self.inner = SelectionState::new(config);

        Python::attach(|py| {
            Py::new(
                py,
                Self {
                    inner: self.inner.clone(),
                },
            )
        })
    }

    fn with_full_directory_tree(&mut self, value: bool) -> PyResult<Py<Self>> {
        let mut config = self.inner.config.clone();
        config.full_directory_tree = value;
        self.inner = SelectionState::new(config);

        Python::attach(|py| {
            Py::new(
                py,
                Self {
                    inner: self.inner.clone(),
                },
            )
        })
    }

    fn with_code_blocks(&mut self, value: bool) -> PyResult<Py<Self>> {
        let mut config = self.inner.config.clone();
        config.no_codeblock = !value; // Invert because API is different
        self.inner = SelectionState::new(config);

        Python::attach(|py| {
            Py::new(
                py,
                Self {
                    inner: self.inner.clone(),
                },
            )
        })
    }

    fn follow_symlinks(&mut self, value: bool) -> PyResult<Py<Self>> {
        let mut config = self.inner.config.clone();
        config.follow_symlinks = value;
        self.inner = SelectionState::new(config);

        Python::attach(|py| {
            Py::new(
                py,
                Self {
                    inner: self.inner.clone(),
                },
            )
        })
    }

    fn include_hidden(&mut self, value: bool) -> PyResult<Py<Self>> {
        let mut config = self.inner.config.clone();
        config.hidden = value;
        self.inner = SelectionState::new(config);

        Python::attach(|py| {
            Py::new(
                py,
                Self {
                    inner: self.inner.clone(),
                },
            )
        })
    }

    fn no_ignore(&mut self, value: bool) -> PyResult<Py<Self>> {
        let mut config = self.inner.config.clone();
        config.no_ignore = value;
        self.inner = SelectionState::new(config);

        Python::attach(|py| {
            Py::new(
                py,
                Self {
                    inner: self.inner.clone(),
                },
            )
        })
    }

    fn sort_by(&mut self, method: &str) -> PyResult<Py<Self>> {
        let mut config = self.inner.config.clone();
        match method.to_lowercase().as_str() {
            "name" | "name_asc" => config.sort_method = Some(FileSortMethod::NameAsc),
            "name_desc" => config.sort_method = Some(FileSortMethod::NameDesc),
            "date" | "date_asc" => config.sort_method = Some(FileSortMethod::DateAsc),
            "date_desc" => config.sort_method = Some(FileSortMethod::DateDesc),
            _ => {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Invalid sort method: {}. Valid values: name_asc, name_desc, date_asc, date_desc",
                    method
                )));
            }
        }
        self.inner = SelectionState::new(config);

        Python::attach(|py| {
            Py::new(
                py,
                Self {
                    inner: self.inner.clone(),
                },
            )
        })
    }

    fn output_format(&mut self, format: &str) -> PyResult<Py<Self>> {
        let mut config = self.inner.config.clone();
        match format.to_lowercase().as_str() {
            "markdown" => config.output_format = OutputFormat::Markdown,
            // Assuming from the error that there's a Plain variant - please replace if needed
            "xml" | "text" => config.output_format = OutputFormat::Xml,
            "json" => config.output_format = OutputFormat::Json,
            _ => {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Invalid output format: {}",
                    format
                )));
            }
        }
        self.inner = SelectionState::new(config);

        Python::attach(|py| {
            Py::new(
                py,
                Self {
                    inner: self.inner.clone(),
                },
            )
        })
    }

    fn with_token_encoding(&mut self, encoding: &str) -> PyResult<Py<Self>> {
        let mut config = self.inner.config.clone();
        match encoding.to_lowercase().as_str() {
            "cl100k" => config.encoding = TokenizerType::Cl100kBase,
            "o200k" => config.encoding = TokenizerType::O200kBase,
            "p50k" => config.encoding = TokenizerType::P50kBase,
            "p50k_edit" => config.encoding = TokenizerType::P50kEdit,
            "r50k" => config.encoding = TokenizerType::R50kBase,
            _ => {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Invalid token encoding: {}",
                    encoding
                )));
            }
        }
        self.inner = SelectionState::new(config);

        Python::attach(|py| {
            Py::new(
                py,
                Self {
                    inner: self.inner.clone(),
                },
            )
        })
    }

    fn with_token_format(&mut self, format: &str) -> PyResult<Py<Self>> {
        let mut config = self.inner.config.clone();
        match format.to_lowercase().as_str() {
            "raw" => config.token_format = TokenFormat::Raw,
            "format" => config.token_format = TokenFormat::Format,
            _ => {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Invalid token format: {}. Use 'raw' or 'format'.",
                    format
                )));
            }
        }
        self.inner = SelectionState::new(config);

        Python::attach(|py| {
            Py::new(
                py,
                Self {
                    inner: self.inner.clone(),
                },
            )
        })
    }

    #[pyo3(signature = (template, name=None))]
    fn with_template(&mut self, template: String, name: Option<String>) -> PyResult<Py<Self>> {
        let mut config = self.inner.config.clone();
        config.template_str = template;
        if let Some(name_val) = name {
            config.template_name = name_val;
        } else {
            config.template_name = "custom".to_string();
        }
        self.inner = SelectionState::new(config);

        Python::attach(|py| {
            Py::new(
                py,
                Self {
                    inner: self.inner.clone(),
                },
            )
        })
    }

    #[pyo3(signature = (key, value))]
    fn with_variable(&mut self, key: String, value: String) -> PyResult<Py<Self>> {
        let mut config = self.inner.config.clone();
        config.user_variables.insert(key, value);
        self.inner = SelectionState::new(config);

        Python::attach(|py| {
            Py::new(
                py,
                Self {
                    inner: self.inner.clone(),
                },
            )
        })
    }

    fn generate(&mut self) -> PyResult<String> {
        run_pipeline(&self.inner.config)
            .map(|r| r.body)
            .map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                    "Failed to generate prompt: {}",
                    e
                ))
            })
    }

    fn info(&self) -> PyResult<HashMap<String, String>> {
        // Since there's no direct info() method, we'll create a simple info map
        let mut info = HashMap::new();
        info.insert(
            "path".to_string(),
            self.inner.config.path.to_string_lossy().to_string(),
        );
        info.insert(
            "include_patterns".to_string(),
            format!("{:?}", self.inner.config.include_patterns),
        );
        info.insert(
            "exclude_patterns".to_string(),
            format!("{:?}", self.inner.config.exclude_patterns),
        );

        Ok(info)
    }

    fn token_count(&self) -> PyResult<usize> {
        run_pipeline(&self.inner.config)
            .map(|r| r.tally.total)
            .map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                    "Failed to count tokens: {}",
                    e
                ))
            })
    }
}

// Module definition - Updated PyO3 syntax
#[pymodule(name = "gnaw")]
fn gnaw(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyGnawSession>()?;
    Ok(())
}
