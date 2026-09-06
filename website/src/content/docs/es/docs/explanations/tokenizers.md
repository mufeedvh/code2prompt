---
title: Tokenización en Code2Prompt
description: Aprende sobre la tokenización y cómo Code2Prompt procesa texto para LLMs.
---

Cuando se trabaja con modelos de lenguaje, el texto debe transformarse en un formato que el modelo pueda entender: **tokens**, que son secuencias de números. Esta transformación se realiza mediante un **tokenizador**.

---

El recuento indicado es una **estimación**: Code2Prompt cuenta los archivos en
paralelo después del procesamiento y la numeración de líneas opcional, y añade el
coste estimado de la plantilla. El prompt completo no se vuelve a tokenizar. Los
límites entre fragmentos y las transformaciones de la plantilla pueden cambiar el
resultado; la estimación no es un límite superior garantizado. El campo numérico
`token_count` conserva su nombre y excluye la envoltura JSON y los escapes de
serialización.

## ¿Qué es un Tokenizador?

Un tokenizador convierte texto sin procesar en tokens, que son los bloques de construcción para cómo los modelos de lenguaje procesan la entrada. Estos tokens pueden representar palabras, subpalabras o incluso caracteres individuales, dependiendo del diseño del tokenizador.

Para `code2prompt`, utilizamos el tokenizador **tiktoken**. Es eficiente, robusto y optimizado para modelos de OpenAI.
Puedes explorar su funcionalidad en el repositorio oficial

👉 [Repositorio de GitHub de tiktoken](https://github.com/openai/tiktoken)

Si deseas aprender más sobre tokenizadores en general, consulta

👉 [Guía de Tokenización de Mistral](https://docs.mistral.ai/guides/tokenization/).

## Implementación en `code2prompt`

La tokenización se implementa utilizando [`tiktoken-rs`](https://github.com/zurawiki/tiktoken-rs). `tiktoken` admite estos codificaciones utilizadas por los modelos de OpenAI:

| Argumento de CLI | Nombre de codificación  | Modelos de OpenAI                                                         |
|----|-----------------------| ------------------------------------------------------------------------- |
|`cl100k`| `cl100k_base`           | Modelos de ChatGPT, `text-embedding-ada-002`                              |
|`p50k`| `p50k_base`             | Modelos de código, `text-davinci-002`, `text-davinci-003`                 |
|`p50k_edit`| `p50k_edit`             | Utilizar para modelos de edición como `text-davinci-edit-001`, `code-davinci-edit-001` |
|`r50k`| `r50k_base` (o `gpt2`) | Modelos de GPT-3 como `davinci`                                            |
|`gpt2`| `o200k_base`            | Modelos de GPT-4o                                                         |

Para obtener más contexto sobre los diferentes tokenizadores, consulta [OpenAI Cookbook](https://github.com/openai/openai-cookbook/blob/66b988407d8d13cad5060a881dc8c892141f2d5c/examples/How_to_count_tokens_with_tiktoken.ipynb)

> Esta página ha sido traducida automáticamente para su conveniencia. Consulte la versión en inglés para ver el contenido original.
