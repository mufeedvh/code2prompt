+++
title = "Shell completion"
description = "How gnaw's dynamic completion is installed per shell and what it completes."
weight = 60
+++

gnaw uses dynamic completion: the binary itself produces candidates at request
time, so there's no script file to install or keep in sync. See the
[how-to](/how-to/shell-completion/) for setup; this page is the quick reference.

## Install hooks

Add the matching line to your shell startup file, then start a new shell.

| Shell | Startup file | Line |
| --- | --- | --- |
| fish | `~/.config/fish/config.fish` | `COMPLETE=fish gnaw \| source` |
| zsh | `~/.zshrc` | `source <(COMPLETE=zsh gnaw)` |
| bash | `~/.bashrc` | `source <(COMPLETE=bash gnaw)` |

The hook re-invokes `gnaw` per completion request, so it always matches the
installed binary. Requirements: `gnaw` must be on `PATH`, completion fires for
the bare command name `gnaw`, and you should re-source after upgrading.

## What completes

| Argument | Candidates |
| --- | --- |
| All flags | flag names |
| `--secret-scan` | `off`, `warn`, `redact`, `block` |
| `--compress` | `light`, `moderate`, `full` |
| `--output-format`, `-F` | `markdown`, `json`, `xml` |
| `--encoding` | `cl100k`, `p50k`, `p50k_edit`, `r50k`, `o200k` |
| `--token-format` | `raw`, `format` |
| `--sort` | `name_asc`, `name_desc`, `date_asc`, `date_desc` |
| `--diff-mode` | `staged`, `unstaged`, `all` |
| `-t`, `--template` | built-in template names (with descriptions) + file paths |
| `--compress-strip` | `tests`, `fn-bodies`, `doc-comments`, `private-bodies` and `no-` forms |
| path arg, `-O` | native file/path completion |

## Troubleshooting

| Symptom | Cause |
| --- | --- |
| Tab does nothing | Hook not sourced in this shell, or `gnaw` not on `PATH` |
| Files complete instead of values | No completer registered for `gnaw` — re-source the hook |
| Worked before an upgrade, now broken | Protocol changed between releases — re-source in a fresh shell |

In fish, `complete -c gnaw` lists the registered completer; an empty result
means the hook didn't load.
