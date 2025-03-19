"""
code2prompt is a Python library for generating LLM prompts from codebases.

It provides a simple interface to the Rust-based code2prompt library, allowing you to:
- Generate prompts from code directories
- Filter files using glob patterns
- Get git diffs and logs
- Count tokens for different models
"""

# Import the Python wrapper class from the renamed file
from .code2prompt import Code2Prompt

__all__ = ['Code2Prompt']