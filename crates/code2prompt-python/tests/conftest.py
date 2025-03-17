"""Pytest fixtures for code2prompt tests."""
import os
import pytest
import tempfile
import shutil
from pathlib import Path

@pytest.fixture(scope="session")
def test_hierarchy():
    """Create a test hierarchy of files and directories."""
    # Create a temporary directory
    temp_dir = tempfile.mkdtemp()
    
    try:
        # Create directories
        lowercase_dir = Path(temp_dir) / "lowercase"
        uppercase_dir = Path(temp_dir) / "uppercase"
        secret_dir = Path(temp_dir) / ".secret"
        
        for dir_path in [lowercase_dir, uppercase_dir, secret_dir]:
            dir_path.mkdir(parents=True, exist_ok=True)
        
        # Create files
        files = [
            ("lowercase/foo.py", "def foo():\n    return 'foo'\n"),
            ("lowercase/bar.py", "def bar():\n    return 'bar'\n"),
            ("lowercase/baz.py", "def baz():\n    return 'baz'\n"),
            ("lowercase/qux.txt", "content qux.txt"),
            ("lowercase/corge.txt", "content corge.txt"),
            ("lowercase/grault.txt", "content grault.txt"),
            ("uppercase/FOO.py", "def FOO():\n    return 'FOO'\n"),
            ("uppercase/BAR.py", "def BAR():\n    return 'BAR'\n"),
            ("uppercase/BAZ.py", "def BAZ():\n    return 'BAZ'\n"),
            ("uppercase/QUX.txt", "CONTENT QUX.TXT"),
            ("uppercase/CORGE.txt", "CONTENT CORGE.TXT"),
            ("uppercase/GRAULT.txt", "CONTENT GRAULT.TXT"),
            (".secret/secret.txt", "SECRET"),
        ]
        
        for file_path, content in files:
            full_path = Path(temp_dir) / file_path
            full_path.write_text(content)
            
        # Create a gitignore file
        gitignore_path = Path(temp_dir) / ".gitignore"
        gitignore_path.write_text("*.txt\n")
            
        # Return the path
        yield temp_dir
    finally:
        # Clean up
        shutil.rmtree(temp_dir)

@pytest.fixture
def test_dir(test_hierarchy):
    """Return the path to the test hierarchy."""
    return test_hierarchy