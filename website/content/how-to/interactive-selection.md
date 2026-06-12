+++
title = "Interactive selection in the TUI"
description = "Use the gnaw TUI to browse a repo, filter with substring and glob search, trim to a token budget, pick a template, and copy the result."
weight = 10
+++

The TUI is the fastest way to hand-pick exactly which files land in a prompt and
to watch the token cost update as you go. This guide walks the full loop:
**launch → browse and search → select → trim to budget → template → copy.**

For a flat list of every key, see the [TUI keymap](/reference/tui-keymap/).

## 1. Launch

```sh
gnaw --tui            # current directory
gnaw ./my-repo --tui  # a specific path
```

You land on the **File tree** tab (`1`). The tree is gitignore-aware, so build
artifacts and ignored files are already out of the way. Token counting runs in
the background as you select — the count you see is live, not a one-shot
estimate.

Move around with `j`/`k` (or the arrow keys), `g`/`G` to jump to top and bottom.
Expand a directory with `l` (or `→`), collapse with `h` (or `←`).

## 2. Find files with search

Press `/` (or `s`) to start searching. The tree filters as you type and, in
search mode, descends into directories regardless of whether they're expanded —
so a query reaches the whole repo, not just what's currently unfolded.

Search has two behaviours, and gnaw picks between them automatically:

{% tabs() %}
=== Substring (default)
A plain query is a **case-insensitive substring** match against file name and
path. Good for quick narrowing.

```text
token        → matches tokenizer.rs, token_map.rs, src/model/token_states.rs
config       → matches config.rs, config_loader.rs, .gnawconfig
```
=== Glob (auto)
As soon as your query contains `*`, `?`, or `{`, gnaw switches to the **full
glob engine** — the same matcher used by `--include` and `--exclude`. Brace
expansion and `**` both work, and matching stays case-insensitive.

```text
*.rs              → every Rust file
src/**/*.rs       → Rust files anywhere under src/
*.{rs,toml}       → Rust and TOML files
test_?.rs         → test_a.rs, test_b.rs … (single-char wildcard)
**/model/*.rs     → files directly inside any model/ directory
```
{% end %}

{% aside(kind="tip", title="You don't toggle anything") %}
There's no mode switch to remember. Type `config` and you're doing substring
matching; add a `*` and the same box is now a glob. The search field tints when
it sees a `*`, so you get a visual cue the instant a query becomes a pattern.
{% end %}

`Ctrl+P` recalls your previous searches — handy when you're cycling between a few
patterns. Press `Enter` to keep the filter applied, or `Esc` to clear it.

## 3. Select

Selection is per file. With the cursor on a file or directory, press `Space` to
toggle it. Selecting a directory selects the files under it.

When a search is active, two bulk keys operate on **exactly what's visible**:

| Key | Action |
| --- | --- |
| `a` | Select every matched file |
| `d` | Deselect every matched file |

A common pattern is *search, then bulk-select*: type `src/**/*.rs`, press `a` to
take all of them, clear the search, then refine by hand. Because the bulk keys
act on matches only, you can layer several searches without clobbering earlier
choices — each `a`/`d` adds to or removes from the running selection rather than
replacing it.

## 4. Trim to a token budget

This is where the TUI earns its keep. Two tools work together, both driven from
the command line (press `:`).

**Sort by weight** to see what's actually costing you tokens:

```text
:sort tokens
```

The tree reorders heaviest-first, and every directory shows the aggregate token
weight of its selected contents — so an expensive subtree is obvious at a glance.
`:sort path` returns to the familiar directory-first ordering, and a bare
`:sort` toggles between the two.

**Filter by size** to hunt down outliers:

```text
:size >2k     show only files over 2,000 tokens
:size <100    show only files under 100 tokens
:size 0       clear the filter
```

`k` and `m` suffixes are accepted. A typical trim: `:sort tokens` to surface the
giants, `:size >2k` to isolate them, then `Space` to drop the ones you don't
need. `:collapse` folds everything back up when you want a clean overview again.

{% aside(kind="note", title="Why a file might vanish under :size") %}
Size filtering compares against each file's *counted* token total. Counting is
debounced and runs in the background, so a freshly selected file may briefly have
no count yet and stay hidden while a `:size` filter is active. It reappears once
its count lands. The [TUI model explanation](/explanations/tui-model/) covers the
counting pipeline.
{% end %}

## 5. Pick a template

Switch to the **Template** tab (`4`). Press `p` to focus the picker and choose a
built-in (for example `claude-xml`, `chatgpt-markdown`, or `gemini`) with
`Enter`. Press `e` to edit the body, or `v` to fill in any template variables —
the variable pane lists what's missing and lets you type values inline. Save a
tweaked template to disk with `s`.

## 6. Generate and copy

Press `Enter` from almost any tab to render the prompt, then jump to **Prompt
output** (`5`) to read it. From there:

| Key | Action |
| --- | --- |
| `c` | Copy the whole prompt to the clipboard |
| `s` | Save it to a timestamped `.md` file |
| `↑` / `↓`, `PageUp` / `PageDown` | Scroll |

Paste it straight into your model of choice. When you're done, `:q` to quit.

## The loop, in one breath

`/` to search → `*.rs` to glob → `a` to take the matches → `:sort tokens` and
`:size >2k` to trim the heavy ones → `4` then `p` to pick a template → `Enter`
→ `5` → `c`. That's a budgeted, hand-tuned prompt in well under a minute.
