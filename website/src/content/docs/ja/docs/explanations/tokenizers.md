---
title: Code2Promptにおけるトークン化
description: Code2PromptがLLM用にテキストを処理する方法について、トークン化の概要を学びます。
---

言語モデルを扱う場合、テキストをモデルが理解できる形式—**トークン**（数列）に変換する必要があります。この変換は、**トークナイザー**によって処理されます。

---

## トークナイザーとは？

トークナイザーは、生のテキストをトークンに変換します。これは、言語モデルが入力を処理するための基本的な構成要素です。これらのトークンは、トークナイザーの設計に応じて、単語、サブワード、または個々の文字を表すことができます。

`code2prompt`では、**tiktoken**トークナイザーを使用します。これは、効率的で堅牢であり、OpenAIモデルに最適化されています。
その機能は、公式リポジトリで確認できます。

👉 [tiktoken GitHub リポジトリ](https://github.com/openai/tiktoken)

トークナイザー全般について詳しく知りたい場合は、以下を参照してください。

👉 [Mistral トークン化ガイド](https://docs.mistral.ai/guides/tokenization/).

## `code2prompt`での実装

トークン化は、[`tiktoken-rs`](https://github.com/zurawiki/tiktoken-rs)を使用して実装されます。`tiktoken`は、OpenAIモデルで使用される以下のエンコーディングをサポートしています。

| CLI引数 | エンコーディング名 | OpenAIモデル |
| --- | --- | --- |
| `cl100k` | `cl100k_base` | ChatGPTモデル、`text-embedding-ada-002` |
| `p50k` | `p50k_base` | コードモデル、`text-davinci-002`、`text-davinci-003` |
| `p50k_edit` | `p50k_edit` | `text-davinci-edit-001`、`code-davinci-edit-001`などの編集モデル |
| `r50k` | `r50k_base`（または`gpt2`） | `davinci`などのGPT-3モデル |
| `gpt2` | `o200k_base` | GPT-4oモデル |

トークナイザーの詳細については、[OpenAI Cookbook](https://github.com/openai/openai-cookbook/blob/66b988407d8d13cad5060a881dc8c892141f2d5c/examples/How_to_count_tokens_with_tiktoken.ipynb)を参照してください。

> このページは便宜上、自動的に翻訳されています。元のコンテンツについては英語版を参照してください。
