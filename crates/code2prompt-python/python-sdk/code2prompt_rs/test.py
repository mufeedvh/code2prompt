#!/usr/bin/env python3
"""
Basic example script for code2prompt_rs

This script shows how to use the Rust-powered code2prompt library
to generate prompts from a code repository.
"""

import sys
import os
from pathlib import Path
from code2prompt_rs import PyCode2PromptSession

def main():
    # Get the target directory from command line or use current directory
    target_dir = sys.argv[1] if len(sys.argv) > 1 else "."
    target_path = str(Path(target_dir).absolute())
    
    print(f"Analyzing code in: {target_path}")
    print("-" * 80)
    
    try:
        # Initialize the session
        session = PyCode2PromptSession(target_path)
        
        # Configure the session with a fluent interface
        session = (session.include(["*.py", "*.rs", "*.js"])
                         .exclude(["**/node_modules/**", "**/target/**", "**/__pycache__/**"])
                         .with_line_numbers(True)
                         .with_relative_paths(True)
                         .with_code_blocks(True)
                         .sort_by("name_asc")
                         .with_token_encoding("cl100k"))
        
        # Print some information about the session
        info = session.info()
        print("Session configuration:")
        for key, value in info.items():
            print(f"- {key}: {value}")
        
        # Generate the prompt
        print("\nGenerating prompt...")
        prompt = session.generate()
        
        # Get token count
        token_count = session.token_count()
        
        # Print results
        print(f"\nToken count: {token_count}")
        print("\nPrompt preview (first 500 chars):")
        print("-" * 80)
        print(prompt[:500] + "..." if len(prompt) > 500 else prompt)
        print("-" * 80)
        
        # Save to file
        output_file = "code_prompt.md"
        with open(output_file, "w") as f:
            f.write(prompt)
        print(f"\nFull prompt saved to: {output_file}")
        
    except Exception as e:
        print(f"Error: {e}")
        return 1
    
    return 0

if __name__ == "__main__":
    sys.exit(main())