<div align="center">
  <a href="https://code2prompt.dev">
    <img align="center" width="550px" src=".assets/logo_dark_v0.0.2.svg" alt="Code2prompt"/>
  </a>
  <br>
  <h3>Convert your codebase into a single LLM prompt.</h3>
</div>

<p align="center">
  <a href="https://code2prompt.dev"><b>Website</b></a> •
  <a href="https://code2prompt.dev/docs/welcome/"><b>Documentation</b></a> •
  <a href="https://discord.com/invite/ZZyBbsHTwH"><b>Discord</b></a>
</p>

<div align="center">

[![License](https://img.shields.io/github/license/mufeedvh/code2prompt.svg?style=flat-square)](https://github.com/mufeedvh/code2prompt/blob/master/LICENSE)
[![Crates.io](https://img.shields.io/crates/v/code2prompt.svg?style=flat-square)](https://crates.io/crates/code2prompt)
[![PyPI](https://img.shields.io/pypi/v/code2prompt-rs?style=flat-square&logo=pypi&logoColor=white)](https://pypi.org/project/code2prompt-rs/)
[![CI](https://github.com/mufeedvh/code2prompt/actions/workflows/ci.yml/badge.svg?style=flat-square)](https://github.com/mufeedvh/code2prompt/actions)
[![Discord](https://img.shields.io/discord/1342336677905039451?style=flat-square&logo=discord&logoColor=white)](https://discord.com/invite/ZZyBbsHTwH)
[![Docs.rs](https://docs.rs/code2prompt-core/badge.svg?style=flat-square)](https://docs.rs/code2prompt-core)
[![Crates.io Downloads](https://img.shields.io/crates/d/code2prompt.svg?style=flat-square)](https://crates.io/crates/code2prompt)
[![GitHub Stars](https://img.shields.io/github/stars/mufeedvh/code2prompt?style=social)](https://github.com/mufeedvh/code2prompt)

</div>

---

<h1 align="center">
  <a href="https://code2prompt.dev"><img src=".assets/demo.gif" alt="code2prompt demo"></a>
</h1>

![Flow Diagram](.assets/flow_diagram.png)

**Code2Prompt** is a powerful context engineering tool designed to ingest codebases and format them for Large Language Models. Whether you are manually copying context for ChatGPT, building AI agents via Python, or running a MCP server, Code2Prompt streamlines the context preparation process.

## ⚡ Quick Install

### Cargo

```bash
cargo install code2prompt 
```

To enable optional Wayland support (e.g., for clipboard integration on Wayland-based systems), use the `wayland` feature flag:

```bash
cargo install --features wayland code2prompt
```

### Homebrew

```bash
brew install code2prompt
```

### SDK with pip 🐍

```bash
pip install code2prompt-rs
```

## 🚀 Quick Start

Once installed, generating a prompt from your codebase is as simple as pointing the tool to your directory.

**Basic Usage**: Generate a prompt from the current directory and copy it to the clipboard.

```sh
code2prompt .
```

**Save to file**:

```sh
code2prompt path/to/project --output prompt.txt
```

## 🌐 Ecosystem

Code2Prompt is more than just a CLI tool. It is a complete ecosystem for codebase context.

| 🧱 Core Library <br><br><img src="https://img.shields.io/badge/Rust-FF6700?style=for-the-badge&logo=rust&logoColor=white" alt="Rust Core Badge"/>| 💻 CLI Tool <br><img src="https://img.shields.io/badge/Terminal-2C3E50?style=for-the-badge&logo=gnu-bash&logoColor=white" alt="CLI Badge"/> | 🐍 Python SDK <br><img src="https://img.shields.io/badge/Python-3776AB?style=for-the-badge&logo=python&logoColor=white" alt="Python SDK Badge"/> | 🤖 MCP Server <img src="https://img.shields.io/badge/Agentic%20Flow-7E57C2?style=for-the-badge&logo=server&logoColor=white" alt="MCP Server Badge"/> |
| :---: | :---: | :---: | :---: |
| The internal, high-speed library responsible for secure file traversal, respecting `.gitignore` rules, and structuring Git metadata. | Designed for humans, featuring both a minimal CLI and an interactive TUI. Generate formatted prompts, track token usage, and outputs the result to your clipboard or stdout. | Provides fast Python bindings to the Rust Core. Ideal for AI Agents, automation scripts, or deep integration into RAG pipelines. Available on PyPI. | Run Code2Prompt as a local service, enabling agentic applications to read your local codebase efficiently without bloating your context window. |

## 📚 Documentation

Check our online [documentation](https://code2prompt.dev/docs/welcome/) for detailed instructions

## ✨ Features

Code2Prompt transforms your entire codebase into a well-structured prompt for large language models. Key features include:

- **Terminal User Interface (TUI)**: Interactive terminal interface for configuring and generating prompts
- **Smart Filtering**: Include/exclude files using glob patterns and respect `.gitignore` rules
- **Flexible Templating**: Customize prompts with Handlebars templates for different use cases
- **Automatic Code Processing**: Convert codebases of any size into readable, formatted prompts
- **Token Tracking**: Track token usage to stay within LLM context limits
- **Smart File Reading**: Simplify reading various file formats for LLMs (CSV, Notebooks, JSONL, etc.)
- **Git Integration**: Include diffs, logs, and branch comparisons in your prompts
- **Blazing Fast**: Built in Rust for high performance and low resource usage

Stop manually copying files and formatting code for LLMs. Code2Prompt handles the tedious work so you can focus on getting insights and solutions from AI models.

## Alternative Installation

Refer to the [documentation](https://code2prompt.dev/docs/how_to/install/) for detailed installation instructions.

### Binary releases

Download the latest binary for your OS from [Releases](https://github.com/mufeedvh/code2prompt/releases).

### Source build

Requires:

- [Git](https://git-scm.org/downloads), [Rust](https://rust-lang.org/tools/install) and `Cargo`.

```sh
git clone https://github.com/mufeedvh/code2prompt.git
cd code2prompt/
cargo install --path crates/code2prompt
```

## ⭐ Star Gazing

[![Star History Chart](https://api.star-history.com/svg?repos=mufeedvh/code2prompt&type=Date)](https://star-history.com/#mufeedvh/code2prompt&Date)

## 📜 License

Licensed under the MIT License, see <a href="https://github.com/mufeedvh/code2prompt/blob/master/LICENSE">LICENSE</a> for more information.

## Liked the project?

If you liked the project and found it useful, please give it a :star: !

## 👥 Contribution

Ways to contribute:

- Suggest a feature
- Report a bug
- Fix something and open a pull request
- Help me document the code
- Spread the word
