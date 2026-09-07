---
name: code2prompt
description: Explore codebases efficiently with code2prompt, using sem-backed entity maps and grouped source reads to reduce agent round trips. Use when understanding an unfamiliar repository, locating implementations, gathering context for changes or reviews, or installing code2prompt.
---

# Code2Prompt

Let code2prompt traverse, filter, parse, and assemble repository context locally.
Use compact structural maps to choose relevant files, then read the implementation
and its tests together. Prefer this over repeated `find`, `rg --files`, and
individual file reads for broad exploration.

Choose the shortest useful workflow:

- If the relevant paths are known, go directly to a grouped source read.
- For an unfamiliar area, request one entity map, then use its paths and symbols
  to select a grouped read. Reuse the map until relevant source files change.
- For a small area, reading its source directly can be cheaper than mapping it.
- If entity extraction is unavailable, use the directory map as a fallback.

Adapt the scope to the task and context budget. Avoid scanning the whole repository
again for each question or dumping all source before identifying the relevant area.

## Install the CLI

Check availability once per environment, not before every read:

```sh
code2prompt --version
code2prompt --help
```

For the sem-backed entity map, use a CLI built with the optional `entity-map`
feature. With Rust/Cargo available, build it from the code2prompt repository:

```sh
cargo install --git https://github.com/mufeedvh/code2prompt \
  --locked --features entity-map code2prompt
```

For ordinary directory maps and source reads, a standard installation is enough:

```sh
# Choose one available package manager:
cargo install code2prompt --locked
brew install code2prompt
```

An existing CLI may list `--entity-map` in its help even when built without the
feature; in that case extraction returns no entities. If building with the feature
is unavailable or unnecessary for the task, continue with ordinary source reads.
There is no separate `sem` executable or service to configure: code2prompt uses
`sem-core` internally.

Prebuilt binaries are available from the
[GitHub releases](https://github.com/mufeedvh/code2prompt/releases).
Installing this skill adds instructions; the CLI is installed separately.
`pip install code2prompt-rs` installs the Python SDK, not the CLI.

## Map an unfamiliar area

Run from the repository you want to inspect. Replace the example paths and
patterns below with its actual layout. Set `CODE2PROMPT_SKILL_DIR` to the absolute
directory containing this installed `SKILL.md`; the bundled template is
[assets/code-map.hbs](assets/code-map.hbs). It is a skill resource, not a file in
the repository being analyzed.

For a source area and its tests:

```sh
code2prompt . --include 'src/**' --include 'tests/**' \
  --exclude '**/*.snap' --entity-map \
  --template "$CODE2PROMPT_SKILL_DIR/assets/code-map.hbs" \
  --output-file - --quiet
```

For an unknown layout, omit the includes and exclude generated/vendor paths, or
get a directory map first. The template lists selected file paths and extracted
entities: names, kinds, declaration snippets, parent names, and line ranges. It
does not render full file contents; `--entity-map` with the default template does.
Declaration snippets are the first line of an entity and can contain inline code;
exclude generated/minified files to keep the map useful.
Do not combine entity extraction with `--exclude '**'` or `--deselected`: files
must be selected for code2prompt to parse them.

`(no extracted entities)` means extraction returned nothing or the binary lacks
the feature. Some files yield generic `chunk` entries instead of declarations;
the template shows their ranges without content snippets. Neither case means the
file is irrelevant. Read manifests, documentation, or other such files directly
as needed. This map is not a dependency graph or semantic text search.

Fallback directory map, without file contents:

```sh
code2prompt . --full-directory-tree --exclude '**' --output-file - --quiet
```

The full tree keeps excluded paths visible while respecting ignore rules and
skipping hidden entries by default. To reduce this tree, analyze a subdirectory;
include/exclude patterns do not prune a full directory tree.

## Read the selected context together

Load the relevant implementation and tests together with line numbers:

```sh
code2prompt . --include 'src/auth/**' --include 'tests/auth/**' \
  --exclude '**/*.snap' --line-numbers --output-file - --quiet
```

Read a small set of specific files when their paths are known:

```sh
code2prompt . --include 'README.md' --include 'src/auth/session.ts' \
  --line-numbers --output-file - --quiet
```

Use the map to collect the likely implementation, its callers or interfaces, and
related tests in one request. Add another batch only when the returned source
reveals a missing dependency or leaves a concrete question unresolved. Narrow the
selection if output is too large or truncated.

For an exact content match or a small line range within a very large file, use an
appropriate text search or file-reading tool. code2prompt selects whole files by
path; it does not offer a symbol-query or line-range CLI selector.

## Useful controls

- Quote glob patterns so the shell does not expand them. Patterns are relative to
  the analyzed directory; repeat `--include` and `--exclude` for multiple patterns.
  Use `**/*.rs` for a language or `src/**` for a subtree.
- Use `--output-file -` to explicitly send the prompt to stdout, including when
  configuration defaults to clipboard output. `--quiet` suppresses progress and
  summary messages. Avoid `--tui` in automated agent workflows.
- For structured output, add `--output-format json`. For a larger result, use
  `--output-file` with a temporary file outside the analyzed directory, then read
  that file. Omit `--quiet` to see the estimated token count on stderr.
  The current estimate includes selected file contents even for a template that
  omits their bodies; do not interpret it as the compact map's actual token size.
- Hidden files require `--hidden`. Ignored files require `--no-ignore`; keep the
  default ignore behavior for ordinary exploration and target any extra files
  explicitly with include patterns.
- If output differs from the requested scope, rerun without `--quiet` to see which
  configuration was loaded. The CLI reads `.c2pconfig` from the working directory,
  then falls back to the platform's user configuration directory. Supplying any
  CLI include/exclude patterns replaces both configured pattern lists; a configured
  `path` can override the positional directory.

See the [CLI guide](https://code2prompt.dev/docs/how_to/cli/) for more options.
