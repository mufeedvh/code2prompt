---
title: Tokenization in Code2Prompt
description: Learn about tokenization and how Code2Prompt processes text for LLMs.
---

When working with language models, text needs to be transformed into a format that the model can understand—**tokens**, which are sequences of numbers. This transformation is handled by a **tokenizer**.

---

## What is a Tokenizer?

A tokenizer converts raw text into tokens, which are the building blocks for how language models process input. These tokens can represent words, subwords, or even individual characters, depending on the tokenizer's design.

For `code2prompt`, we use the **tiktoken** tokenizer. It’s efficient, robust, and optimized for OpenAI models.
You can explore its functionality in the official repository

👉 [tiktoken GitHub Repository](https://github.com/openai/tiktoken)

If you want to learn more about tokenizer in general, check out the

👉 [Mistral Tokenization Guide](https://docs.mistral.ai/guides/tokenization/).

## Implementation in `code2prompt`

Tokenization is implemented using [`tiktoken-rs`](https://github.com/zurawiki/tiktoken-rs). `tiktoken` supports these encodings used by OpenAI models:

| CLI Argument | Encoding name           | OpenAI models                                                             |
|----| ----------------------- | ------------------------------------------------------------------------- |
|`cl100k`| `cl100k_base`           | ChatGPT models, `text-embedding-ada-002`                                  |
|`p50k`| `p50k_base`             | Code models, `text-davinci-002`, `text-davinci-003`                       |
|`p50k_edit`| `p50k_edit`             | Use for edit models like `text-davinci-edit-001`, `code-davinci-edit-001` |
|`r50k`| `r50k_base` (or `gpt2`) | GPT-3 models like `davinci`                                               |
|`gpt2`| `o200k_base`            | GPT-4o models                                                             |

For more context on the different tokenizers, see the [OpenAI Cookbook](https://github.com/openai/openai-cookbook/blob/66b988407d8d13cad5060a881dc8c892141f2d5c/examples/How_to_count_tokens_with_tiktoken.ipynb)
