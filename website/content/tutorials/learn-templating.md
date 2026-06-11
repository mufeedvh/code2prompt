+++
title = "Learn Templating"
description = "Understand how to use and create custom Handlebars templates for prompt generation."
weight = 2
+++

{% aside(kind="note", title="Tutorial Overview") %}
This tutorial demonstrates how to use and create custom Handlebars templates
for prompt generation in the `gnaw` CLI.
{% end %}

## Prerequisites

Ensure you have `gnaw` installed. If you haven't installed it yet, refer to the
[Installation Guide](/how-to/install/).

## What are Handlebars Templates?

[Handlebars](https://handlebarsjs.com/) is a popular templating engine that
allows you to create dynamic templates using placeholders. In `gnaw`,
Handlebars templates are used to format the generated prompts based on the
codebase structure and user-defined variables.

## How to use Handlebars Templates

You can use these templates by passing the `-t` or `--template` flag followed
by the path to the template file. For example:

{% code(title="sh") %}
```sh
gnaw path/to/codebase -t templates/document-the-code.hbs
```
{% end %}

## Template Syntax

Handlebars templates use a simple syntax for placeholders and expressions. You
will place variables in double curly braces `{{variable_name}}` to include them
in the generated prompt. `gnaw` provides a set of default variables that you
can use in your templates:

- `absolute_code_path`: The absolute path to the codebase.
- `source_tree`: The source tree of the codebase, which includes all files and directories.
- `files`: A list of files in the codebase, including their paths and contents.
- `git_diff`: The git diff of the codebase, if applicable.
- `code`: The code content of the file being processed.
- `path`: The path of the file being processed.

You can also use Handlebars helpers to perform conditional logic, loops, and
other operations within your templates. For example:

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

## Built-in Templates

`gnaw` comes with a set of built-in templates for common use cases. You can
find them in the
[`templates`](https://github.com/gitbadger/gnaw/tree/main/crates/gnaw-core/templates)
directory: `document-the-code`, `find-security-vulnerabilities`,
`clean-up-code`, `fix-bugs`, `write-github-pull-request`,
`write-github-readme`, `write-git-commit`, and `improve-performance`.

## User Defined Variables

`gnaw` supports the use of user defined variables in the Handlebars templates.
Any variables in the template that are not part of the default context
(`absolute_code_path`, `source_tree`, `files`) will be treated as user defined
variables.

During prompt generation, `gnaw` will prompt the user to enter values for these
user defined variables. This allows for further customization of the generated
prompts based on user input.

For example, if your template includes `{{challenge_name}}` and
`{{challenge_description}}`, you will be prompted to enter values for these
variables when running `gnaw`.

{% aside(kind="tip") %}
You can avoid runtime prompting entirely by predefining values in the
`[user_variables]` section of your `.c2pconfig` — see
[Configuration](/tutorials/configuration/).
{% end %}
