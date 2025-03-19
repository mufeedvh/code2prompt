use pyo3::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;

use code2prompt_core::configuration::Code2PromptConfigBuilder;
use code2prompt_core::session::Code2PromptSession;
use code2prompt_core::sort::FileSortMethod;
use code2prompt_core::template::OutputFormat;
use code2prompt_core::tokenizer::{TokenFormat, TokenizerType};

#[pyclass]
#[derive(Clone)]
struct PyCode2PromptSession {
    inner: Code2PromptSession,
}

#[pymethods]
impl PyCode2PromptSession {
    #[new]
    fn new(path: &str) -> PyResult<Self> {
        let config = Code2PromptConfigBuilder::default()
            .path(PathBuf::from(path))
            .build()
            .map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                    "Failed to create config: {}",
                    e
                ))
            })?;

        Ok(Self {
            inner: Code2PromptSession::new(config),
        })
    }

    // Configure methods that modify the config
    fn include(&mut self, patterns: Vec<String>) -> PyResult<Py<Self>> {
        let mut config = self.inner.config.clone();
        config.include_patterns = patterns;
        self.inner = Code2PromptSession::new(config);

        Python::with_gil(|py| {
            Ok(Py::new(
                py,
                Self {
                    inner: self.inner.clone(),
                },
            )?)
        })
    }

    fn exclude(&mut self, patterns: Vec<String>) -> PyResult<Py<Self>> {
        let mut config = self.inner.config.clone();
        config.exclude_patterns = patterns;
        self.inner = Code2PromptSession::new(config);

        Python::with_gil(|py| {
            Ok(Py::new(
                py,
                Self {
                    inner: self.inner.clone(),
                },
            )?)
        })
    }

    fn include_priority(&mut self, value: bool) -> PyResult<Py<Self>> {
        let mut config = self.inner.config.clone();
        config.include_priority = value;
        self.inner = Code2PromptSession::new(config);

        Python::with_gil(|py| {
            Ok(Py::new(
                py,
                Self {
                    inner: self.inner.clone(),
                },
            )?)
        })
    }

    fn with_line_numbers(&mut self, value: bool) -> PyResult<Py<Self>> {
        let mut config = self.inner.config.clone();
        config.line_numbers = value;
        self.inner = Code2PromptSession::new(config);

        Python::with_gil(|py| {
            Ok(Py::new(
                py,
                Self {
                    inner: self.inner.clone(),
                },
            )?)
        })
    }

    fn with_absolute_paths(&mut self, value: bool) -> PyResult<Py<Self>> {
        let mut config = self.inner.config.clone();
        config.absolute_path = value;
        self.inner = Code2PromptSession::new(config);

        Python::with_gil(|py| {
            Ok(Py::new(
                py,
                Self {
                    inner: self.inner.clone(),
                },
            )?)
        })
    }

    fn with_full_directory_tree(&mut self, value: bool) -> PyResult<Py<Self>> {
        let mut config = self.inner.config.clone();
        config.full_directory_tree = value;
        self.inner = Code2PromptSession::new(config);

        Python::with_gil(|py| {
            Ok(Py::new(
                py,
                Self {
                    inner: self.inner.clone(),
                },
            )?)
        })
    }

    fn with_code_blocks(&mut self, value: bool) -> PyResult<Py<Self>> {
        let mut config = self.inner.config.clone();
        config.no_codeblock = !value; // Invert because API is different
        self.inner = Code2PromptSession::new(config);

        Python::with_gil(|py| {
            Ok(Py::new(
                py,
                Self {
                    inner: self.inner.clone(),
                },
            )?)
        })
    }

    fn follow_symlinks(&mut self, value: bool) -> PyResult<Py<Self>> {
        let mut config = self.inner.config.clone();
        config.follow_symlinks = value;
        self.inner = Code2PromptSession::new(config);

        Python::with_gil(|py| {
            Ok(Py::new(
                py,
                Self {
                    inner: self.inner.clone(),
                },
            )?)
        })
    }

    fn include_hidden(&mut self, value: bool) -> PyResult<Py<Self>> {
        let mut config = self.inner.config.clone();
        config.hidden = value;
        self.inner = Code2PromptSession::new(config);

        Python::with_gil(|py| {
            Ok(Py::new(
                py,
                Self {
                    inner: self.inner.clone(),
                },
            )?)
        })
    }

    fn no_ignore(&mut self, value: bool) -> PyResult<Py<Self>> {
        let mut config = self.inner.config.clone();
        config.no_ignore = value;
        self.inner = Code2PromptSession::new(config);

        Python::with_gil(|py| {
            Ok(Py::new(
                py,
                Self {
                    inner: self.inner.clone(),
                },
            )?)
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
            )))
            }
        }
        self.inner = Code2PromptSession::new(config);

        Python::with_gil(|py| {
            Ok(Py::new(
                py,
                Self {
                    inner: self.inner.clone(),
                },
            )?)
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
                )))
            }
        }
        self.inner = Code2PromptSession::new(config);

        Python::with_gil(|py| {
            Ok(Py::new(
                py,
                Self {
                    inner: self.inner.clone(),
                },
            )?)
        })
    }

    fn with_token_encoding(&mut self, encoding: &str) -> PyResult<Py<Self>> {
        let mut config = self.inner.config.clone();
        match encoding.to_lowercase().as_str() {
            "gpt2" => config.encoding = TokenizerType::Gpt2,
            "cl100k" => config.encoding = TokenizerType::Cl100kBase,
            "o200k" => config.encoding = TokenizerType::O200kBase,
            "p50k" => config.encoding = TokenizerType::P50kBase,
            "p50k_edit" => config.encoding = TokenizerType::P50kEdit,
            "r50k" => config.encoding = TokenizerType::R50kBase,
            _ => {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Invalid token encoding: {}",
                    encoding
                )))
            }
        }
        self.inner = Code2PromptSession::new(config);

        Python::with_gil(|py| {
            Ok(Py::new(
                py,
                Self {
                    inner: self.inner.clone(),
                },
            )?)
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
                )))
            }
        }
        self.inner = Code2PromptSession::new(config);

        Python::with_gil(|py| {
            Ok(Py::new(
                py,
                Self {
                    inner: self.inner.clone(),
                },
            )?)
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
        self.inner = Code2PromptSession::new(config);

        Python::with_gil(|py| {
            Ok(Py::new(
                py,
                Self {
                    inner: self.inner.clone(),
                },
            )?)
        })
    }

    #[pyo3(signature = (key, value))]
    fn with_variable(&mut self, key: String, value: String) -> PyResult<Py<Self>> {
        let mut config = self.inner.config.clone();
        config.user_variables.insert(key, value);
        self.inner = Code2PromptSession::new(config);

        Python::with_gil(|py| {
            Ok(Py::new(
                py,
                Self {
                    inner: self.inner.clone(),
                },
            )?)
        })
    }

    fn generate(&mut self) -> PyResult<String> {
        match self.inner.generate_prompt() {
            Ok(rendered) => Ok(rendered.prompt),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Failed to generate prompt: {}",
                e
            ))),
        }
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
        // Generate the prompt and count tokens
        match self.inner.clone().generate_prompt() {
            Ok(rendered) => Ok(rendered.token_count),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Failed to count tokens: {}",
                e
            ))),
        }
    }
}

// Module definition - Updated PyO3 syntax
#[pymodule(name = "code2prompt_rs")]
fn code2prompt_rs(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyCode2PromptSession>()?;
    Ok(())
}
