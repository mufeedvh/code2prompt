use code2prompt::path::traverse_directory;
use code2prompt::template::render_template;
use pyo3::prelude::*;

/// A Python module implemented in Rust.
#[pymodule(name = "code2prompt_rs")]
fn code2prompt(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<CodePrompt>()?;
    Ok(())
}

#[pyclass]
struct CodePrompt {
    path: String,
}

#[pymethods]
impl CodePrompt {
    #[new]
    fn new(path: String) -> Self {
        Self { path }
    }

    fn generate(&self) -> PyResult<String> {
        let (tree, _files) = traverse_directory(&self.path);
        let rendered = render_template(tree);
        Ok(rendered)
    }
}
