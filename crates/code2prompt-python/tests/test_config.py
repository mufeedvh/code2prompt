"""Tests for Code2Prompt configuration."""
import pytest
from pathlib import Path
from code2prompt_rs import Code2Prompt

def test_basic_initialization(test_dir):
    """Test that Code2Prompt can be initialized with minimal settings."""
    prompt = Code2Prompt(path=test_dir)
    assert prompt is not None
    assert str(prompt.path) == test_dir
    assert prompt.include_patterns == []
    assert prompt.exclude_patterns == []

def test_initialization_with_options(test_dir):
    """Test initialization with various options."""
    prompt = Code2Prompt(
        path=test_dir,
        include_patterns=["*.py"],
        exclude_patterns=["**/uppercase/*"],
        include_priority=True,
        line_numbers=True,
        absolute_paths=True,
        full_directory_tree=True,
        code_blocks=False,
        follow_symlinks=True
    )
    
    assert prompt.include_patterns == ["*.py"]
    assert prompt.exclude_patterns == ["**/uppercase/*"]
    assert prompt.include_priority is True
    assert prompt.line_numbers is True
    assert prompt.absolute_paths is True
    assert prompt.full_directory_tree is True
    assert prompt.code_blocks is False
    assert prompt.follow_symlinks is True

def test_session_creation(test_dir):
    """Test that a session can be created."""
    prompt = Code2Prompt(path=test_dir)
    session = prompt.session()
    assert session is not None
    
    # Verify that the session contains expected info
    info = session.info()
    assert "path" in info
    assert Path(info["path"]) == Path(test_dir)

def test_configuration_chain(test_dir):
    """Test using session for complex configuration."""
    prompt = Code2Prompt(path=test_dir)
    session = prompt.session()
    
    # Apply multiple configurations (using the original session would
    # involve setting up method calls to return 'self')
    session = session.include(["*.py"])
    session = session.exclude(["**/uppercase/*"])
    session = session.with_line_numbers(True)
    
    # Verify configuration was applied
    info = session.info()
    assert info["include_patterns"] != "[]"