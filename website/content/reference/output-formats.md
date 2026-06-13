+++
title = "Output formats"
description = "The --output-format flag, gnaw's Markdown and XML structures, and the related rendering flags."
weight = 20
+++

gnaw renders the selected files through a template chosen by `--output-format`
(short `-F`). The default is Markdown.

```sh
gnaw . -F xml
gnaw . --output-format markdown -O context.md
```

| Value | Output |
| --- | --- |
| `markdown` *(default)* | Path headers with fenced code blocks |
| `xml` | XML-tagged structure (below) |
| `json` | See the caveat below |

## Markdown

Each file is introduced by its path and wrapped in a fenced code block tagged
with the file's extension, preceded by a directory tree. This is the most
human-readable form and pastes cleanly into most chats.

## XML

The XML form wraps everything in explicit tags, which models like Claude parse
precisely — useful when file contents themselves contain Markdown that could
blur boundaries. gnaw's tags are:

| Tag | Contents |
| --- | --- |
| `<directory>` | The absolute project path |
| `<source-tree>` | The directory tree |
| `<files>` | Wraps all file entries |
| `<file path="…">` | One file; its code in a fenced block unless `--no-codeblock` |
| `<git-diff>` | Present when a diff was requested (see [git context](/reference/git-context/)) |

{% aside(kind="caution", title="`json` currently renders as XML") %}
At present `--output-format json` resolves to the **same template as `xml`** — it
does not emit a structured JSON document. If you need real JSON, treat this as
not-yet-implemented and verify against your build before depending on it. Use
`xml` or `markdown` for now.
{% end %}

## Related rendering flags

| Flag | Effect |
| --- | --- |
| `--no-codeblock` | Omit the fenced code fences around file contents |
| `--token-format raw\|format` | Print token counts as `1234` (`raw`) or `1,234` (`format`) |
| `-t`, `--template <NAME_OR_PATH>` | Use a built-in or custom Handlebars template instead of the format default |
| `-l`, `--line-numbers` | Prefix code lines with line numbers |

Choosing a format selects gnaw's built-in template for that format; `--template`
overrides it entirely with a named built-in (for example `claude-xml`) or a path
to your own.
