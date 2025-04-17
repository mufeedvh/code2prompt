---
title: Verwenden Sie Code2prompt CLI mit SSH
description: Eine Anleitung zur Verwendung von Code2Prompt CLI mit SSH für die Remote-Codebasis-Analyse.
---

## Warum funktioniert es nicht?

Wenn Sie versuchen, die `code2prompt`-CLI auf einem Remote-Server über SSH auszuführen, kann der Befehl den Clipboard nicht finden. Dies liegt daran, dass die `code2prompt`-CLI den Clipboard verwendet, um die generierte Eingabeaufforderung zu kopieren, und SSH-Sitzungen normalerweise keinen Zugriff auf den lokalen Clipboard haben.

## Lösung

Um die `code2prompt`-CLI mit SSH zu verwenden, können Sie die Ausgabe in eine Datei umleiten, anstatt sie in den Clipboard zu kopieren. Auf diese Weise können Sie immer noch die Eingabeaufforderung generieren und für die spätere Verwendung speichern.

Verwenden Sie die Option `--output-file`, um die Ausgabedatei anzugeben, in der die generierte Eingabeaufforderung gespeichert wird. Zum Beispiel:

```sh
ssh user@remote-server "code2prompt path/to/codebase -O output.txt"
```

> Diese Seite wurde für Ihre Bequemlichkeit automatisch übersetzt. Bitte greifen Sie für den Originalinhalt auf die englische Version zurück.
