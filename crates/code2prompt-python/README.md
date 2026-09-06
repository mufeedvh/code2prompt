# code2prompt Python bindings

Native Python bindings for the stateful
[`code2prompt_core`](https://crates.io/crates/code2prompt_core) API.

## Installation

```bash
pip install code2prompt_rs
```

Python 3.11 or newer is supported.

Rust `PathBuf` values accept Python strings or `os.PathLike` objects and are exposed as
`pathlib.Path` objects.

## Usage

Create a core configuration, move it into a session, and operate on that session just as you
would from Rust:

```python
from code2prompt_rs import Code2PromptConfig, Code2PromptSession

config = Code2PromptConfig(
    ".",
    include_patterns=["**/*.py", "**/*.rs"],
    exclude_patterns=["**/tests/**"],
    line_numbers=True,
    deselected=True,
)
session = Code2PromptSession(config)

session.select_file("src/main.rs")
session.select_file("src/lib.rs")

result = session.generate_prompt()
print(result.prompt)
print(result.token_count)
print(result.files)
```

`result.token_count` estimates the prompt size from cached per-file token counts
(including optional line numbers) and estimated template overhead. The full rendered
prompt is not re-tokenized, and the JSON output envelope is not counted.

Selection methods mutate and return the same session, so calls may also be chained:

```python
session.select_file("src/main.rs").deselect_file("src/generated.rs")
```

## Configuration

Configuration values use native enum classes rather than string aliases:

```python
from code2prompt_rs import (
    Code2PromptConfig,
    FileProcessorsConfig,
    FileSortMethod,
    IpynbProcessorConfig,
    OutputFormat,
    TokenFormat,
    TokenizerType,
)

config = Code2PromptConfig(
    ".",
    absolute_path=False,
    no_codeblock=False,
    output_format=OutputFormat.Markdown,
    sort_method=FileSortMethod.NameAsc,
    encoding=TokenizerType.Cl100kBase,
    token_format=TokenFormat.Raw,
    processors=FileProcessorsConfig(
        ipynb=IpynbProcessorConfig(
            max_code_cells=5,
            include_outputs=True,
            include_markdown=True,
        )
    ),
)
```

`Code2PromptConfig` exposes the user-facing fields from core 4.3. The optional Rust
`entity-map` feature is not included in the Python wheels.

## Session API

`Code2PromptSession` exposes the core operational methods:

- Pattern updates: `add_include_pattern`, `add_exclude_pattern`
- Selection: `select_file`, `deselect_file`, `toggle_file_selection`,
  `is_file_selected`, `get_selected_files`, `clear_user_actions`, `has_user_actions`,
  `set_deselected`
- Loading: `load_codebase`, `load_git_diff`, `load_git_diff_between_branches`,
  `load_git_log_between_branches`
- Results: `generate_prompt`, `raw_analysis`, `contextual_analysis`

`generate_prompt()` returns a typed `RenderedPrompt`. Loaded codebase and Git values are
available from the typed `session.data` snapshot. Analysis calls return `CodebaseAnalysis`,
whose `raw_files()`, `by_extension()`, and `token_map(options)` methods return typed models.

## Local development

```bash
uv sync
uv run pytest
```

`uv sync` creates `.venv`, installs the development dependencies, and builds the native
extension in editable mode. The lockfile is committed for reproducible development environments.

To build a release wheel locally:

```bash
uv run maturin build --release
```
