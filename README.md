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

**Code2Prompt** is a powerful context engineering tool designed to ingest codebases and format them for Large Language Models (LLMs). Whether you are manually copying context for ChatGPT, building AI agents via Python, or running a Model Context Protocol (MCP) server, Code2Prompt streamlines the data preparation process.

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

**Core**

`code2prompt-core` is a code ingestion library that streamlines the process of creating LLM prompts for code analysis, generation, and other tasks. It works by traversing directories, building a tree structure, and gathering information about each file. The core library can easily be integrated into other applications.

**CLI**

`code2prompt` command line interface (CLI) was designed for humans to generate prompts directly from your codebase. The generated prompt is automatically copied to your clipboard and can also be saved to an output file. Furthermore, you can customize the prompt generation using Handlebars templates. Check out the provided prompts in the doc !

**SDK**

`code2prompt` software development kit (SDK) offers python binding to the core library. This is perfect for AI agents or automation scripts that want to interact with codebase seamlessly. The SDK is hosted on Pypi and can be installed via pip.

**MCP**

`code2prompt` is also available as a Model Context Protocol (MCP) server, which allows you to run it as a local service. this enables agentic applications to read your local codebase efficiently without bloating your context window.

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
