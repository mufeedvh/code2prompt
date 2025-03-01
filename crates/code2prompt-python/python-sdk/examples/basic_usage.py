"""Example usage of the code2prompt Python SDK."""

from code2prompt_rs import CodePrompt

def main():
    # Create a CodePrompt instance for the current directory
    prompt = CodePrompt(
        path=".",
        include_patterns=["*.py", "*.rs"],  # Only include Python and Rust files
        exclude_patterns=["**/tests/*"],     # Exclude test files
        line_numbers=True                    # Add line numbers to code
    )

    # Generate a prompt with token counting
    result = prompt.generate(encoding="cl100k")
    
    # Print the results
    print(f"Generated prompt for directory: {result['directory']}")
    print(f"Token count: {result['token_count']}")
    print(f"Model info: {result['model_info']}")
    
    # Print the first 1000 characters of the prompt, or less if shorter
    print("\nPrompt preview:")
    prompt_text = result['prompt']
    if prompt_text:
        preview_length = min(1000, len(prompt_text))
        print(f"{prompt_text[:preview_length]}...")
    else:
        print("No prompt generated")

    # Git operations example
    print("\nGit operations:")
    
    try:
        # Get current changes
        diff = prompt.get_git_diff()
        print("\nCurrent git diff:")
        print(diff[:200] + "..." if diff else "No changes")
        
        # Get diff between branches
        branch_diff = prompt.get_git_diff_between_branches("main", "develop")
        print("\nDiff between main and develop:")
        print(branch_diff[:200] + "..." if branch_diff else "No differences")
        
        # Get git log
        git_log = prompt.get_git_log("main", "develop")
        print("\nGit log between main and develop:")
        print(git_log[:200] + "..." if git_log else "No log entries")
        
    except Exception as e:
        print(f"Git operations failed: {e}")

if __name__ == "__main__":
    main()