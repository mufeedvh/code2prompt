+++
title = "How the Glob Pattern Filter Works"
description = "How gnaw decides which files to keep or discard using include (-i) and exclude (-e) globs."
weight = 1
+++

gnaw uses glob patterns to include or exclude files and directories, working
similarly to tools like tree or grep. It lets you pass two independent *lists*
of glob patterns:

- **include list** (`--include` or `-i`) — "these patterns allow files"
- **exclude list** (`--exclude` or `-e`) — "these patterns disallow files"

gnaw must decide, for every file in the project, whether it is kept or
discarded. This page explains the rules, and the design choices behind them.

## 1. Sets and Symbols

Throughout the explanation we use the usual set notation:

| Symbol | Meaning |
| --- | --- |
| `A` | set of files that match **at least one** include pattern |
| `B` | set of files that match **at least one** exclude pattern |
| `Ω` | the whole project tree (the *universe*) |
| `C = A ∩ B` | files that match both lists (the *overlap*) |
| `D = Ω \ (A ∪ B)` | files that match neither list |

## 2. Four Situations

| Include list | Exclude list | Files kept |
| --- | --- | --- |
| `A = ∅` | `B = ∅` | `Ω` |
| `A = ∅` | `B ≠ ∅` | `¬B` |
| `A ≠ ∅` | `B = ∅` | `A` |
| `A ≠ ∅` | `B ≠ ∅` | `A \ B` |

1. **No include list, no exclude list** — if no patterns are specified, all files are kept (`Ω`).
2. **Exclude list only** — gnaw acts as a blacklist, removing files that match the excluded patterns (`Ω \ B = ¬B`).
3. **Include list only** — gnaw acts as a whitelist, keeping only files that match the included patterns (`A`).
4. **Include *and* exclude lists** — gnaw keeps files that match the include patterns, but removes those that match the exclude patterns (`A \ B`).

## 3. More on the Overlap

With both lists present (`A ≠ ∅`, `B ≠ ∅`) you have four logical possibilities
for the overlap `C` and the rest `D`:

| Want `C`? | Want `D`? | Reasonable? |
| --- | --- | --- |
| No | No | Default behaviour (`A \ B`) |
| Yes | No | Same behavior as case 3 (`A`) |
| No | Yes | Surprising ("discard what I asked for, keep what I didn't") |
| Yes | Yes | Same behavior as case 1 (`Ω`) |

This is the reason there is no include-priority option: it would produce the
same result as having only an include list (case 3).

## 4. Quick Reference Table

| Want to keep… | Use |
| --- | --- |
| everything | no `-i`, no `-e` |
| everything *except* some patterns | `-e` only |
| *only* what matches the patterns | `-i` only |
| what matches `-i`, minus what matches `-e` | `-i` **and** `-e` |

This design keeps the mental model simple:

- The include list is a whitelist as soon as it exists.
- The exclude list is a blacklist layered on top.
- The overlap is discarded by default.
