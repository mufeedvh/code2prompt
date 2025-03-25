---
title: Getting Started with Code2Prompt
description: A step-by-step guide to installing and running Code2Prompt for the first time.
---

# Getting Started

## Using Code2Prompt

Generate a prompt from a codebase directory:

```sh
code2prompt path/to/codebase
```

Use a custom Handlebars template file:

```sh
code2prompt path/to/codebase --template path/to/template.hbs
```

Filter files using glob patterns:

```sh
code2prompt path/to/codebase --include="*.rs,*.toml"
```

Exclude files using glob patterns:

```sh
code2prompt path/to/codebase --exclude="*.txt,*.md"
```

Display the token count of the generated prompt:

```sh
code2prompt path/to/codebase --tokens
```

Specify a tokenizer for token count:

```sh
code2prompt path/to/codebase --tokens --encoding=p50k
```

Supported tokenizers: `cl100k`, `p50k`, `p50k_edit`, `r50k_bas`.
> [!NOTE]  
> See [Tokenizers](#tokenizers) for more details.

Save the generated prompt to an output file:

```sh
code2prompt path/to/codebase --output-file=output.txt
```

Print output as JSON:

```sh
code2prompt path/to/codebase -O json
```

The JSON output will have the following structure:

```json
{
  "prompt": "<Generated Prompt>", 
  "directory_name": "codebase",
  "token_count": 1234,
  "model_info": "ChatGPT models, text-embedding-ada-002",
  "files": []
}
```

Generate a Git commit message (for staged files):

```sh
code2prompt path/to/codebase --diff --template templates/write-git-commit.hbs
```

Generate a Pull Request with branch comparing (for staged files):

```sh
code2prompt path/to/codebase --git-diff-branch 'main, development' --git-log-branch 'main, development' --template templates/write-github-pull-request.hbs
```

Add line numbers to source code blocks:

```sh
code2prompt path/to/codebase --line-number
```

Disable wrapping code inside markdown code blocks:

```sh
code2prompt path/to/codebase --no-codeblock
```

Include hidden files and directories:

```sh
code2prompt path/to/codebase --hidden
```

Skip .gitignore rules:

```sh
code2prompt path/to/codebase --no-ignore
```
