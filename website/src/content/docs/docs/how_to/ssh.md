---
title: Use Code2prompt CLI with SSH
description: A guide to using Code2Prompt CLI with SSH for remote codebase analysis.
---

## Why it doesn't work?

When you try to run the `code2prompt` CLI on a remote server via SSH, the command is unable to find the clipboard. This is because the `code2prompt` CLI uses the clipboard to copy the generated prompt, and SSH sessions typically do not have access to the local clipboard.

## Solution

To use the `code2prompt` CLI with SSH, you can redirect the output to a file instead of copying it to the clipboard. This way, you can still generate the prompt and save it for later use.

Use the `--output-file` option to specify the output file where the generated prompt will be saved. For example:

```sh
ssh user@remote-server "code2prompt path/to/codebase -O output.txt"
```
