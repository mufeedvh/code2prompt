+++
title = "Git diffs and logs"
description = "Inject working-tree diffs, branch diffs, and branch logs into the prompt with gnaw's git flags."
weight = 30
+++

gnaw can fold live git context into the output so a model sees what you're
working on now and how the code has moved, not just a static snapshot. There are
three independent sources, each surfaced as its own section in the rendered
prompt.

## Working-tree diff

`-d` / `--diff` injects the current diff. `--diff-mode` chooses which changes,
and **requires `--diff`** to be set.

| `--diff-mode` | Diff shown |
| --- | --- |
| `staged` *(default)* | Staged changes only |
| `unstaged` | Unstaged working-tree changes |
| `all` | All uncommitted changes |

```sh
gnaw . --diff                      # staged (default)
gnaw . --diff --diff-mode unstaged
gnaw . --diff --diff-mode all
```

This populates the `git_diff` template variable, rendered under a `<git-diff>`
tag (XML) or a `Git Diff:` heading (Markdown).

## Diff between two branches

`--git-diff-branch` takes **two** refs (comma- or space-separated) and renders the
diff between them:

```sh
gnaw . --git-diff-branch main,feature/login
gnaw . --git-diff-branch v1.0.0 HEAD
```

This populates `git_diff_branch`.

## Log between two branches

`--git-log-branch` likewise takes two refs and injects the commit log across that
range:

```sh
gnaw . --git-log-branch main,feature/login
```

This populates `git_log_branch`.

{% aside(kind="note", title="Three separate variables") %}
`git_diff`, `git_diff_branch`, and `git_log_branch` are distinct and can all
appear in one run. A custom template can reference each independently with
`{{#if git_diff}}…{{/if}}` and the matching names — handy if you want, say, the
branch log but not the working-tree diff.
{% end %}

{% aside(kind="tip", title="Pairs well with the commit-splitting template") %}
The built-in template for proposing an atomic commit sequence expects a diff —
run it with `--diff --diff-mode unstaged` (or `all`) so there's a changeset to
work from.
{% end %}
