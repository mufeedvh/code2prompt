---
name: verify-cli
description: Verify changes to the code2prompt CLI or TUI.
---

# Verify CLI / TUI

Run from the repository root:

```bash
cargo test
cargo clippy --all-targets --all-features
```

For changed flags, configuration, or output behavior, also run or add a representative
integration test and compare the CLI with:

```bash
cargo run --quiet -- --help
```

Check CLI-over-config precedence, stdout versus stderr, quiet mode, and that TUI state
stays in the model/session rather than widgets. Use temporary fixtures, not personal
repositories. Report failed checks explicitly.
