---
name: verify-website
description: Verify changes under website/.
---

# Verify website

From `website/`, run `pnpm install --frozen-lockfile` only when dependencies are missing
or package metadata changed, then run:

```bash
pnpm build
```

Check changed pages for broken links or routes, incorrect examples, navigation changes,
relevant localized copies, and visual regressions when layout or styling changed.

Do not commit `website/dist/`. Report build or environment failures explicitly.
