+++
title = "Install gnaw"
description = "A complete installation guide for gnaw on different operating systems."
weight = 1
+++

{% aside(kind="note", title="Guide Overview") %}
Welcome to the `gnaw` installation guide. This document provides step-by-step
instructions for installing it on various platforms, including Windows, macOS,
and Linux.
{% end %}

**TL;DR**

{% code(title="bash") %}
```bash
cargo install gnaw-ctx
```
{% end %}

## Prerequisite

Make sure [Rust](https://www.rust-lang.org/tools/install) and cargo are
installed on your system.

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

This is the official way to install the latest stable version of Rust and
Cargo. Make sure to refresh your `PATH` variable after installing Rust. Restart
your terminal or run the instructions proposed by the installer.

```sh
source $HOME/.cargo/env
```

You can check that everything is installed correctly by running:

```sh
cargo --version
git --version
```

## Command Line Interface (CLI) 👨‍💻

{% code(title="bash") %}
```bash
cargo install gnaw-ctx
```
{% end %}

{% aside(kind="caution") %}
The crates.io package is `gnaw-ctx`; the installed binary is `gnaw`.
{% end %}

### 🧪 Install the latest (unpublished) version from GitHub

If you want the latest features or fixes before they're released on crates.io:

```sh
cargo install --git https://github.com/gitbadger-clan/gnaw
```

### Source build

Ideal for developers that want to build from source or contribute to the
project.

1. 🛠️ Install prerequisites: [Rust](https://www.rust-lang.org/tools/install) and Cargo, plus [Git](https://git-scm.com/downloads).
2. 📥 Clone the repository:
   ```sh
   git clone https://github.com/gitbadger-clan/gnaw.git
   cd gnaw
   ```
3. 📦 Install the binary. To build and install from source:
   ```sh
   cargo install --path crates/gnaw
   ```
   To build the binary without installing it:
   ```sh
   cargo build --release
   ```
   The binary will be available in the `target/release` directory.
4. 🚀 Run it:
   ```sh
   gnaw --help
   ```

### Binary releases

Best for users that want to use the latest version without building from
source. Download the latest binary for your OS from
[Releases](https://github.com/gitbadger/gnaw/releases).

{% aside(kind="caution") %}
Binary releases may lag behind the latest GitHub version. For cutting-edge
features, consider building from source.
{% end %}

## Python bindings 🐍

The Python package is built with PyO3/maturin and is not yet published to PyPI.
To build it from source:

1. 🛠️ Install prerequisites: [Rust](https://www.rust-lang.org/tools/install) and Cargo, [Git](https://git-scm.com/downloads), and [maturin](https://www.maturin.rs/).
2. 📥 Clone the repository:
   ```sh
   git clone https://github.com/gitbadger/gnaw.git
   cd gnaw/crates/gnaw-python
   ```
3. ⚙️ Build the package into your virtual environment:
   ```sh
   maturin develop -r
   ```

## REST interface 🌐

A REST interface for browser-extension integration is planned and will be
documented here when it ships.
