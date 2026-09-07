---
name: code2prompt-conventions
description: Follow code2prompt architecture and implementation conventions when changing or reviewing code.
metadata:
  internal: true
---

# code2prompt conventions

Before changing an unfamiliar area, read the target code, a nearby example, and relevant tests.

## Ownership

- `code2prompt-core`: reusable logic, configuration, files, Git, templates, tokenization, analysis, and sessions.
- `code2prompt`: CLI, config precedence, stdout/stderr, clipboard, and TUI model/views/widgets.
- `code2prompt-python`: thin PyO3 adapter over `code2prompt-core`.
- `website`: site and product documentation, not product semantics.

Implement behavior once, in the lowest appropriate layer.

`Code2PromptSession` coordinates configuration, selection state, and loaded data.
`SessionData` is the source of truth for loaded codebase and Git results, not for all
session state.

## Preserve

- deterministic output where order is visible;
- optional-feature behavior with and without the feature;
- intended fallbacks and useful error context;
- stdout for prompt/data output and stderr for human diagnostics;
- existing public names, defaults, serialized values, CLI behavior, and Python API unless intentionally changed.

## Style

- Prefer existing abstractions and dependencies.
- Avoid unrelated refactors.
- Use `Result`/`?`; add context at filesystem, Git, parsing, and rendering boundaries.
- Avoid runtime panics for recoverable failures.
- Comments explain why, invariants, compatibility, or non-obvious algorithms.
- Put focused private Rust tests in local `#[cfg(test)] mod tests`; use integration tests for public behavior.
