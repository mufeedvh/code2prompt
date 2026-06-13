+++
title = "Shrink a prompt with compression"
description = "Use gnaw's syntax-aware compression to cut token count by stripping bodies, tests, and doc comments while keeping signatures."
weight = 20
+++

When a repo is too large for your budget, compression strips the heavy parts of
your Rust source — function bodies, test modules, doc comments — while keeping
the structure a model needs: signatures, types, and the shape of each file.
Because it runs **per file before token counting**, the counts gnaw reports
already reflect the compressed size, so you can see the saving immediately.

{% aside(kind="caution", title="Rust-only, and feature-gated") %}
Compression is **Rust-only** today — it parses `.rs` files with tree-sitter, and
any other file type passes through untouched. It also lives behind the
`compression` build feature; a binary built without it treats every compression
flag as a no-op. If your output isn't shrinking, check both.
{% end %}

## Pick a level

`--compress` takes one of three presets, ordered by how much they remove. The
names describe aggressiveness, not quality.

| Level | Removes | Keeps |
| --- | --- | --- |
| `light` | `#[cfg(test)]` modules | Everything else |
| `moderate` | + function and method bodies (→ `{ /* ... */ }`) | Signatures, types, doc comments |
| `full` | + doc comments | Signatures and types |

```sh
gnaw . --compress moderate
gnaw . --compress full -O context.md
```

A stripped body becomes `{ /* ... */ }`; a stripped test module becomes a
`// [test module stripped]` marker, so the output stays valid-looking and
obviously elided.

## Fine-tune with `--compress-strip`

`--compress-strip` takes a comma-separated list of toggles that apply **on top
of** the preset (or on their own, with no preset). Prefix any token with `no-`
to turn it off.

| Token | Effect |
| --- | --- |
| `tests` | Strip `#[cfg(test)]` modules |
| `fn-bodies` | Strip all function/method bodies |
| `doc-comments` | Strip `///` and `/** */` doc comments |
| `private-bodies` | Strip bodies of **non-`pub`** functions only, keeping public ones |

```sh
# Full, but keep the doc comments
gnaw . --compress full --compress-strip no-doc-comments

# No preset — just drop tests and private bodies
gnaw . --compress-strip tests,private-bodies
```

{% aside(kind="tip", title="The public-API view") %}
`private-bodies` is the one toggle no preset turns on. It strips the bodies of
unexported functions while leaving `pub` ones intact — a "here's the public
surface, with internals elided" view. It only does anything when `fn-bodies` is
off (otherwise every body goes anyway), so reach for it on its own:
`--compress-strip tests,private-bodies`.
{% end %}

Unknown tokens are rejected with a suggestion rather than ignored — a silent typo
would quietly misreport your token budget, so gnaw fails loudly instead.

## Make it the default

Set compression once in `.gnawconfig` (in the working directory, or globally at
`~/.config/gnaw/.gnawconfig`) and every run picks it up:

```toml
[compression]
strip_test_modules = true
strip_fn_bodies = true
strip_doc_comments = false
strip_private_bodies = false
```

Precedence is **`--compress` preset → config `[compression]` → none**, with any
`--compress-strip` toggles layered on top of whichever base applies. So a preset
on the command line overrides the config file for that run, and `--compress-strip`
can still adjust the result.

## Check the saving

Pair compression with the [token map](/reference/token-map/) to see where the
tokens went, or just read the per-run total — since compression happens before
counting, the number you see is the compressed cost.
