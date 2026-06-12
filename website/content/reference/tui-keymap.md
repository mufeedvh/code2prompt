+++
title = "TUI keymap"
description = "Every key binding in the gnaw TUI: modes, global shortcuts, the command line, and per-tab keys."
weight = 10
+++

The gnaw TUI is modal and vim-flavoured. This page lists every binding,
grouped by the mode or tab it applies in. Launch the interface with:

```sh
gnaw --tui            # current directory
gnaw ./my-repo --tui  # a specific path
```

{% aside(kind="caution", title="Esc no longer quits") %}
In **Normal** mode `Esc` is a no-op. Quitting moved to the command line: `:q`,
`:q!`, or `:quit`. `Ctrl+Q` remains as a hard escape hatch from anywhere.
{% end %}

## Modes

gnaw has three top-level interaction modes. Only the command line is stored;
the rest is derived from context, so you rarely think about modes explicitly —
but they explain which keys are live at any moment.

| Mode | You're in it when… | What keys do |
| --- | --- | --- |
| **Normal** | Browsing a tab, nothing being typed | Navigation and single-key actions |
| **Insert** | A text field is focused — file-tree search, a template variable value, or the template editor | Characters go into the field; `Esc` leaves it |
| **Command** | The `:` command line is open | Characters build a command; `Enter` runs it, `Esc` cancels |

## Global (Normal mode, any tab)

| Key | Action |
| --- | --- |
| `1` … `5` | Jump straight to a tab (see below) |
| `Tab` | Next tab |
| `Shift+Tab` | Previous tab |
| `:` | Open the command line |
| `Ctrl+Q` | Quit immediately |
| `Esc` | Nothing (does **not** quit) |

The five tabs, in order: `1` File tree · `2` Settings · `3` Statistics ·
`4` Template · `5` Prompt output.

## Command line

Open with `:`. While it's open:

| Key | Action |
| --- | --- |
| *(type)* | Build the command |
| `Enter` | Execute |
| `Backspace` | Delete a character |
| `Esc` | Cancel and close |

### Commands

| Command | Effect |
| --- | --- |
| `:q` · `:q!` · `:quit` | Quit |
| `:sort` | Toggle tree ordering (path ↔ token weight) |
| `:sort tokens` · `:sort weight` | Order the tree by token weight, heaviest first |
| `:sort path` · `:sort name` | Order the tree by path (directories first, then alphabetical) |
| `:collapse` | Collapse every folder |
| `:size >N` | Show only files **larger** than N tokens |
| `:size <N` | Show only files **smaller** than N tokens |
| `:size 0` · `:size` | Clear the size filter |

`N` accepts `k` and `m` suffixes — `:size >500`, `:size <2k`, `:size <1m`.
Size filtering compares against each file's counted token total, so files that
haven't been counted yet are hidden while a filter is active.

## File tree — `1`

The default tab. Browse the repository, select what goes into the prompt.

### Browsing

| Key | Action |
| --- | --- |
| `↑` / `k` · `↓` / `j` | Move the cursor |
| `PageUp` / `PageDown` | Move by 10 |
| `Home` / `g` · `End` / `G` | Jump to top / bottom |
| `Space` | Toggle selection of the file or directory at the cursor |
| `→` / `l` | Expand a directory |
| `←` / `h` | Collapse a directory |
| `/` or `s` | Enter search |
| `a` | Select every currently matched file |
| `d` | Deselect every currently matched file |
| `r` | Refresh the tree from disk |
| `Enter` | Generate the prompt from the current selection |

### Search

| Key | Action |
| --- | --- |
| *(type)* | Edit the query — the tree filters live |
| `Backspace` | Delete a character |
| `Ctrl+P` | Recall the previous search from history |
| `Enter` | Apply the filter and leave search |
| `Esc` | Leave search |

{% aside(kind="tip", title="Substring vs glob") %}
A plain query is a **case-insensitive substring** match. The moment your query
contains a glob metacharacter — `*`, `?`, or `{` — search switches to the full
glob engine (the same one behind `--include`/`--exclude`), with brace expansion
and `**`, matched against both file name and path. The search field tints when a
`*` is present, as a cue that you're in glob mode. See the
[interactive selection guide](/how-to/interactive-selection/) for examples.
{% end %}

## Settings — `2`

| Key | Action |
| --- | --- |
| `↑` · `↓` | Move the cursor |
| `Space` | Toggle the highlighted setting |
| `←` · `→` | Cycle the highlighted setting's value |
| `Enter` | Generate the prompt |

## Statistics — `3`

Three views, cycled in place.

| Key | Action |
| --- | --- |
| `←` · `→` | Cycle view: Overview → Token map → Extensions |
| `↑` · `↓` | Scroll |
| `PageUp` · `PageDown` | Scroll by 5 |
| `Home` · `End` | Top / bottom |
| `Enter` | Generate the prompt |

## Template — `4`

A three-pane tab: a picker (choose a built-in or saved template), an editor,
and a variable list. From Normal focus:

| Key | Action |
| --- | --- |
| `e` | Focus the editor and start editing |
| `v` | Focus the variable list and start editing |
| `p` | Focus the template picker |
| `s` | Save the current template to a timestamped file |
| `r` | Reload the default template |
| `Enter` | Generate the prompt |

### Picker focus

| Key | Action |
| --- | --- |
| `↑` · `↓` | Move the selection |
| `Enter` / `l` / `Space` | Load the highlighted template |
| `r` | Refresh the template list |

### Variable focus

| Key | Action |
| --- | --- |
| `↑` · `↓` | Move between variables |
| `Enter` / `Space` | Start editing the highlighted (missing) variable |
| *(type)* / `Backspace` | Edit the value |
| `Enter` | Commit the value |
| `Esc` | Cancel the edit / leave editing |

### Editor focus

Plain text entry into the template body. `Esc` leaves the editor.

## Prompt output — `5`

The rendered prompt.

| Key | Action |
| --- | --- |
| `↑` · `↓` | Scroll |
| `PageUp` · `PageDown` | Scroll by 10 |
| `Home` · `End` | Top / bottom |
| `c` | Copy the output to the clipboard |
| `s` | Save the output to a timestamped `.md` file |
| `Enter` | Re-generate |
