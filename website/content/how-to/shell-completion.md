+++
title = "Enable shell completion"
description = "Turn on tab-completion for gnaw's flags, enum values, template names, and paths in bash, zsh, or fish."
weight = 50
+++

gnaw ships dynamic shell completion: tab-complete flags, their enum values
(`--secret-scan <TAB>` → `off warn redact block`), built-in template names,
and file paths. There's no static script to install or regenerate — the binary
emits the completion hook itself when your shell asks for it.

## Turn it on

Source the hook from your shell's startup file so every new shell picks it up.

{% tabs() %}
=== fish
```fish
# ~/.config/fish/config.fish
if command -q gnaw
    COMPLETE=fish gnaw | source
end
```
=== zsh
```zsh
# ~/.zshrc
source <(COMPLETE=zsh gnaw)
```
=== bash
```bash
# ~/.bashrc
source <(COMPLETE=bash gnaw)
```
{% end %}

Reload your shell (open a new terminal, or `exec fish` / `exec zsh` / `exec bash`)
and try it:

```sh
gnaw --secret-scan <TAB>     # off  warn  redact  block
gnaw --compress <TAB>        # light  moderate  full
gnaw -t <TAB>                # built-in template names, with descriptions
gnaw --encoding <TAB>        # cl100k  p50k  p50k_edit  r50k  o200k
```

## How it works (and why one detail matters)

The hook re-invokes `gnaw` with a `COMPLETE` environment variable each time your
shell needs candidates, so completion always reflects the installed binary — no
stale generated script. Two consequences:

- `gnaw` must be on your `PATH`. If you installed with `cargo install --path crates/gnaw-ctx`,
  it lives in `~/.cargo/bin`, which is normally already on `PATH`. Completion
  fires for the bare command `gnaw`, not for `./target/debug/gnaw` or other paths.
- Re-source after upgrading gnaw. The shell↔binary completion protocol can change
  between releases, so refresh the hook (open a new shell) when you update.

{% aside(kind="tip", title="Check it's registered") %}
If tab does nothing, confirm a completer is registered in the current shell —
in fish, `complete -c gnaw` should print a line. If it's empty, the hook didn't
source (wrong startup file, or `gnaw` not on `PATH`); re-run the source command
in the live shell and try again.
{% end %}

## Dynamic candidates

Beyond flags and enums, gnaw completes two things at runtime:

- **`--template`** offers every built-in template name (`claude-xml`,
  `chatgpt-markdown`, `gemini`, the task templates, …) with each one's
  description shown alongside, and still falls back to file-path completion for
  custom templates.
- **`--compress-strip`** completes the strip tokens (`tests`, `fn-bodies`,
  `doc-comments`, `private-bodies`) and their `no-` negations.

Path arguments (the directory to analyze, `-O`, `-t`) use your shell's native
file completion.
