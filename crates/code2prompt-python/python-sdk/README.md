# code2prompt Python SDK

Python bindings for [code2prompt](https://github.com/mufeedvh/code2prompt) - A tool to generate LLM prompts from codebases.

## Installation

### Local Development Installation

1. Clone the repository:

```bash
git clone https://github.com/mufeedvh/code2prompt.git
cd code2prompt
```

2. Install development dependencies:

```bash
python3 -m venv .venv
source .venv/bin/activate
pip install maturin pytest
```

3. Build and install the package locally:

```bash
cd code2prompt/ # root repo directory
maturin develop -r
```

### Running Examples

Try out the example script:

```bash
python examples/basic_usage.py
```

## Usage

```python
from code2prompt import CodePrompt

# Create a new CodePrompt instance
prompt = CodePrompt(
    path="./my_project",
    include_patterns=["*.py", "*.rs"],  # Optional: Only include Python and Rust files
    exclude_patterns=["**/tests/*"],     # Optional: Exclude test files
    line_numbers=True,                   # Optional: Add line numbers to code
)

# Generate a prompt
result = prompt.generate(
    template=None,  # Optional: Custom Handlebars template
    encoding="cl100k"  # Optional: Token encoding (for token counting)
)

# Access the generated prompt and metadata
print(f"Generated prompt: {result['prompt']}")
print(f"Token count: {result['token_count']}")
print(f"Model info: {result['model_info']}")

# Git operations
git_diff = prompt.get_git_diff()
branch_diff = prompt.get_git_diff_between_branches("main", "feature")
git_log = prompt.get_git_log("main", "feature")
```

## API Reference

### `CodePrompt`

Main class for generating prompts from code.

#### Constructor

```python
CodePrompt(
    path: str,
    include_patterns: List[str] = [],
    exclude_patterns: List[str] = [],
    include_priority: bool = False,
    line_numbers: bool = False,
    relative_paths: bool = False,
    exclude_from_tree: bool = False,
    no_codeblock: bool = False,
    follow_symlinks: bool = False
)
```

- `path`: Path to the codebase directory
- `include_patterns`: List of glob patterns for files to include
- `exclude_patterns`: List of glob patterns for files to exclude
- `include_priority`: Give priority to include patterns in case of conflicts
- `line_numbers`: Add line numbers to code blocks
- `relative_paths`: Use relative paths instead of absolute
- `exclude_from_tree`: Exclude files from source tree based on patterns
- `no_codeblock`: Don't wrap code in markdown code blocks
- `follow_symlinks`: Follow symbolic links when traversing directories

#### Methods

##### `generate(template: Optional[str] = None, encoding: Optional[str] = None) -> Dict`

Generate a prompt from the codebase.

- `template`: Optional custom Handlebars template
- `encoding`: Optional token encoding (cl100k, p50k, p50k_edit, r50k, gpt2)

Returns a dictionary containing:

- `prompt`: The generated prompt
- `directory`: The processed directory path
- `token_count`: Number of tokens (if encoding was specified)
- `model_info`: Information about the model (if encoding was specified)

##### `get_git_diff() -> str`

Get git diff for the repository.

##### `get_git_diff_between_branches(branch1: str, branch2: str) -> str`

Get git diff between two branches.

##### `get_git_log(branch1: str, branch2: str) -> str`

Get git log between two branches.

## License

MIT License - see LICENSE file for details.
