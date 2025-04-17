---
title: Understanding Glob Patterns
description: A detailed explanation of glob patterns and how they are used in Code2Prompt.
---

Glob 模式是一种简单而强大的方法，用于使用通配符匹配文件名和路径。它们在命令行界面和编程语言中被广泛使用，以指定文件名或目录的集合。以下是一些最常用的 Glob 模式的详细介绍：

## 基本通配符

- `*`：匹配任意数量的字符，包括零个字符。
  - 示例：`*.txt` 匹配所有以 `.txt` 结尾的文件。

- `?`：匹配恰好一个字符。
  - 示例：`file?.txt` 匹配 `file1.txt`、`fileA.txt`，但不匹配 `file10.txt`。

- `[]`：匹配任意一个括弧内的字符。
  - 示例：`file[1-3].txt` 匹配 `file1.txt`、`file2.txt`、`file3.txt`。

- `[!]` 或 `[^]`：匹配任意一个不在括弧内的字符。
  - 示例：`file[!1-3].txt` 匹配 `file4.txt`、`fileA.txt`，但不匹配 `file1.txt`。

## 高级模式

- `**`：递归匹配任意数量的目录和子目录。
  - 示例：`**/*.txt` 匹配当前目录和所有子目录中的所有 `.txt` 文件。

- `{}`：匹配任意一个用逗号分隔的模式。
  - 示例：`file{1,2,3}.txt` 匹配 `file1.txt`、`file2.txt`、`file3.txt`。

## 示例

1. **匹配目录中的所有文本文件：**

   ```sh
   *.txt
   ```

2. **匹配扩展名前面有一个数字的所有文件：**

   ```sh
   file?.txt
   ```

3. **匹配扩展名为 `.jpg` 或 `.png` 的文件：**

   ```sh
   *.{jpg,png}
   ```

4. **匹配任何子目录中的所有 `.txt` 文件：**

   ```sh
   **/*.txt
   ```

5. **匹配以 `a` 或 `b` 开头并以 `.txt` 结尾的文件：**

   ```sh
   {a,b}*.txt
   ```

## 用例

- **命令行工具：** Glob 模式在 `ls`、`cp`、`mv` 和 `rm` 等命令行工具中被广泛使用，以指定多个文件或目录。
- **编程语言：** Python、JavaScript 和 Ruby 等语言通过 Python 中的 `glob` 库等支持文件匹配的 Glob 模式。
- **构建系统：** Makefile 等工具使用 Glob 模式指定源文件和依赖项。

## 结论

Glob 模式提供了一种灵活直观的方法来匹配文件名和路径，使其对于脚本编写、自动化和文件管理任务来说非常宝贵。理解和利用这些模式可以显著提高您处理文件和目录的效率和生产力。

> 为了您的方便，本页面已自动翻译。请参考英文版本获取原始内容。
