+++
title = "Configuration"
description = "Learn how to use .c2pconfig to automate your prompt generation workflow and ensure team consistency."
weight = 4
+++

{% aside(kind="note", title="Tutorial Overview") %}
Manually typing long exclude patterns or specific tokenizer settings every time
can be tedious. This tutorial shows you **how to use a `.c2pconfig`
configuration file** to "set and forget" your project settings.
{% end %}

## Prerequisites

Ensure you have `gnaw` installed. Familiarity with
[TOML syntax](https://toml.io/en/) is helpful but not required.

## What is .c2pconfig?

The `.c2pconfig` file is a configuration file written in **TOML** format. When
you run `gnaw`, it automatically searches for this file in your current working
directory.

It allows you to define:

- **Filtering Rules:** persistent include/exclude patterns.
- **Output Formats:** default to JSON, Markdown, or XML.
- **Template Context:** pre-define variables for your Handlebars templates.

## Quick Start

Create a file named `.c2pconfig` at the root of your project:

{% code(title=".c2pconfig") %}
```toml
default_output = "stdout" # Options: stdout, clipboard, file
include_patterns = ["src/**/*.rs", "Cargo.toml"]
exclude_patterns = ["**/target/**", "tests/fixtures/**"]
line_numbers = true
output_format = "markdown"

[user_variables]
project_name = "MyAwesomeProject"
author = "Developer"
```
{% end %}

## Configuration Reference

| Key | Type | Description |
| --- | --- | --- |
| `path` | String | Default path to codebase (usually `.`). |
| `include_patterns` | Array | Glob patterns of files to include. |
| `exclude_patterns` | Array | Glob patterns of files to exclude. |
| `line_numbers` | Boolean | If `true`, adds line numbers to code blocks. |
| `absolute_path` | Boolean | Use absolute paths instead of relative paths. |
| `full_directory_tree` | Boolean | Generate the full tree even for excluded files. |
| `output_format` | String | `markdown`, `json`, or `xml`. |
| `sort_method` | String | `name_asc`, `name_desc`, `date_asc`, `date_desc`. |
| `encoding` | String | Tokenizer: `cl100k`, `p50k`, `o200k`. |
| `diff_enabled` | Boolean | Include git diff (HEAD vs Index). |
| `token_map_enabled` | Boolean | Display a hierarchical token usage map. |

## Understanding Precedence

{% aside(kind="tip", title="Priority Order") %}
**CLI Arguments > Configuration File > Default Settings.**
Arguments passed directly via the CLI override values defined in `.c2pconfig`,
and CLI include/exclude patterns are merged onto config-file patterns. This
lets you maintain a "base" config while remaining flexible for one-off
commands.
{% end %}

## Example: The "Review-Ready" Config

{% code(title=".c2pconfig") %}
```toml
default_output = "clipboard"
line_numbers = true
token_map_enabled = true

exclude_patterns = [
    "tests/**",
    "**/migrations/**",
    "*.md"
]

[user_variables]
review_focus = "Check for DRY principle violations and complexity."
```
{% end %}

## Next Steps

Explore [Custom Templates](/tutorials/learn-templating/) to see how to use
`user_variables` effectively, and the [Filtering Guide](/tutorials/learn-filtering/)
for advanced glob pattern syntax.
