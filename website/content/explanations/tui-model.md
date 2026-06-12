+++
title = "How the TUI thinks"
description = "The modal (vim-style) interaction model and the background token-counting and weighting pipeline behind the gnaw file tree."
weight = 10
+++

The TUI keymap tells you *what* each key does. This page explains the two ideas
that make the rest of it make sense: the **modal interaction model** and the
**token-counting pipeline** that feeds weight sorting and size filtering.

## A modal, vim-style surface

gnaw borrows vim's central idea: the same keys mean different things depending on
what you're doing. There are three modes.

| Mode | Purpose |
| --- | --- |
| **Normal** | Navigate and act. Single keys move the cursor, toggle selection, switch tabs. |
| **Insert** | Enter text — a search query, a template variable value, the template body. |
| **Command** | The `:` line, for actions that need an argument or are deliberately deliberate (like quitting). |

The design choice worth knowing: **only the command line is stored as state.**
The current mode isn't a variable the code sets and clears — it's *derived* from
context. If a text field is focused, you're effectively in Insert; if the `:`
line is open, you're in Command; otherwise you're in Normal. Deriving mode from
existing state rather than tracking a separate copy means the two can never drift
out of sync, which is a recurring class of modal-UI bug.

{% aside(kind="note", title="Why Esc stopped quitting") %}
Quitting moved to `:q` precisely because mode is derived. With a modal surface, a
bare `Esc` should mean "back out of whatever I'm in" — leave search, cancel a
variable edit — not "tear down the application." Routing quit through the command
line keeps `Esc` safe to press reflexively. `Ctrl+Q` stays as a hard exit for
when you really mean it.
{% end %}

## The token-counting pipeline

Every number you see in the file tree — per-file counts, directory weights, the
denominator behind selection percentages — comes from one background pipeline.
Understanding it explains several behaviours that otherwise look like quirks.

### Counting is per-file, lazy, and cached

Each selected file moves through a small state machine:

| State | Meaning |
| --- | --- |
| **Pending** | Selected and queued, not yet counted |
| **Counting** | Handed to a background task |
| **Done(n)** | Counted: `n` tokens |
| **Failed** | Binary, empty, or unreadable |

The state map does double duty: it's both the work queue (Pending entries are
what get drained) and the cache (Done entries are kept). Deselecting a file
doesn't immediately evict its count — the entry lingers, so re-selecting the same
file is instant rather than triggering a fresh count. Totals simply ignore
entries for files that aren't currently selected.

### Counting is debounced to quiescence

Counts don't fire on every keystroke or every toggle. After your last selection
change there's a short quiet window (~200 ms); only then does a batch of Pending
files get counted. Rapidly selecting twenty files schedules **one** count batch
once you stop, not twenty cascading re-counts.

This is why a freshly selected file can briefly show no count, and why — if a
`:size` filter is active — it can momentarily disappear: a file with no Done
count can't satisfy a size comparison, so it's hidden until its count lands a
fraction of a second later. It's the debounce settling, not a bug.

### Weights aggregate bottom-up

Directories show the **sum of the Done counts of the selected leaves beneath
them**. This aggregate is recomputed bottom-up whenever counting reaches
quiescence: each file contributes its count (or zero if unselected), each
directory sums its children. The same aggregate is the sort key for
`:sort tokens` — which is why weight sorting only becomes meaningful *after*
counts have landed. Before then, everything weighs zero and the tree falls back
to a stable path order rather than jumping around.

{% aside(kind="tip", title="The practical upshot") %}
Select broadly, wait a beat for counts to settle, then `:sort tokens`. The heavy
subtrees float to the top with honest aggregate weights, and `:size >N` becomes a
precise scalpel. The debounce that makes a file briefly vanish is the same
mechanism that keeps the whole tree from churning while you work.
{% end %}

## Determinism

Counting runs concurrently, but the results are order-independent: the aggregate
for a directory is a sum, and ties in weight sorting fall back to the stable
path ordering. Given the same repository state and the same selection, the tree
settles into the same layout every time — no matter what order the background
counts happened to finish in.
