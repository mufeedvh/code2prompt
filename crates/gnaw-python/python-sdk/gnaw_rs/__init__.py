"""
gnaw is a Python library for generating LLM prompts from codebases.

It provides a simple interface to the Rust-based gnaw library, allowing you to:
- Generate prompts from code directories
- Filter files using glob patterns
- Get git diffs and logs
- Count tokens for different models
"""

# Import the Python wrapper class from the renamed file
from .gnaw import Gnaw

__all__ = ['Gnaw']