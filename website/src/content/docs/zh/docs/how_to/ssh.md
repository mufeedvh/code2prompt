---
title: 在 SSH 环境下使用 Code2prompt CLI
description: 使用 Code2Prompt CLI 与 SSH 进行远程代码库分析的指南。
---

## 为什么无法工作？

当您尝试通过 SSH 在远程服务器上运行 `code2prompt` CLI 时，命令无法找到剪贴板。这是因为 `code2prompt` CLI 使用剪贴板复制生成的提示，而 SSH 会话通常无法访问本地剪贴板。

## 解决方案

要在 SSH 环境下使用 `code2prompt` CLI，您可以将输出重定向到文件，而不是复制到剪贴板。这样，您仍然可以生成提示并将其保存以备后用。

使用 `--output-file` 选项指定输出文件，其中将保存生成的提示。例如：

```sh
ssh user@remote-server "code2prompt path/to/codebase -O output.txt"
```

> 为了您的方便，本页面已自动翻译。请参考英文版本获取原始内容。
