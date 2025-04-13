---
title: Comprendre les modèles Glob
description: Une explication détaillée des modèles Glob et de leur utilisation dans Code2Prompt.
---

Les modèles Glob sont un moyen simple mais puissant de faire correspondre les noms de fichiers et les chemins d'accès à l'aide de caractères génériques. Ils sont couramment utilisés dans les interfaces de ligne de commande et les langages de programmation pour spécifier des ensembles de noms de fichiers ou de répertoires. Voici une ventilation des modèles Glob les plus couramment utilisés :

## Wildcards de base

- `*` : Correspond à un nombre quelconque de caractères, y compris zéro caractère.
  - Exemple : `*.txt` correspond à tous les fichiers se terminant par `.txt`.

- `?` : Correspond exactement à un caractère.
  - Exemple : `file?.txt` correspond à `file1.txt`, `fileA.txt`, mais pas à `file10.txt`.

- `[]` : Correspond à l'un des caractères enfermés.
  - Exemple : `file[1-3].txt` correspond à `file1.txt`, `file2.txt`, `file3.txt`.

- `[!]` ou `[^]` : Correspond à tout caractère non enfermé.
  - Exemple : `file[!1-3].txt` correspond à `file4.txt`, `fileA.txt`, mais pas à `file1.txt`.

## Modèles avancés

- `**` : Correspond à un nombre quelconque de répertoires et sous-répertoires de manière récursive.
  - Exemple : `**/*.txt` correspond à tous les fichiers `.txt` dans le répertoire courant et tous les sous-répertoires.

- `{}` : Correspond à l'un des modèles séparés par des virgules enfermés.
  - Exemple : `file{1,2,3}.txt` correspond à `file1.txt`, `file2.txt`, `file3.txt`.

## Exemples

1. **Correspondance à tous les fichiers texte dans un répertoire :**

   ```sh
   *.txt
   ```

2. **Correspondance à tous les fichiers avec un seul chiffre avant l'extension :**

   ```sh
   file?.txt
   ```

3. **Correspondance à des fichiers avec des extensions `.jpg` ou `.png` :**

   ```sh
   *.{jpg,png}
   ```

4. **Correspondance à tous les fichiers `.txt` dans n'importe quel sous-répertoire :**

   ```sh
   **/*.txt
   ```

5. **Correspondance à des fichiers commençant par `a` ou `b` et se terminant par `.txt` :**

   ```sh
   {a,b}*.txt
   ```

## Cas d'utilisation

- **Outils de ligne de commande :** Les modèles Glob sont largement utilisés dans les outils de ligne de commande tels que `ls`, `cp`, `mv` et `rm` pour spécifier plusieurs fichiers ou répertoires.
- **Langages de programmation :** Des langages tels que Python, JavaScript et Ruby prennent en charge les modèles Glob pour la correspondance de fichiers via des bibliothèques telles que `glob` en Python.
- **Systèmes de build :** Des outils tels que Makefile utilisent des modèles Glob pour spécifier les fichiers source et les dépendances.

## Conclusion

Les modèles Glob offrent un moyen flexible et intuitif de faire correspondre les noms de fichiers et les chemins d'accès, les rendant indispensables pour les tâches de script, d'automatisation et de gestion de fichiers. Comprendre et utiliser ces modèles peut considérablement améliorer votre productivité et votre efficacité dans la gestion des fichiers et des répertoires.