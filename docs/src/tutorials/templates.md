
## Templates

`code2prompt` comes with a set of built-in templates for common use cases. You can find them in the [`templates`](templates) directory.

### [`document-the-code.hbs`](https://github.com/mufeedvh/code2prompt/tree/main/templates/document-the-code.hbs)

Use this template to generate prompts for documenting the code. It will add documentation comments to all public functions, methods, classes and modules in the codebase.

### [`find-security-vulnerabilities.hbs`](https://github.com/mufeedvh/code2prompt/tree/main/templates/find-security-vulnerabilities.hbs)

Use this template to generate prompts for finding potential security vulnerabilities in the codebase. It will look for common security issues and provide recommendations on how to fix or mitigate them.

### [`clean-up-code.hbs`](https://github.com/mufeedvh/code2prompt/tree/main/templates/clean-up-code.hbs)

Use this template to generate prompts for cleaning up and improving the code quality. It will look for opportunities to improve readability, adherence to best practices, efficiency, error handling, and more.

### [`fix-bugs.hbs`](https://github.com/mufeedvh/code2prompt/tree/main/templates/fix-bugs.hbs)

Use this template to generate prompts for fixing bugs in the codebase. It will help diagnose issues, provide fix suggestions, and update the code with proposed fixes.

### [`write-github-pull-request.hbs`](https://github.com/mufeedvh/code2prompt/tree/main/templates/write-github-pull-request.hbs)

Use this template to create GitHub pull request description in markdown by comparing the git diff and git log of two branches.

### [`write-github-readme.hbs`](https://github.com/mufeedvh/code2prompt/tree/main/templates/write-github-readme.hbs)

Use this template to generate a high-quality README file for the project, suitable for hosting on GitHub. It will analyze the codebase to understand its purpose and functionality, and generate the README content in Markdown format.

### [`write-git-commit.hbs`](https://github.com/mufeedvh/code2prompt/tree/main/templates/write-git-commit.hbs)

Use this template to generate git commits from the staged files in your git directory. It will analyze the codebase to understand its purpose and functionality, and generate the git commit message content in Markdown format.

### [`improve-performance.hbs`](https://github.com/mufeedvh/code2prompt/tree/main/templates/improve-performance.hbs)

Use this template to generate prompts for improving the performance of the codebase. It will look for optimization opportunities, provide specific suggestions, and update the code with the changes.

You can use these templates by passing the `-t` flag followed by the path to the template file. For example:

```sh
code2prompt path/to/codebase -t templates/document-the-code.hbs
```

## User Defined Variables

`code2prompt` supports the use of user defined variables in the Handlebars templates. Any variables in the template that are not part of the default context (`absolute_code_path`, `source_tree`, `files`) will be treated as user defined variables.

During prompt generation, `code2prompt` will prompt the user to enter values for these user defined variables. This allows for further customization of the generated prompts based on user input.

For example, if your template includes `{{challenge_name}}` and `{{challenge_description}}`, you will be prompted to enter values for these variables when running `code2prompt`.

This feature enables creating reusable templates that can be adapted to different scenarios based on user provided information.
