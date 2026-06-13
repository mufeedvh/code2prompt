+++
title = "Token map"
description = "Print a token-weighted tree of the repo to find exactly what's inflating your prompt."
weight = 40
+++

`--token-map` prints a directory tree to the terminal with a token weight on
every file and folder, so instead of guessing what's bloating a 291k-token run
you can see it:

```sh
gnaw . --token-map
```

```text
└── src/
    ├── components/   180,000 tokens
    ├── lib.rs          4,200 tokens
    └── main.rs           900 tokens
```

The weight on a directory is the sum of its contents, so the heavy subtrees are
obvious at a glance — and that's usually your cue for what to `--exclude` or
[compress](/how-to/compress-prompts/) next.

## Tuning the output

| Flag | Effect |
| --- | --- |
| `--token-map` | Enable the token-weighted tree |
| `--token-map-lines <N>` | Cap the tree at roughly `N` lines |
| `--token-map-min-percent <P>` | Hide entries below `P` percent of the total |

```sh
# Top of the tree only, hiding anything under 1% of the total
gnaw . --token-map --token-map-lines 40 --token-map-min-percent 1
```

`--token-map-min-percent` is the quickest way to cut noise: set it to `1` and the
map collapses to just the files and folders actually worth your attention.

{% aside(kind="tip", title="The same idea, interactively") %}
The TUI's File tree tab does this live: `:sort tokens` reorders by the same
weights and directories show their aggregate cost as you select. The
[interactive selection guide](/how-to/interactive-selection/) walks through using
it to trim to a budget.
{% end %}
