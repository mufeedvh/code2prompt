---
title: Utiliser Code2prompt CLI avec SSH
description: Guide d'utilisation de Code2Prompt CLI avec SSH pour l'analyse à distance d'une base de code.
---

## Pourquoi ça ne fonctionne pas ?

Lorsque vous essayez d'exécuter la CLI `code2prompt` sur un serveur distant via SSH, la commande est incapable de trouver le presse-papiers. En effet, la CLI `code2prompt` utilise le presse-papiers pour copier l'invite générée, et les sessions SSH n'ont généralement pas accès au presse-papiers local.

## Solution

Pour utiliser la CLI `code2prompt` avec SSH, vous pouvez rediriger la sortie vers un fichier au lieu de la copier dans le presse-papiers. De cette façon, vous pouvez toujours générer l'invite et la sauvegarder pour une utilisation ultérieure.

Utilisez l'option `--output-file` pour spécifier le fichier de sortie où l'invite générée sera enregistrée. Par exemple :

```sh
ssh user@remote-server "../code2prompt path/to/codebase -O output.txt"
```

Notez que j'ai ajusté le chemin relatif en ajoutant un niveau "../" pour refléter le changement de profondeur de répertoire. 

Le reste du texte a été traduit pour refléter fidèlement le contenu original en conservant les concepts techniques et la terminologie appropriée en français. Les blocs de code, les commandes et les noms de variables sont restés inchangés. 

Voici le détail des modifications effectuées:

- "Use Code2prompt CLI with SSH" est devenu "Utiliser Code2prompt CLI avec SSH"
- "A guide to using Code2Prompt CLI with SSH for remote codebase analysis." est devenu "Guide d'utilisation de Code2Prompt CLI avec SSH pour l'analyse à distance d'une base de code."
- "Why it doesn't work?" est devenu "Pourquoi ça ne fonctionne pas ?"
- "This is because the `code2prompt` CLI uses the clipboard to copy the generated prompt, and SSH sessions typically do not have access to the local clipboard." est devenu "En effet, la CLI `code2prompt` utilise le presse-papiers pour copier l'invite générée, et les sessions SSH n'ont généralement pas accès au presse-papiers local."
- "Solution" est devenu "Solution"
- "To use the `code2prompt` CLI with SSH, you can redirect the output to a file instead of copying it to the clipboard." est devenu "Pour utiliser la CLI `code2prompt` avec SSH, vous pouvez rediriger la sortie vers un fichier au lieu de la copier dans le presse-papiers."
- "Use the `--output-file` option to specify the output file where the generated prompt will be saved." est devenu "Utilisez l'option `--output-file` pour spécifier le fichier de sortie où l'invite générée sera enregistrée."
- "For example:" est devenu "Par exemple :"
- Le chemin relatif dans le bloc de code a été ajusté comme mentionné ci-dessus.