+++
title = "Split a big repo across files"
description = "Use --split-size to slice an oversized prompt into multiple token-budgeted parts you can paste one at a time."
weight = 30
+++

When the whole repo won't fit in a single message, gnaw can slice the output
into several self-contained parts, each kept under a token budget you choose. You
paste them one at a time without tripping a model's per-message limit.

## The basics

`--split-size` takes a **token** budget and needs `--output-file` as a base name:

```sh
gnaw . --split-size 50000 -O context.md
```

That writes `context.part1.md`, `context.part2.md`, and so on — the base name's
stem and extension with `.partN` inserted before the extension. Each part is a
complete, independently pasteable document rendered through the normal template,
so every part carries its own structure and is valid on its own.

{% aside(kind="caution", title="Tokens, not bytes") %}
The budget is in **tokens**, not file size — there's no `1mb`/`500kb` form.
`--split-size 50000` means "about 50,000 tokens per part," measured with the same
encoding as the rest of the run.
{% end %}

## How files are grouped

gnaw walks the selected files **in order** and fills a part until the next file
would push it over the budget, then starts a new part. It's a simple greedy
bin-pack by token count, preserving file order.

{% aside(kind="note", title="Not grouped by directory (yet)") %}
Parts are packed by token budget in traversal order — they are **not** grouped by
top-level directory. A part can span several directories, and one directory can
land across two parts. Directory-aware grouping is planned, but this is what
ships today, so don't rely on a part mapping to a single folder.
{% end %}

If a **single file is larger than the budget** on its own, it can't be split
further, so it gets its own part and gnaw prints a warning naming the file and
its size. That's expected — lower the budget won't help a lone oversized file;
compress it or exclude it instead.

## Controlling the parts

Because grouping follows file order and token cost, the two levers that matter
are *what's included* and *how big each file is*:

- **Trim first.** Apply [compression](/how-to/compress-prompts/) or tighten your
  `--include`/`--exclude` globs so individual files are smaller and pack better.
- **Find the heavy files.** Run with [`--token-map`](/reference/token-map/) to see
  which files dominate, then decide what to compress or drop before splitting.

```sh
# Compress, then split into ~40k-token parts
gnaw . --compress moderate --split-size 40000 -O review.md
```

{% aside(kind="tip", title="Clipboard is off the table here") %}
`--split-size` is mutually exclusive with `--clipboard` — multiple files can't go
to a single clipboard buffer. Splitting always writes files; copy the part you
need from disk.
{% end %}
