---
title: Code2Prompt 中的分词
description: 了解分词以及 Code2Prompt 如何为大型语言模型处理文本。
---

在处理语言模型时，文本需要转换为模型可以理解的格式——**tokens**，即数字序列。这种转换由 **tokenizer** 处理。

---

## 什么是 Tokenizer？

Tokenizer 将原始文本转换为 tokens，这些是语言模型处理输入的基本单位。这些 tokens 可以根据 tokenizer 的设计表示单词、子单词甚至单个字符。

对于 `code2prompt`，我们使用 **tiktoken** tokenizer。它高效、稳健，并针对 OpenAI 模型进行了优化。
您可以在官方仓库中探索其功能

👉 [tiktoken GitHub 仓库](https://github.com/openai/tiktoken)

如果您想了解更多关于 tokenizer 的信息，请查看

👉 [Mistral 分词指南](https://docs.mistral.ai/guides/tokenization/).

## 在 `code2prompt` 中的实现

分词使用 [`tiktoken-rs`](https://github.com/zurawiki/tiktoken-rs) 实现。`tiktoken` 支持 OpenAI 模型使用的以下编码：

| 命令行参数 | 编码名称           | OpenAI 模型                                                             |
| ---- | ----------------------- | ------------------------------------------------------------------------- |
|`cl100k`| `cl100k_base`           | ChatGPT 模型，`text-embedding-ada-002`                                  |
|`p50k`| `p50k_base`             | 代码模型，`text-davinci-002`，`text-davinci-003`                       |
|`p50k_edit`| `p50k_edit`             | 用于编辑模型，如 `text-davinci-edit-001`，`code-davinci-edit-001` |
|`r50k`| `r50k_base`（或 `gpt2`） | GPT-3 模型，如 `davinci`                                               |
|`gpt2`| `o200k_base`            | GPT-4o 模型                                                             |

有关不同 tokenizer 的更多上下文，请参阅 [OpenAI Cookbook](https://github.com/openai/openai-cookbook/blob/66b988407d8d13cad5060a881dc8c892141f2d5c/examples/How_to_count_tokens_with_tiktoken.ipynb)

> 为了您的方便，本页面已自动翻译。请参考英文版本获取原始内容。
