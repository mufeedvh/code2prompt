---
title: Comprendre les modèles Glob
description: Une explication détaillée des modèles Glob et de leur utilisation dans Code2Prompt.
---

Les modèles Glob sont un moyen simple mais puissant de faire correspondre les noms de fichiers et les chemins d'accès à l'aide de caractères génériques. Ils sont couramment utilisés dans les interfaces de ligne de commande et les langages de programmation pour spécifier des ensembles de noms de fichiers ou de répertoires. Voici une analyse des modèles Glob les plus couramment utilisés :

## Générateurs de base

- `*` : Correspond à tout nombre de caractères, y compris zéro caractère.
  - Exemple : `*.txt` correspond à tous les fichiers se terminant par `.txt`.

- `?` : Correspond exactement à un caractère.
  - Exemple : `file?.txt` correspond à `file1.txt`, `fileA.txt`, mais pas à `file10.txt`.

- `[]` : Correspond à l'un des caractères enfermés.
  - Exemple : `file[1-3].txt` correspond à `file1.txt`, `file2.txt`, `file3.txt`.

- `[!]` ou `[^]` : Correspond à tout caractère non enfermé.
  - Exemple : `file[!1-3].txt` correspond à `file4.txt`, `fileA.txt`, mais pas à `file1.txt`.

## Modèles avancés

- `**` : Correspond à tout nombre de répertoires et sous-répertoires de manière récursive.
  - Exemple : `**/*.txt` correspond à tous les fichiers `.txt` dans le répertoire actuel et tous les sous-répertoires.

- `{}` : Correspond à l'un des modèles séparés par des virgules enfermés.
  - Exemple : `file{1,2,3}.txt` correspond à `file1.txt`, `file2.txt`, `file3.txt`.

## Exemples

1. **Faire correspondre tous les fichiers texte dans un répertoire :**

   ```sh
   *.txt
   ```

2. **Faire correspondre tous les fichiers avec un seul chiffre avant l'extension :**

   ```sh
   file?.txt
   ```

3. **Faire correspondre les fichiers avec les extensions `.jpg` ou `.png` :**

   ```sh
   *.{jpg,png}
   ```

4. **Faire correspondre tous les fichiers `.txt` dans n'importe quel sous-répertoire :**

   ```sh
   **/*.txt
   ```

5. **Faire correspondre les fichiers qui commencent par `a` ou `b` et se terminent par `.txt` :**

   ```sh
   {a,b}*.txt
   ```

## Cas d'utilisation

- **Outils de ligne de commande :** Les modèles Glob sont largement utilisés dans les outils de ligne de commande tels que `ls`, `cp`, `mv` et `rm` pour spécifier plusieurs fichiers ou répertoires.
- **Langages de programmation :** Les langages tels que Python, JavaScript et Ruby prennent en charge les modèles Glob pour la correspondance de fichiers via des bibliothèques telles que `glob` en Python.
- **Systèmes de build :** Des outils tels que Makefile utilisent des modèles Glob pour spécifier les fichiers source et les dépendances.

## Conclusion

Les modèles Glob fournissent un moyen flexible et intuitif de faire correspondre les noms de fichiers et les chemins d'accès, les rendant indispensables pour les tâches de script, d'automatisation et de gestion de fichiers. Comprendre et utiliser ces modèles peut considérablement améliorer votre productivité et votre efficacité dans la gestion des fichiers et des répertoires.

> Cette page a été traduite automatiquement pour votre commodité. Veuillez vous référer à la version anglaise pour le contenu original.
