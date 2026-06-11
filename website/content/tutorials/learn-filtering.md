+++
title = "Learn Filtering"
description = "Learn how to exclude or include files in your LLM prompts using powerful filtering options."
weight = 3
+++

{% aside(kind="note", title="Tutorial Overview") %}
This tutorial demonstrates how to use the **glob pattern tool** in the `gnaw`
CLI to filter and manage files based on include and exclude patterns.
{% end %}

Glob patterns work similarly to tools like `tree` or `grep`, providing powerful
filtering capabilities. Check out the
[detailed explanation](/explanations/glob-pattern-filter/) for more information.

## Prerequisites

Ensure you have `gnaw` installed. If you haven't installed it yet, refer to the
[Installation Guide](/how-to/install/).

## Understanding Include and Exclude Patterns

Glob patterns allow you to specify rules for filtering files and directories.

- **Include Patterns** (`--include`): Specify files and directories you want to include.
- **Exclude Patterns** (`--exclude`): Specify files and directories you want to exclude.

When both are given, gnaw keeps what matches the include patterns minus what
matches the exclude patterns. CLI patterns are **merged onto** patterns from
your config file rather than replacing them.

## Setting Up the Environment

To practice with glob patterns, let's create a sample folder structure with
some files.

{% code(title="setup.sh") %}
```bash
#!/bin/bash

# Create base directory
mkdir -p test_dir/{lowercase,uppercase,.secret}

# Create files in the structure
echo "content foo.py" > "test_dir/lowercase/foo.py"
echo "content bar.py" > "test_dir/lowercase/bar.py"
echo "content baz.py" > "test_dir/lowercase/baz.py"
echo "content qux.txt" > "test_dir/lowercase/qux.txt"
echo "content corge.txt" > "test_dir/lowercase/corge.txt"
echo "content grault.txt" > "test_dir/lowercase/grault.txt"

echo "CONTENT FOO.py" > "test_dir/uppercase/FOO.PY"
echo "CONTENT BAR.py" > "test_dir/uppercase/BAR.PY"
echo "CONTENT BAZ.py" > "test_dir/uppercase/BAZ.PY"
echo "CONTENT QUX.txt" > "test_dir/uppercase/QUX.TXT"
echo "CONTENT CORGE.txt" > "test_dir/uppercase/CORGE.TXT"
echo "CONTENT GRAULT.txt" > "test_dir/uppercase/GRAULT.TXT"

echo "top secret" > "test_dir/.secret/secret.txt"
```
{% end %}

To clean up the structure later, run `rm -rf test_dir`.

It will create the following directory structure:

```
test_dir
├── lowercase
│   ├── foo.py
│   ├── bar.py
│   ├── baz.py
│   ├── qux.txt
│   ├── corge.txt
│   └── grault.txt
├── uppercase
│   ├── FOO.PY
│   ├── BAR.PY
│   ├── BAZ.PY
│   ├── QUX.TXT
│   ├── CORGE.TXT
│   └── GRAULT.TXT
└── .secret
    └── secret.txt
```

## General Usage

By default, `gnaw` includes all files in the specified directory, respecting
`.gitignore`.

### Case 1: Default Behavior (No Filters)

```bash
gnaw test_dir
```

All files are included: `lowercase/*`, `uppercase/*`, and `.secret/secret.txt`.

### Case 2: Exclude Specific File Types

Exclude `.txt` files:

```bash
gnaw test_dir --exclude="*.txt"
```

Excluded: all `.txt` files. Included: all `.py` files.

### Case 3: Include Specific File Types

Include only Python files:

```bash
gnaw test_dir --include="*.py"
```

Included: all `.py` files. Excluded: everything else, including
`.secret/secret.txt`.

### Case 4: Combine Include and Exclude

Include `.py` files but exclude files in the `uppercase` folder:

```bash
gnaw test_dir --include="*.py" --exclude="**/uppercase/*"
```

Included: all `lowercase` files with the `.py` extension. Excluded: all
`uppercase` files and `.secret/secret.txt`. The result is always *include minus
exclude* — see the [glob filter explanation](/explanations/glob-pattern-filter/)
for why there is no priority flag.

### Case 5: Exclude a Specific Directory

```bash
gnaw test_dir --exclude="**/uppercase/*"
```

Included: all files in `lowercase` and `.secret`. Excluded: all files in
`uppercase`.

## Summary

The glob pattern tool in `gnaw` allows you to filter files and directories
effectively using `--include` for files to include and `--exclude` for files to
exclude. To practice, set up the sample directory, try out the commands, and
see how the tool filters files dynamically.
