---
title: Utiliser Code2prompt CLI avec SSH
description: Un guide pour utiliser Code2Prompt CLI avec SSH pour l'analyse à distance d'une base de code.
---

## Pourquoi ça ne fonctionne pas ?

Lorsque vous essayez d'exécuter la CLI `code2prompt` sur un serveur distant via SSH, la commande est incapable de trouver le presse-papiers. En effet, la CLI `code2prompt` utilise le presse-papiers pour copier l'invite générée, et les sessions SSH n'ont généralement pas accès au presse-papiers local.

## Solution

Pour utiliser la CLI `code2prompt` avec SSH, vous pouvez rediriger la sortie vers un fichier au lieu de la copier dans le presse-papiers. De cette façon, vous pouvez toujours générer l'invite et la sauvegarder pour une utilisation ultérieure.

Utilisez l'option `--output-file` pour spécifier le fichier de sortie où l'invite générée sera enregistrée. Par exemple :

```sh
ssh user@remote-server "code2prompt path/to/codebase -O output.txt"
```

> Cette page a été traduite automatiquement pour votre commodité. Veuillez vous référer à la version anglaise pour le contenu original.
