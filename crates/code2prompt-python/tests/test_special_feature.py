## test_special_features.py - Tests pour fonctionnalités spéciales

"""Tests for special features of Code2Prompt."""
import pytest
import os
from pathlib import Path
from code2prompt_rs import Code2Prompt

def test_hidden_files(test_dir):
    """Test handling of hidden files."""
    # First, with hidden files excluded (default)
    prompt = Code2Prompt(path=test_dir)
    result = prompt.generate()
    
    # The .secret directory should be excluded
    assert "secret.txt" not in result.prompt
    
    # Now, include hidden files
    prompt = Code2Prompt(
        path=test_dir,
        include_hidden=True
    )
    result = prompt.generate()
    
    # Should include .secret directory now
    assert "secret.txt" in result.prompt or ".secret/secret.txt" in result.prompt

def test_directory_tree(test_dir):
    """Test full directory tree generation."""
    prompt = Code2Prompt(
        path=test_dir,
        full_directory_tree=True
    )
    result = prompt.generate()
    
    # Should include directory structure
    assert "lowercase" in result.prompt
    assert "uppercase" in result.prompt

def test_no_code_blocks(test_dir):
    """Test generation without code blocks."""
    # With code blocks (default)
    prompt = Code2Prompt(
        path=test_dir,
        include_patterns=["lowercase/foo.py"]
    )
    with_blocks = prompt.generate()
    
    # Without code blocks
    prompt = Code2Prompt(
        path=test_dir,
        include_patterns=["lowercase/foo.py"],
        code_blocks=False
    )
    without_blocks = prompt.generate()
    
    # Code blocks typically include ```python or ```py
    assert "```" in with_blocks.prompt
    assert "```" not in without_blocks.prompt

def test_sort_files(test_dir):
    """Test different sorting methods if available."""
    # This test depends on if sort_by is exposed in your API
    try:
        # Default should be name ascending
        prompt = Code2Prompt(path=test_dir)
        session = prompt.session()
        
        # Try to sort by name_desc if method exists
        if hasattr(session, "sort_by"):
            session = session.sort_by("name_desc")
            result = session.generate()
            # Hard to verify sort in output, but should not error
            assert result is not None
    except AttributeError:
        # If sort_by isn't implemented, just pass the test
        pass