---
title: Filtern von Dateien in Code2Prompt
description: Eine Schritt-für-Schritt-Anleitung zum Einschließen oder Ausschließen von Dateien mithilfe verschiedener Filtermethoden.
---


## Verwendung

Generieren Sie einen Prompt aus einem Codebasis-Verzeichnis:

```sh
code2prompt path/to/codebase
```

Verwenden Sie eine benutzerdefinierte Handlebars-Vorlagendatei:

```sh
code2prompt path/to/codebase -t path/to/template.hbs
```

Filtern Sie Dateien mithilfe von Glob-Mustern:

```sh
code2prompt path/to/codebase --include="*.rs,*.toml"
```

Schließen Sie Dateien mithilfe von Glob-Mustern aus:

```sh
code2prompt path/to/codebase --exclude="*.txt,*.md"
```

Schließen Sie Dateien/Ordner aus dem Quellbaum basierend auf Ausschlussmustern aus:

```sh
code2prompt path/to/codebase --exclude="*.npy,*.wav" --exclude-from-tree
```

Zeigen Sie die Tokenanzahl des generierten Prompts an:

```sh
code2prompt path/to/codebase --tokens
```

Geben Sie einen Tokenizer für die Tokenanzahl an:

```sh
code2prompt path/to/codebase --tokens --encoding=p50k
```

Unterstützte Tokenizer: `cl100k`, `p50k`, `p50k_edit`, `r50k_bas`.
> [!HINWEIS]  
> Siehe [Tokenizer](#tokenizers) für weitere Details.

Speichern Sie den generierten Prompt in einer Ausgabedatei:

```sh
code2prompt path/to/codebase --output=output.txt
```

Drucken Sie die Ausgabe als JSON:

```sh
code2prompt path/to/codebase --json
```

Die JSON-Ausgabe hat die folgende Struktur:

```json
{
  "prompt": "<Generierter Prompt>", 
  "directory_name": "codebase",
  "token_count": 1234,
  "model_info": "ChatGPT-Modelle, text-embedding-ada-002",
  "files": []
}
```

Generieren Sie eine Git-Commit-Nachricht (für bereitgestellte Dateien):

```sh
code2prompt path/to/codebase --diff -t templates/write-git-commit.hbs
```

Generieren Sie eine Pull-Anfrage mit Branch-Vergleich (für bereitgestellte Dateien):

```sh
code2prompt path/to/codebase --git-diff-branch 'main, development' --git-log-branch 'main, development' -t templates/write-github-pull-request.hbs
```

Fügen Sie Zeilennummern zu Quellcodeblöcken hinzu:

```sh
code2prompt path/to/codebase --line-number
```

Deaktivieren Sie das Umbrechen von Code innerhalb von Markdown-Codeblöcken:

```sh
code2prompt path/to/codebase --no-codeblock
```

- Übersetzen Sie den Code in eine andere Sprache.
- Suchen Sie nach Fehlern/Sicherheitslücken.
- Dokumentieren Sie den Code.
- Implementieren Sie neue Funktionen.

> Ich habe dies ursprünglich für den persönlichen Gebrauch geschrieben, um das 200K-Kontextfenster von Claude 3.0 zu nutzen, und es hat sich als ziemlich nützlich erwiesen, daher habe ich mich entschieden, es Open-Source zu machen!

> Diese Seite wurde für Ihre Bequemlichkeit automatisch übersetzt. Bitte greifen Sie für den Originalinhalt auf die englische Version zurück.
