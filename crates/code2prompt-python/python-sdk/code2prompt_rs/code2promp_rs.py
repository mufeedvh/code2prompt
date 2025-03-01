# Import the Rust module
from code2prompt_rs import CodePrompt as RustCodePrompt

class Code2Prompt:
    def __init__(self, path, include_patterns=None, exclude_patterns=None, 
                 include_priority=False, line_numbers=False, relative_paths=False,
                 exclude_from_tree=False, no_codeblock=False, follow_symlinks=False):
        self._inner = RustCodePrompt(
            path,
            include_patterns or [],
            exclude_patterns or [],
            include_priority,
            line_numbers,
            relative_paths,
            exclude_from_tree,
            no_codeblock,
            follow_symlinks
        )

    def generate(self, template=None, encoding=None):
        return self._inner.generate(template, encoding)

    def get_git_diff(self):
        return self._inner.get_git_diff()

    def get_git_diff_between_branches(self, branch1, branch2):
        return self._inner.get_git_diff_between_branches(branch1, branch2)

    def get_git_log(self, branch1, branch2):
        return self._inner.get_git_log(branch1, branch2) 