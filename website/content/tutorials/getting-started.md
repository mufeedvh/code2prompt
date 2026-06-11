+++
title = "Getting Started"
description = "A comprehensive tutorial introducing gnaw's core functionality and its use across CLI, Python, and REST integrations."
weight = 1
+++

{% aside(kind="note", title="Tutorial Overview") %}
Welcome to gnaw! This tutorial provides a comprehensive introduction to using
gnaw to generate AI-ready prompts from your codebases. We'll explore its core
functionality and demonstrate its usage across different integration methods.
{% end %}

## What is gnaw?

gnaw is a versatile tool designed to bridge the gap between your codebase and
Large Language Models (LLMs). It intelligently extracts relevant code snippets,
applies powerful filtering, and formats the information into structured prompts
optimized for LLM consumption. This simplifies tasks like code documentation,
bug detection, refactoring, and more.

gnaw offers different integration points:

{% tabs() %}
=== Core
A core Rust library (`gnaw-core`) that provides the foundation for code ingestion and prompt generation.
=== CLI / TUI
A user-friendly command-line and terminal interface for quick prompt generation. Ideal for interactive use and one-off tasks.
=== Python
Python bindings for seamless integration into your Python projects. Perfect for automating prompt generation within larger workflows.
=== REST
A planned REST interface for advanced integration with browser extensions and other clients, enabling real-time interactions with your codebase.
{% end %}

## 📥 Installation

For detailed installation instructions, please refer to the comprehensive
[Installation Guide](/how-to/install/).

## 🏁 Generating Prompts: A CLI Example

Let's start with a simple example using the CLI. Create a sample project:

{% code(title="bash") %}
```bash
mkdir -p my_project/{src,tests}
touch my_project/src/main.rs my_project/tests/test_1.rs
echo 'fn main() { println!("Hello, world!"); }' > my_project/src/main.rs
```
{% end %}

Now, generate a prompt:

{% code(title="bash") %}
```bash
gnaw my_project
```
{% end %}

This copies a prompt to your clipboard. You can customize this:

- **Filtering:** `gnaw my_project --include="*.rs" --exclude="tests/*"` (includes only `.rs` files, excludes `tests` directory)
- **Output File:** `gnaw my_project --output-file=my_prompt.txt`
- **JSON Output:** `gnaw my_project -O json` (structured JSON output)
- **Custom Templates:** `gnaw my_project -t my_template.hbs` (requires creating `my_template.hbs`)

See the [Learn Context Filtering](/tutorials/learn-filtering/) and
[Learn Handlebar Templates](/tutorials/learn-templating/) tutorials to learn
more advanced usages.

## 🐍 Python Integration

For programmatic control, use the Python bindings:

```python
from gnaw import Gnaw

config = {
    "path": "my_project",
    "include_patterns": ["*.rs"],
    "exclude_patterns": ["tests/*"],
}

g = Gnaw(**config)
prompt = g.generate_prompt()
print(prompt)
```

{% aside(kind="caution") %}
The Python package is built with PyO3/maturin and is not yet published to PyPI.
Package and API names may change before release.
{% end %}

{% aside(kind="note", title="Next Steps") %}
Explore the advanced tutorials and documentation to master gnaw's capabilities
and integrate it into your workflows.
{% end %}
