---
title: Filtrage de fichiers dans Code2Prompt
description: Un guide étape par étape pour inclure ou exclure des fichiers à l'aide de différentes méthodes de filtrage.
---


## Utilisation

Générez une invite à partir d'un répertoire de base de code :

```sh
code2prompt path/to/codebase
```

Utilisez un fichier de modèle Handlebars personnalisé :

```sh
code2prompt path/to/codebase -t path/to/template.hbs
```

Filtrez les fichiers à l'aide de modèles glob :

```sh
code2prompt path/to/codebase --include="*.rs,*.toml"
```

Excluez les fichiers à l'aide de modèles glob :

```sh
code2prompt path/to/codebase --exclude="*.txt,*.md"
```

Excluez les fichiers/dossiers de l'arborescence source en fonction des modèles d'exclusion :

```sh
code2prompt path/to/codebase --exclude="*.npy,*.wav" --exclude-from-tree
```

Affichez le nombre de jetons de l'invite générée :

```sh
code2prompt path/to/codebase --tokens
```

Spécifiez un tokenizeur pour le décompte des jetons :

```sh
code2prompt path/to/codebase --tokens --encoding=p50k
```

Tokenizeurs pris en charge : `cl100k`, `p50k`, `p50k_edit`, `r50k_bas`.
> [!NOTE]  
> Voir [Tokenizeurs](#tokenizers) pour plus de détails.

Enregistrez l'invite générée dans un fichier de sortie :

```sh
code2prompt path/to/codebase --output=output.txt
```

Imprimez la sortie au format JSON :

```sh
code2prompt path/to/codebase --json
```

La sortie JSON aura la structure suivante :

```json
{
  "prompt": "<Invite générée>", 
  "directory_name": "codebase",
  "token_count": 1234,
  "model_info": "Modèles ChatGPT, text-embedding-ada-002",
  "files": []
}
```

Générez un message de commit Git (pour les fichiers en scène) :

```sh
code2prompt path/to/codebase --diff -t templates/write-git-commit.hbs
```

Générez une demande de tirage avec comparaison de branche (pour les fichiers en scène) :

```sh
code2prompt path/to/codebase --git-diff-branch 'main, development' --git-log-branch 'main, development' -t templates/write-github-pull-request.hbs
```

Ajoutez des numéros de ligne aux blocs de code source :

```sh
code2prompt path/to/codebase --line-number
```

Désactivez l'emballage de code à l'intérieur des blocs de code markdown :

```sh
code2prompt path/to/codebase --no-codeblock
```

- Réécrivez le code dans un autre langage.
- Recherchez des bogues/vulnérabilités de sécurité.
- Documentez le code.
- Implémentez de nouvelles fonctionnalités.

> J'ai initialement écrit cela pour une utilisation personnelle afin de profiter de la fenêtre de contexte de 200K de Claude 3.0 et cela s'est avéré assez utile, alors j'ai décidé de le rendre open-source !

> Cette page a été traduite automatiquement pour votre commodité. Veuillez vous référer à la version anglaise pour le contenu original.
