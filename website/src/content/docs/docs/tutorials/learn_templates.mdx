---
title: Learn Handlebar Templates with Code2Prompt
description: Understand how to use and create custom Handlebars templates for prompt generation.
---

import { Card } from "@astrojs/starlight/components";

<Card title="Tutorial Overview">
  This tutorial demonstrates how to use and create custom Handlebars templates
  for prompt generation in `code2prompt` CLI.
</Card>

---

## Prerequisites

Ensure you have `code2prompt` installed. If you haven't installed it yet, refer to the [Installation Guide](/docs/how_to/install).

---

## What are Handlebars Templates ?

[Handlebars](https://handlebarsjs.com/) is a popular templating engine that allows you to create dynamic templates using placeholders.
In `code2prompt`, Handlebars templates are used to format the generated prompts based on the codebase structure and user-defined variables.

## How to use Handlebars Templates ?

You can use these templates by passing the `-t` or `--template` flag followed by the path to the template file. For example:

```sh
code2prompt path/to/codebase -t templates/document-the-code.hbs
```

## Template Syntax

Handlebars templates use a simple syntax for placeholders and expressions. You will place variables in double curly braces `{{variable_name}}` to include them in the generated prompt.
`Code2prompt` provides a set of default variables that you can use in your templates:

- `absolute_code_path`: The absolute path to the codebase.
- `source_tree`: The source tree of the codebase, which includes all files and directories.
- `files`: A list of files in the codebase, including their paths and contents.
- `git_diff`: The git diff of the codebase, if applicable.
- `code`: The code content of the file being processed.
- `path`: The path of the file being processed.

You can also use Handlebars helpers to perform conditional logic, loops, and other operations within your templates. For example:

```handlebars
{{#if files}}
  {{#each files}}
    File:
    {{this.path}}
    Content:
    {{this.content}}
  {{/each}}
{{else}}
  No files found.
{{/if}}
```

---

## Existing Templates

`code2prompt` comes with a set of built-in templates for common use cases. You can find them in the [`templates`](https://github.com/mufeedvh/code2prompt/tree/main/crates/code2prompt-core/templates) directory.

### [`document-the-code.hbs`](https://github.com/mufeedvh/code2prompt/tree/main/crates/code2prompt-core/templates/document-the-code.hbs)

Use this template to generate prompts for documenting the code. It will add documentation comments to all public functions, methods, classes and modules in the codebase.

### [`find-security-vulnerabilities.hbs`](https://github.com/mufeedvh/code2prompt/tree/main/crates/code2prompt-core/templates/find-security-vulnerabilities.hbs)

Use this template to generate prompts for finding potential security vulnerabilities in the codebase. It will look for common security issues and provide recommendations on how to fix or mitigate them.

### [`clean-up-code.hbs`](https://github.com/mufeedvh/code2prompt/tree/main/crates/code2prompt-core/templates/clean-up-code.hbs)

Use this template to generate prompts for cleaning up and improving the code quality. It will look for opportunities to improve readability, adherence to best practices, efficiency, error handling, and more.

### [`fix-bugs.hbs`](https://github.com/mufeedvh/code2prompt/tree/main/crates/code2prompt-core/templates/fix-bugs.hbs)

Use this template to generate prompts for fixing bugs in the codebase. It will help diagnose issues, provide fix suggestions, and update the code with proposed fixes.

### [`write-github-pull-request.hbs`](https://github.com/mufeedvh/code2prompt/tree/main/crates/code2prompt-core/templates/write-github-pull-request.hbs)

Use this template to create GitHub pull request description in markdown by comparing the git diff and git log of two branches.

### [`write-github-readme.hbs`](https://github.com/mufeedvh/code2prompt/tree/main/crates/code2prompt-core/templates/write-github-readme.hbs)

Use this template to generate a high-quality README file for the project, suitable for hosting on GitHub. It will analyze the codebase to understand its purpose and functionality, and generate the README content in Markdown format.

### [`write-git-commit.hbs`](https://github.com/mufeedvh/code2prompt/tree/main/crates/code2prompt-core/templates/write-git-commit.hbs)

Use this template to generate git commits from the staged files in your git directory. It will analyze the codebase to understand its purpose and functionality, and generate the git commit message content in Markdown format.

### [`improve-performance.hbs`](https://github.com/mufeedvh/code2prompt/tree/main/crates/code2prompt-core/templates/improve-performance.hbs)

Use this template to generate prompts for improving the performance of the codebase. It will look for optimization opportunities, provide specific suggestions, and update the code with the changes.

## User Defined Variables

`code2prompt` supports the use of user defined variables in the Handlebars templates. Any variables in the template that are not part of the default context (`absolute_code_path`, `source_tree`, `files`) will be treated as user defined variables.

During prompt generation, `code2prompt` will prompt the user to enter values for these user defined variables. This allows for further customization of the generated prompts based on user input.

For example, if your template includes `{{challenge_name}}` and `{{challenge_description}}`, you will be prompted to enter values for these variables when running `code2prompt`.

This feature enables creating reusable templates that can be adapted to different scenarios based on user provided information.
