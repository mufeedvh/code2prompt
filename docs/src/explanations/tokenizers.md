# Tokenizers for `code2prompt`

When working with language models, text needs to be transformed into a format that the model can understand—**tokens**, which are sequences of numbers. This transformation is handled by a **tokenizer**.

---

## What is a Tokenizer?

A tokenizer converts raw text into tokens, which are the building blocks for how language models process input. These tokens can represent words, subwords, or even individual characters, depending on the tokenizer's design.

For `code2prompt`, we use the **tiktoken** tokenizer. It’s efficient, robust, and optimized for OpenAI models.
You can explore its functionality in the official repository

👉 [tiktoken GitHub Repository](https://github.com/openai/tiktoken)

If you want to learn more about tokenizer in general, check out the

👉 [Mistral Tokenization Guide](https://docs.mistral.ai/guides/tokenization/).
