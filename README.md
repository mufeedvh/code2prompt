<div align="center">
  <a href="https://code2prompt.dev">
  <img align="center" width="550px" src=".assets/logo_dark_v0.0.2.svg" alt="Code2prompt"/>
  </a>
  <p align="center">
  <b>Convert your codebase into a single LLM prompt !</b>
  </p>
</div>

<h1 align="center">
  <a href="https://code2prompt.dev"><img src=".assets/CLI.png" alt="code2prompt"></a>
</h1>

<hr />

[![crates.io](https://img.shields.io/crates/v/code2prompt.svg)](https://crates.io/crates/code2prompt)
[![LICENSE](https://img.shields.io/github/license/mufeedvh/code2prompt.svg#cache1)](https://github.com/mufeedvh/code2prompt/blob/master/LICENSE)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/user/repo/pulls)
[![Lines of Code](https://tokei.rs/b1/github/mufeedvh/code2prompt)](https://github.com/mufeedvh/code2prompt)
[![Discord](https://img.shields.io/discord/1342336677905039451?logo=discord)](https://discord.com/invite/zxVMKtsFbK)

<hr />

<p align="center">
  Want to engage with us ? Join our <a href="https://discord.com/invite/zxVMKtsFbK">Discord</a> channel!<br>
  <i>Stay updated on new features</i> 📢<br>
  <i>Give your insight and suggestion</i> 💬<br>
  <i>Get help with configuration and usage</i> 🛠️<br>
  <i>Report Bug</i> 🐛<br>
</p>

## Quick Install ⚡

CLI with cargo 🦀

```bash
cargo install code2prompt
```

SDK with pip 🐍

```bash
pip install code2prompt-rs
```

## How is it useful?

**Core**

`code2prompt` is a code ingestion tool that streamline the process of creating LLM prompts for code analysis, generation, and other tasks. It works by traversing directories, building a tree structure, and gathering informations about each file. The core library can easily be integrated into other applications.

**CLI**

`code2prompt` command line interface (CLI) was designed for humans to generate prompts directly from your codebase. The generated prompt is automatically copied to your clipboard and can also be saved to an output file. Furthermore, you can customize the prompt generation using Handlebars templates. Check out the provided prompts in the doc !

**SDK**

`code2prompt` software development kit (SDK) offers python binding to the core library. This is perfect for AI agents or automation scripts that want to interact with codebase seamlessly. The SDK is hosted on Pypi and can be installed via pip.

**MCP**

`code2prompt` is also available as a Model Context Protocol (MCP) server, which allows you to run it as a local service. This enables LLMs on steroids by providing them a tool to automatically gather a well-structured context of your codebase.

## Documentation 📚

Check our online [documentation](https://code2prompt.dev/docs/welcome/) for detailed instructions

## Features

Code2Prompt transforms your entire codebase into a well-structured prompt for large language models. Key features include:

- **Automatic Code Processing**: Convert codebases of any size into readable, formatted prompts
- **Smart Filtering**: Include/exclude files using glob patterns and respect `.gitignore` rules
- **Flexible Templating**: Customize prompts with Handlebars templates for different use cases
- **Token Tracking**: Track token usage to stay within LLM context limits
- **Git Integration**: Include diffs, logs, and branch comparisons in your prompts
- **Developer Experience**: Automatic clipboard copy, line numbers, and file organization options

Stop manually copying files and formatting code for LLMs. Code2Prompt handles the tedious work so you can focus on getting insights and solutions from AI models.

## Alternative Installation

Refer to the [documentation](https://code2prompt.dev/docs/welcome/) for detailed installation instructions.

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

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=mufeedvh/code2prompt&type=Date)](https://star-history.com/#mufeedvh/code2prompt&Date)

## License

Licensed under the MIT License, see <a href="https://github.com/mufeedvh/code2prompt/blob/master/LICENSE">LICENSE</a> for more information.

## Liked the project?

If you liked the project and found it useful, please give it a :star: !

## Contribution

Ways to contribute:

- Suggest a feature
- Report a bug  
- Fix something and open a pull request
- Help me document the code
- Spread the word
