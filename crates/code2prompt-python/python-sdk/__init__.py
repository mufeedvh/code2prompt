"""
code2prompt is a Python library for generating LLM prompts from codebases.

It provides a simple interface to the Rust-based code2prompt library, allowing you to:
- Generate prompts from code directories
- Filter files using glob patterns
- Get git diffs and logs
- Count tokens for different models
"""

from .code2prompt import CodePrompt

__version__ = "2.0.0"
__all__ = ["CodePrompt"]