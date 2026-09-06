---
name: verify-python
description: Verify code2prompt Python bindings and packaging.
---

# Verify Python

From `crates/code2prompt-python`:

```bash
uv run --locked pytest
```

Run `uv sync --locked` first when the environment is missing or project metadata changed.

From the repository root:

```bash
cargo clippy -p code2prompt-python --all-targets
```

Build a release wheel only when PyO3 exports or packaging changed:

```bash
cd crates/code2prompt-python
uv run --locked maturin build --release --out ../../dist
```

Keep Python a thin adapter over the core. Preserve public Python names and types unless
intentionally changed. Report the Python version and failed checks explicitly.
