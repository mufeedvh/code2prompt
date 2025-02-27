---
title: How the Glob Pattern Filter Works
description: An in-depth look at how Code2Prompt processes include and exclude patterns.
---

# How the Glob Pattern Filter Works

The tool uses glob patterns to include or exclude files and directories, working similarly to tools like `tree` or `grep`. Here’s a detailed explanation:

## Key Concepts

- **Include List (A)**: A set containing the glob patterns for files and directories you want to include.
- **Exclude List (B)**: A set containing the glob patterns for files and directories you want to exclude.
- **Universe (Ω)**: The set of all files and directories.

When you specify an `--exclude` list and/or an `--include` list, the following logic applies:

### Cases and Logic

1. **No include list, no exclude list**  
   Include everything:  
   $$
   \neg A \cap \neg B \implies \Omega
   $$

2. **No include list, with exclude list**  
   Include everything except what matches the exclude list:  
   $$
   \neg A \cap B \implies \Omega \setminus B
   $$

3. **With include list, no exclude list**  
   Include only what matches the include list:  
   $$
   A \cap \neg B \implies A
   $$

4. **With include list and exclude list**  
   Include what matches the include list and exclude what matches the exclude list. Handle the intersection based on the `include_priority` parameter:

   - **Include priority == true**:  
     $$
     A \setminus (A \cap B)
     $$

    - **Include priority != true***:  
     $$
     B \setminus (A \cap B)
     $$

### Visual Representation of Case 4

Let (A) and (B) overlap. Depending on the priority, the intersection $$( A \cap B )$$ is either included or excluded based on the `include_priority` parameter.

![Visual Representation of Case 4](../../../../../assets/filter.png)
