---
title: Understanding Glob Patterns
description: A detailed explanation of glob patterns and how they are used in Code2Prompt.
---

# Glob Patterns

Glob patterns are a simple yet powerful way to match file names and paths using wildcard characters. They are commonly used in command-line interfaces and programming languages to specify sets of filenames or directories. Here's a breakdown of the most commonly used glob patterns:

## Basic Wildcards

- `*`: Matches any number of characters, including zero characters.
  - Example: `*.txt` matches all files ending with `.txt`.

- `?`: Matches exactly one character.
  - Example: `file?.txt` matches `file1.txt`, `fileA.txt`, but not `file10.txt`.

- `[]`: Matches any one of the enclosed characters.
  - Example: `file[1-3].txt` matches `file1.txt`, `file2.txt`, `file3.txt`.

- `[!]` or `[^]`: Matches any character not enclosed.
  - Example: `file[!1-3].txt` matches `file4.txt`, `fileA.txt`, but not `file1.txt`.

## Advanced Patterns

- `**`: Matches any number of directories and subdirectories recursively.
  - Example: `**/*.txt` matches all `.txt` files in the current directory and all subdirectories.

- `{}`: Matches any of the comma-separated patterns enclosed.
  - Example: `file{1,2,3}.txt` matches `file1.txt`, `file2.txt`, `file3.txt`.

## Examples

1. **Matching all text files in a directory:**

   ```sh
   *.txt
   ```

2. **Matching all files with a single digit before the extension:**

   ```sh
   file?.txt
   ```

3. **Matching files with extensions `.jpg` or `.png`:**

   ```sh
   *.{jpg,png}
   ```

4. **Matching all `.txt` files in any subdirectory:**

   ```sh
   **/*.txt
   ```

5. **Matching files that start with `a` or `b` and end with `.txt`:**

   ```sh
   {a,b}*.txt
   ```

## Use Cases

- **Command-Line Tools:** Glob patterns are extensively used in command-line tools like `ls`, `cp`, `mv`, and `rm` to specify multiple files or directories.
- **Programming Languages:** Languages like Python, JavaScript, and Ruby support glob patterns for file matching through libraries like `glob` in Python.
- **Build Systems:** Tools like Makefile use glob patterns to specify source files and dependencies.

## Conclusion

Glob patterns provide a flexible and intuitive way to match filenames and paths, making them invaluable for scripting, automation, and file management tasks. Understanding and utilizing these patterns can significantly enhance your productivity and efficiency in handling files and directories.
