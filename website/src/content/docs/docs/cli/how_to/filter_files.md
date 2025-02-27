---
title: Filtering Files in Code2Prompt
description: A step-by-step guide to including or excluding files using different filtering methods.
---


## Usage

Generate a prompt from a codebase directory:

```sh
code2prompt path/to/codebase
```

Use a custom Handlebars template file:

```sh
code2prompt path/to/codebase -t path/to/template.hbs
```

Filter files using glob patterns:

```sh
code2prompt path/to/codebase --include="*.rs,*.toml"
```

Exclude files using glob patterns:

```sh
code2prompt path/to/codebase --exclude="*.txt,*.md"
```

Exclude files/folders from the source tree based on exclude patterns:

```sh
code2prompt path/to/codebase --exclude="*.npy,*.wav" --exclude-from-tree
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
code2prompt path/to/codebase --output=output.txt
```

Print output as JSON:

```sh
code2prompt path/to/codebase --json
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
code2prompt path/to/codebase --diff -t templates/write-git-commit.hbs
```

Generate a Pull Request with branch comparing (for staged files):

```sh
code2prompt path/to/codebase --git-diff-branch 'main, development' --git-log-branch 'main, development' -t templates/write-github-pull-request.hbs
```

Add line numbers to source code blocks:

```sh
code2prompt path/to/codebase --line-number
```

Disable wrapping code inside markdown code blocks:

```sh
code2prompt path/to/codebase --no-codeblock
```

- Rewrite the code to another language.
- Find bugs/security vulnerabilities.
- Document the code.
- Implement new features.

> I initially wrote this for personal use to utilize Claude 3.0's 200K context window and it has proven to be pretty useful so I decided to open-source it!
