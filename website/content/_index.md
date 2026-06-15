+++
title = "gnaw Documentation"
description = "Official gnaw documentation"
template = "index.html"
sort_by = "weight"

[extra.hero]
title = "gnaw"
tagline = "Transform your code into AI-optimized prompts in seconds"
image = "logo-dark.svg"
[[extra.hero.actions]]
text = "Get Started 🚀"
link = "/tutorials/getting-started/"
icon = "right-arrow"
variant = "primary"
[[extra.hero.actions]]
text = "Installation 📥"
link = "/how-to/install/"
variant = "minimal"
+++

## Quick Start

{{ linkcard(title="Getting Started 🚀", href="/tutorials/getting-started/") }}
{{ linkcard(title="Installation 📥", href="/how-to/install/") }}
{{ linkcard(title="Learn Filtering 🔍", href="/tutorials/learn-filtering/") }}
{{ linkcard(title="Learn Templating 📝", href="/tutorials/learn-templating/") }}
{{ linkcard(title="Vision 🔮", href="/vision/") }}

`gnaw` is a powerful code ingestion tool designed to generate prompts for code
analysis, generation, and other tasks. It works by traversing directories,
building a tree structure, and gathering information about each file.

It simplifies the process of combining and formatting code, making it easy to
analyze, document, or refactor code using LLMs.

You can use `gnaw` the following ways:

{% cardgrid() %}
{% card(title="Core", icon="setting") %}
Core library, blazingly fast code ingestion
{% end %}
{% card(title="CLI / TUI", icon="document") %}
Command line and terminal interface specially designed for humans
{% end %}
{% card(title="Python", icon="add-document") %}
Python bindings for AI agents and automation scripts
{% end %}
{% card(title="REST", icon="external") %}
REST interface for browser-extension integration (planned)
{% end %}
{% end %}

## Key Features

- **Generate LLM Prompts**: Quickly convert entire codebases into structured LLM prompts.
- **Glob Pattern Filtering**: Include or exclude specific files and directories using glob patterns.
- **Customizable Templates**: Tailor prompt generation with Handlebars templates.
- **Token Counting**: Analyze token usage and optimize for LLMs with varying context windows.
- **[Secret Scanning](/how-to/scan-for-secrets/)**: Catch API keys, tokens, and private keys before they leak into a prompt — warn, redact, or block.
- **Git Integration**: Include Git diffs and commit messages in prompts for code reviews.
- **Respects `.gitignore`**: Automatically ignores files listed in `.gitignore` to streamline prompt generation.
- **[Shell Completion](/how-to/shell-completion/)**: Tab-complete flags, values, and template names in bash, zsh, and fish.

## Why `gnaw`?

1. **Save Time**: Automates the process of traversing a codebase and formatting files for LLMs, and avoids repetitive copy-pasting of code.
2. **Improve Productivity**: Provides a structured and consistent format for code analysis, and helps identify bugs, refactor code, and write documentation faster.
3. **Handle Large Codebases**: Designed to work seamlessly with large codebases, respecting context limits of LLMs.
4. **Customizable Workflows**: Flexible options for filtering files, using templates, and generating targeted prompts.

## Example Use Cases

- **Code Documentation**: Automatically generate documentation for public functions, methods, and classes.
- **Bug Detection**: Find potential bugs and vulnerabilities by analyzing your codebase with LLMs.
- **Refactoring**: Simplify and optimize code by generating prompts for code quality improvements.
- **Learning and Exploration**: Understand new codebases by generating summaries and detailed breakdowns.
- **Git Commit and PR Descriptions**: Generate meaningful commit messages and pull request descriptions from Git diffs.
- **Safe Sharing**: Scrub secrets out of context before pasting it into a chat, so credentials never leave your machine.
