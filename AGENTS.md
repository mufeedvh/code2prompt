# code2prompt agent guide

Project-specific skills live in `.agents/skills/`.

## Rules

- Follow existing code2prompt patterns before generic best practices.
- Put reusable product logic in `code2prompt-core`.
- Keep CLI/TUI concerns in `code2prompt`.
- Keep Python bindings thin: delegate behavior to the core.
- `Code2PromptSession` coordinates configuration, selection, and loaded `SessionData`.
- Keep the website as a consumer of product behavior, not a second implementation.
- Prefer small changes and existing dependencies.
- Preserve public behavior unless the task explicitly changes it.
- Bug fixes should include a regression test when practical.
- Comments should explain non-obvious intent, constraints, or tradeoffs, not narrate code.
- Do not claim completion until the relevant verification skill passes.

## Skills

- Implementation/refactoring → `code2prompt-conventions`
- `crates/code2prompt-core/**` → `verify-core`
- `crates/code2prompt/**` → `verify-cli`
- `crates/code2prompt-python/**` → `verify-python`
- `website/**` → `verify-website`
- Public behavior/docs/comments → `documentation`

A change can require several verification skills. Root `cargo test` does not exercise
the Python bindings.
