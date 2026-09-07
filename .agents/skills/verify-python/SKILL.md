---
name: verify-python
description: Verify changes to the code2prompt Python bindings.
metadata:
  internal: true
---

# Verify Python

From `crates/code2prompt-python`:

```bash
uv sync
uv run pytest --quiet
```

Keep Python a thin adapter over the core. Preserve public Python names and types unless
intentionally changed. Report the Python version and failed checks explicitly.
