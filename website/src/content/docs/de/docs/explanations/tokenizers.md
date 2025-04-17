---
title: Tokenisierung in Code2Prompt
description: Erfahren Sie mehr über Tokenisierung und wie Code2Prompt Text für LLMs verarbeitet.
---

Bei der Arbeit mit Sprachmodellen muss Text in ein Format umgewandelt werden, das das Modell verstehen kann – **Tokens**, die Sequenzen von Zahlen sind. Diese Transformation wird von einem **Tokenizer** durchgeführt.

---

## Was ist ein Tokenizer?

Ein Tokenizer konvertiert rohen Text in Tokens, die die Bausteine für die Verarbeitung von Eingaben durch Sprachmodelle sind. Diese Tokens können je nach Design des Tokenizers Wörter, Subwörter oder sogar einzelne Zeichen darstellen.

Für `code2prompt` verwenden wir den **tiktoken**-Tokenizer. Er ist effizient, robust und für OpenAI-Modelle optimiert.
Sie können seine Funktionalität im offiziellen Repository erkunden

👉 [tiktoken GitHub Repository](https://github.com/openai/tiktoken)

Wenn Sie mehr über Tokenizer im Allgemeinen erfahren möchten, lesen Sie den

👉 [Mistral Tokenization Guide](https://docs.mistral.ai/guides/tokenization/).

## Implementierung in `code2prompt`

Die Tokenisierung wird mit [`tiktoken-rs`](https://github.com/zurawiki/tiktoken-rs) implementiert. `tiktoken` unterstützt diese Kodierungen, die von OpenAI-Modellen verwendet werden:

| CLI-Argument | Kodierungsname           | OpenAI-Modelle                                                           |
| ---- | ----------------------- | ------------------------------------------------------------------------- |
| `cl100k` | `cl100k_base`           | ChatGPT-Modelle, `text-embedding-ada-002`                                  |
| `p50k` | `p50k_base`             | Code-Modelle, `text-davinci-002`, `text-davinci-003`                       |
| `p50k_edit` | `p50k_edit`             | Für Edit-Modelle wie `text-davinci-edit-001`, `code-davinci-edit-001` |
| `r50k` | `r50k_base` (oder `gpt2`) | GPT-3-Modelle wie `davinci`                                               |
| `gpt2` | `o200k_base`            | GPT-4o-Modelle                                                             |

Für mehr Kontext zu den verschiedenen Tokenizern siehe das [OpenAI Cookbook](https://github.com/openai/openai-cookbook/blob/66b988407d8d13cad5060a881dc8c892141f2d5c/examples/How_to_count_tokens_with_tiktoken.ipynb)

> Diese Seite wurde für Ihre Bequemlichkeit automatisch übersetzt. Bitte greifen Sie für den Originalinhalt auf die englische Version zurück.
