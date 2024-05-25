# Logique

## Description du Problème

Je dois sélectionner des fichiers dans une hiérarchie basée sur plusieurs critères : extension, nom de fichier et dossier. Les critères peuvent être inclus dans des whitelistes (listes blanches) ou des blacklists (listes noires). Par défaut, tous les fichiers sont inclus, mais les spécifications de whitelistes ou de blacklists modifient cette sélection. Si un conflit existe entre une whitelist et une blacklist du même critère, le programme retourne un avertissement indiquant que la blacklist a été ignorée et que la whitelist a été préférée. Si il y a un conflit entre des critères différents, les whitelistes et les blacklists doivent être combinées en priorisant les whitelistes. Si un critère a été explicitement autorisé, le fichier devra être inclus même s'il est également dans une blacklist. Mais pas le contraire.


## Flags Utilisés

- `--include-extension`
- `--exclude-extension`
- `--include-file`
- `--exclude-file`
- `--include-folder`
- `--exclude-folder`

## Logique Formelle

1. **Définitions des Critères**:
    - \(A\) : Whitelist extension (`--include-extension`)
    - \(B\) : Blacklist extension (`--exclude-extension`)
    - \(C\) : Whitelist fichier (`--include-file`)
    - \(D\) : Blacklist fichier (`--exclude-file`)
    - \(E\) : Whitelist dossier (`--include-folder`)
    - \(F\) : Blacklist dossier (`--exclude-folder`)

2. **Règles de Sélection**:
    - Par défaut, tous les fichiers sont inclus.
    - Si une whiteliste est spécifiée, seuls les éléments de cette whiteliste sont conservés.
    - Si une blacklist est spécifiée, tous les fichiers sauf ceux de la blacklist sont conservés.
    - Les whitelistes ont priorité sur les blacklists. Cela signifie que si un fichier est spécifié dans une whiteliste, il est inclus même s'il est également spécifié dans une blacklist.

3. **Expression Logique Inspirée de l'Expression d'origine**:
    - **Négation des Blacklists**:
      \[
      B' = \neg B \quad \text{(Négation de Blacklist extension)}
      \]
      \[
      D' = \neg D \quad \text{(Négation de Blacklist fichier)}
      \]
      \[
      F' = \neg F \quad \text{(Négation de Blacklist dossier)}
      \]
    - **Union de Whitelists**:
      \[
      W = (A \cup C \cup E)
      \]
    - **Union des Négations de Blacklists**:
      \[
      \neg B_{\text{total}} = B' \cup D' \cup F'
      \]
    - **Différence Whitelist - Blacklist**:
      \[
      W' = W \cap \neg B_{\text{total}}
      \]
    - **Sélection Finale**:
      \[
      F_{\text{final}} = \begin{cases} 
      W' & \text{si} \ W \ \neq \emptyset \\
      \neg B_{\text{total}} & \text{sinon}
      \end{cases}
      \]

## Simplification de l'Expression Logique

Pour intégrer correctement les whitelists et les blacklists tout en donnant la priorité aux whitelists, nous devons ajuster la logique pour que chaque conflit soit résolu en faveur des whitelists :

- Si un fichier est dans une whitelist quelconque, il doit être inclus, même s'il est également dans une blacklist.

### Exemple de Cas

#### Cas 1 : Seules des Whitelistes
- **Input**:
  - Whitelist extension : .txt (`--include-extension .txt`)
  - Whitelist fichier : report (`--include-file report`)
  - Whitelist folder : /docs (`--include-folder /docs`)
- **Expression**:
  - \(W = \{\text{.txt}\} \cup \{\text{report}\} \cup \{\text{/docs}\}\)
  - Résultat : Fichiers avec extension .txt, ou nom contenant "report", ou dans le dossier /docs

#### Cas 2 : Whitelistes et Blacklists
- **Input**:
  - Whitelist extension : .txt (`--include-extension .txt`)
  - Blacklist fichier : draft (`--exclude-file draft`)
  - Blacklist folder : /tmp (`--exclude-folder /tmp`)
- **Expression**:
  - \(W = \{\text{.txt}\}\)
  - \(\neg B_{\text{total}} = \neg \{\text{draft}\} \cup \neg \{\text{/tmp}\}\)
  - \(W' = \{\text{.txt}\} \cap (\neg \{\text{draft}\} \cup \neg \{\text{/tmp}\})\)
  - Résultat : Fichiers avec extension .txt sauf ceux nommés "draft" et ceux dans le dossier /tmp

#### Cas 3 : Priorité Whitelist sur Blacklist
- **Input**:
  - Whitelist fichier : coco.txt (`--include-file coco.txt`)
  - Blacklist extension : .txt (`--exclude-extension .txt`)
  - Whitelist extension : .pdf (`--include-extension .pdf`)
  - Blacklist dossier : /tmp (`--exclude-folder /tmp`)
- **Expression**:
  - \(W = \{\text{coco.txt}\} \cup \{\text{.pdf}\}\)
  - \(\neg B_{\text{total}} = \neg \{\text{.txt}\} \cup \neg \{\text{/tmp}\}\)
  - \(W' = (\{\text{coco.txt}\} \cup \{\text{.pdf}\}) \cap (\neg \{\text{.txt}\} \cup \neg \{\text{/tmp}\})\)
  - Résultat : Fichiers nommés "coco.txt", fichiers avec extension .pdf sauf ceux dans le dossier /tmp

### Conclusion

Cette logique garantit que les whitelists ont toujours priorité sur les blacklists, même en cas de conflit. Les fichiers explicitement inclus dans une whitelist seront conservés même s'ils correspondent à un critère de blacklist. Cette approche offre une flexibilité et une clarté maximales pour la sélection des fichiers en fonction des critères spécifiés.