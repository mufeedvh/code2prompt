---
title: Tokenisation dans Code2Prompt
description: Découvrez la tokenisation et comment Code2Prompt traite le texte pour les LLMs.
---

Lorsque l'on travaille avec des modèles de langage, le texte doit être transformé en un format que le modèle peut comprendre : des **jetons** (tokens), qui sont des séquences de nombres. Cette transformation est gérée par un **tokeniseur**.

---

## Qu'est-ce qu'un Tokeniseur ?

Un tokeniseur convertit le texte brut en jetons, qui sont les briques de base pour la façon dont les modèles de langage traitent les entrées. Ces jetons peuvent représenter des mots, des sous-mots ou même des caractères individuels, en fonction de la conception du tokeniseur.

Pour `code2prompt`, nous utilisons le tokeniseur **tiktoken**. Il est efficace, robuste et optimisé pour les modèles OpenAI.
Vous pouvez explorer ses fonctionnalités dans le référentiel officiel

👉 [Référentiel GitHub de tiktoken](https://github.com/openai/tiktoken)

Si vous souhaitez en savoir plus sur les tokeniseurs en général, consultez le

👉 [Guide de tokenisation de Mistral](https://docs.mistral.ai/guides/tokenization/).

## Implémentation dans `code2prompt`

La tokenisation est implémentée à l'aide de [`tiktoken-rs`](https://github.com/zurawiki/tiktoken-rs). `tiktoken` prend en charge ces codages utilisés par les modèles OpenAI :

| Argument CLI | Nom de codage           | Modèles OpenAI                                                             |
| ---- | ----------------------- | ------------------------------------------------------------------------- |
| `cl100k` | `cl100k_base`           | Modèles ChatGPT, `text-embedding-ada-002`                                  |
| `p50k` | `p50k_base`             | Modèles de code, `text-davinci-002`, `text-davinci-003`                       |
| `p50k_edit` | `p50k_edit`             | Utiliser pour les modèles d'édition comme `text-davinci-edit-001`, `code-davinci-edit-001` |
| `r50k` | `r50k_base` (ou `gpt2`) | Modèles GPT-3 comme `davinci`                                               |
| `gpt2` | `o200k_base`            | Modèles GPT-4o                                                             |

Pour plus de contexte sur les différents tokeniseurs, consultez le [OpenAI Cookbook](https://github.com/openai/openai-cookbook/blob/66b988407d8d13cad5060a881dc8c892141f2d5c/examples/How_to_count_tokens_with_tiktoken.ipynb)

Note that I kept code blocks, commands, and variable names unchanged as per your request. I also adjusted no relative paths as there were none starting with "../".

Please let me know if I can help with anything else.

Also, I used French style and grammar guidelines from the "Rectifications de l'orthographe du français en 1990" and "Typographie française". If any changes are required according to your organization's style guide, please provide it.
