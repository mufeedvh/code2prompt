"""Tests for prompt generation."""
import pytest
from code2prompt_rs import Code2Prompt

def test_generate_basic(test_dir):
    """Test basic prompt generation."""
    prompt = Code2Prompt(path=test_dir)
    result = prompt.generate()
    
    # Basic checks
    assert result.prompt is not None
    assert isinstance(result.prompt, str)
    assert result.token_count >= 0
    assert result.directory == test_dir

def test_generate_with_include_patterns(test_dir):
    """Test generation with include patterns."""
    prompt = Code2Prompt(
        path=test_dir,
        include_patterns=["*.py"]
    )
    result = prompt.generate()
    
    # Check that Python files are included
    assert "foo.py" in result.prompt
    assert "bar.py" in result.prompt
    
    # Check that text files are excluded
    assert "qux.txt" not in result.prompt
    assert "corge.txt" not in result.prompt

def test_generate_with_exclude_patterns(test_dir):
    """Test generation with exclude patterns."""
    prompt = Code2Prompt(
        path=test_dir,
        exclude_patterns=["**/uppercase/*"]
    )
    result = prompt.generate()
    
    # Check that uppercase directory files are excluded
    assert "FOO.py" not in result.prompt
    assert "BAR.py" not in result.prompt
    
    # Check that lowercase directory files are included
    assert "foo.py" in result.prompt or "lowercase/foo.py" in result.prompt

def test_generate_with_line_numbers(test_dir):
    """Test generation with line numbers."""
    prompt = Code2Prompt(
        path=test_dir,
        include_patterns=["lowercase/foo.py"],
        line_numbers=True
    )
    result = prompt.generate()
    
    # Check for line numbers in output (either format 1: or 1.|)
    assert "1:" in result.prompt or "1|" in result.prompt

def test_generate_with_relative_paths(test_dir):
    """Test generation with relative paths."""
    prompt = Code2Prompt(
        path=test_dir,
        include_patterns=["lowercase/foo.py"],
        relative_paths=True
    )
    result = prompt.generate()
    
    # Should include relative path format
    assert "lowercase/foo.py" in result.prompt
    
    # Should not include absolute path
    assert test_dir not in result.prompt

def test_generate_with_custom_template(test_dir):
    """Test generation with custom template."""
    template = """# Code Overview
{% for file in files %}
## {{ file.path }}
```{{ file.language }}
{{ file.content }}" \
"{% endfor %}"""

prompt = Code2Prompt(
    path=test_dir,
    include_patterns=["lowercase/foo.py"]
)
result = prompt.generate(template=template)

# Check that custom template was used
assert "# Code Overview" in result.prompt
assert "## " in result.prompt


def test_token_count(test_dir):
    """Test token counting."""
    prompt = Code2Prompt(path=test_dir)

    # Get token count directly
    token_count = prompt.token_count(encoding="cl100k")
    assert isinstance(token_count, int)
    assert token_count > 0

    # Compare with generated result
    result = prompt.generate(encoding="cl100k")
    assert result.token_count == token_count

def test_multiple_encoding_options(test_dir):
    """Test with different encoding options."""
    prompt = Code2Prompt(
    path=test_dir,
    include_patterns=["lowercase/foo.py"]
    )

# Try different encodings
encodings = ["cl100k", "gpt2", "p50k_base"]
token_counts = {}

for encoding in encodings:
    try:
        count = prompt.token_count(encoding=encoding)
        token_counts[encoding] = count
    except Exception as e:
        # Some encodings might not be available, that's OK
        print(f"Encoding {encoding} failed: {e}")

    # At least one encoding should work
    assert len(token_counts) > 0

    # Different encodings might give different counts
    # (but for very small files they might be the same)
    if len(token_counts) > 1:
        unique_counts = set(token_counts.values())
        print(f"Token counts: {token_counts}")