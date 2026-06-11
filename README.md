<div align="center">
  <a href="https://gnaw.gitbadger.com">
    <img align="center" width="550px" src="website/static/wordmark.svg" alt="gnaw"/>
  </a>
  <br>
  <h3>Convert your codebase into a single LLM prompt.</h3>
  <p><sub>A Rust-native fork of <a href="https://github.com/mufeedvh/code2prompt">code2prompt</a>, extended with syntax-aware compression, a REST surface, and more.</sub></p>
</div>

<p align="center">
  <a href="https://gnaw.gitbadger.com"><b>Website</b></a> •
  <a href="https://gnaw.gitbadger.com/how-to/install/"><b>Documentation</b></a>
</p>

<div align="center">

[![License](https://img.shields.io/github/license/gitbadger/gnaw.svg?style=flat-square)](https://github.com/gitbadger/gnaw/blob/main/LICENSE)
[![CI](https://github.com/gitbadger/gnaw/actions/workflows/ci.yml/badge.svg?style=flat-square)](https://github.com/gitbadger/gnaw/actions)
[![GitHub Stars](https://img.shields.io/github/stars/gitbadger/gnaw?style=social)](https://github.com/gitbadger/gnaw)

</div>

<!-- Badges to add once published:
[![Crates.io](https://img.shields.io/crates/v/gnaw-ctx.svg?style=flat-square)](https://crates.io/crates/gnaw-ctx)
[![Docs.rs](https://docs.rs/gnaw-core/badge.svg?style=flat-square)](https://docs.rs/gnaw-core)
-->

---

<!-- TODO: add a demo gif at website/static/demo.gif and uncomment
<h1 align="center">
  <a href="https://gnaw.gitbadger.com"><img src="website/static/demo.gif" alt="gnaw demo"></a>
</h1>
-->

**gnaw** is a powerful context engineering tool designed to ingest codebases and format them for Large Language Models. Whether you are manually copying context for a chat assistant, building AI agents via Python, or wiring up a browser extension over REST, gnaw streamlines the context preparation process.

## ⚡ Quick Install

### Cargo

```bash
cargo install gnaw-ctx
```

The crates.io package is `gnaw-ctx`; the installed binary is `gnaw`.

To enable optional Wayland support (e.g., for clipboard integration on Wayland-based systems), use the `wayland` feature flag:

```bash
cargo install --features wayland gnaw-ctx
```

<!-- Homebrew — uncomment once a tap/formula is published:
### Homebrew

```bash
brew install gnaw
```
-->

### Python bindings 🐍

Built with PyO3/maturin. Not yet published to PyPI — build from source (see [Alternative Installation](#alternative-installation)).

## 🚀 Quick Start

Once installed, generating a prompt from your codebase is as simple as pointing the tool to your directory.

**Basic Usage**: Generate a prompt from the current directory and copy it to the clipboard.

```sh
gnaw .
```

**Save to file**:

```sh
gnaw path/to/project --output-file prompt.txt
```

## 🌐 Ecosystem

gnaw is more than just a CLI tool. It is a complete ecosystem for codebase context.

| 🧱 Core Library | 💻 CLI / TUI | 🐍 Python | 🌐 REST | 🤖 MCP |
| :---: | :---: | :---: | :---: | :---: |
| `gnaw-core` — the internal, high-speed library responsible for secure file traversal, respecting `.gitignore` rules, and structuring Git metadata. | Designed for humans, featuring both a minimal CLI and an interactive TUI. Generate formatted prompts, track token usage, and output the result to your clipboard or stdout. | Fast Python bindings to the Rust core. Ideal for AI agents, automation scripts, or deep integration into RAG pipelines. | A planned axum-based REST interface, enabling browser extensions and other clients to request well-structured context over HTTP. | A planned MCP server, letting agentic applications call gnaw as a tool to read your codebase without bloating their context window. |

## 📚 Documentation

Check our online [documentation](https://gnaw.gitbadger.com/how-to/install/) for detailed instructions.

## ✨ Features

gnaw transforms your entire codebase into a well-structured prompt for large language models. Key features include:

- **Terminal User Interface (TUI)**: Interactive terminal interface for configuring and generating prompts
- **Smart Filtering**: Include/exclude files using glob patterns and respect `.gitignore` rules
- **Flexible Templating**: Customize prompts with Handlebars templates for different use cases
- **Syntax-Aware Compression**: Chunk on whole functions and types via tree-sitter, not arbitrary line cuts
- **Token Tracking**: Track token usage to stay within LLM context limits
- **Git Integration**: Include diffs, logs, and branch comparisons in your prompts
- **Blazing Fast**: Built in Rust for high performance and low resource usage

Stop manually copying files and formatting code for LLMs. gnaw handles the tedious work so you can focus on getting insights and solutions from AI models.

## Alternative Installation

Refer to the [documentation](https://gnaw.gitbadger.com/how-to/install/) for detailed installation instructions.

### Binary releases

Download the latest binary for your OS from [Releases](https://github.com/gitbadger/gnaw/releases).

### Source build

Requires [Git](https://git-scm.com/downloads), [Rust](https://www.rust-lang.org/tools/install) and `Cargo`.

```sh
git clone https://github.com/gitbadger/gnaw.git
cd gnaw/
cargo install --path crates/gnaw
```

## ⭐ Star Gazing

[![Star History Chart](https://api.star-history.com/svg?repos=gitbadger/gnaw&type=Date)](https://star-history.com/#gitbadger/gnaw&Date)

## 🍴 Forked from code2prompt

gnaw began as a fork of [code2prompt](https://github.com/mufeedvh/code2prompt) by [Mufeed VH](https://github.com/mufeedvh) and contributors, and owes its foundation to that project. It carries forward the core idea — turning a codebase into a single, well-structured LLM prompt — while taking the tooling in a Rust-native direction and adding new capabilities:

- **Syntax-aware compression** — chunk on whole functions and types via tree-sitter, rather than arbitrary line cuts
- **REST interface** *(planned)* — an axum surface so browser extensions and other clients can request context over HTTP
- **MCP server** *(planned)* — expose gnaw as a tool for agentic applications

The original code2prompt is MIT licensed. gnaw is dual-licensed under MIT OR Apache-2.0; portions derived from code2prompt remain under the upstream MIT license, whose copyright notice is preserved. See [License](#-license) for details.

## 📜 License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/gitbadger/gnaw/blob/main/LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](https://github.com/gitbadger/gnaw/blob/main/LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

Portions of this project are derived from [code2prompt](https://github.com/mufeedvh/code2prompt) and remain under its original MIT license; that copyright notice is retained in [LICENSE-MIT](https://github.com/gitbadger/gnaw/blob/main/LICENSE-MIT).

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## Liked the project?

If you liked the project and found it useful, please give it a :star:!

## 👥 Contribution

Ways to contribute:

- Suggest a feature
- Report a bug
- Fix something and open a pull request
- Help document the code
- Spread the word
