---
title: Glob-Muster
description: Eine Einführung in Glob-Muster, die Platzhalterzeichen verwenden, um Dateinamen und -pfade abzugleichen.
---

Glob-Muster sind eine einfache, aber leistungsstarke Möglichkeit, Dateinamen und -pfade mithilfe von Platzhalterzeichen abzugleichen. Sie werden häufig in Kommandozeilen-Interfaces und Programmiersprachen verwendet, um Mengen von Dateinamen oder Verzeichnissen anzugeben. Hier ist eine Aufschlüsselung der am häufigsten verwendeten Glob-Muster:

## Grundlegende Platzhalter

- `*`: Passt auf eine beliebige Anzahl von Zeichen, einschließlich null Zeichen.
  - Beispiel: `*.txt` passt auf alle Dateien, die mit `.txt` enden.

- `?`: Passt auf genau ein Zeichen.
  - Beispiel: `file?.txt` passt auf `file1.txt`, `fileA.txt`, aber nicht auf `file10.txt`.

- `[]`: Passt auf jedes der eingeschlossenen Zeichen.
  - Beispiel: `file[1-3].txt` passt auf `file1.txt`, `file2.txt`, `file3.txt`.

- `[!]` oder `[^]`: Passt auf jedes Zeichen, das nicht eingeschlossen ist.
  - Beispiel: `file[!1-3].txt` passt auf `file4.txt`, `fileA.txt`, aber nicht auf `file1.txt`.

## Erweiterte Muster

- `**`: Passt auf eine beliebige Anzahl von Verzeichnissen und Unterverzeichnissen rekursiv.
  - Beispiel: `**/*.txt` passt auf alle `.txt`-Dateien im aktuellen Verzeichnis und in allen Unterverzeichnissen.

- `{}`: Passt auf jedes der durch Kommas getrennten Muster, die eingeschlossen sind.
  - Beispiel: `file{1,2,3}.txt` passt auf `file1.txt`, `file2.txt`, `file3.txt`.

## Beispiele

1. **Alle Textdateien in einem Verzeichnis abgleichen:**

   ```sh
   *.txt
   ```

2. **Alle Dateien mit einer einzelnen Ziffer vor der Erweiterung abgleichen:**

   ```sh
   file?.txt
   ```

3. **Dateien mit den Erweiterungen `.jpg` oder `.png` abgleichen:**

   ```sh
   *.{jpg,png}
   ```

4. **Alle `.txt`-Dateien in einem beliebigen Unterverzeichnis abgleichen:**

   ```sh
   **/*.txt
   ```

5. **Dateien, die mit `a` oder `b` beginnen und mit `.txt` enden, abgleichen:**

   ```sh
   {a,b}*.txt
   ```

## Anwendungsfälle

- **Kommandozeilen-Tools:** Glob-Muster werden umfassend in Kommandozeilen-Tools wie `ls`, `cp`, `mv` und `rm` verwendet, um mehrere Dateien oder Verzeichnisse anzugeben.
- **Programmiersprachen:** Sprachen wie Python, JavaScript und Ruby unterstützen Glob-Muster für die Dateimatching über Bibliotheken wie `glob` in Python.
- **Build-Systeme:** Tools wie Makefile verwenden Glob-Muster, um Quelldateien und Abhängigkeiten anzugeben.

## Schlussfolgerung

Glob-Muster bieten eine flexible und intuitive Möglichkeit, Dateinamen und -pfade abzugleichen, was sie für Skripting, Automatisierung und Dateiverwaltungsaufgaben unverzichtbar macht. Das Verständnis und die Nutzung dieser Muster können Ihre Produktivität und Effizienz bei der Handhabung von Dateien und Verzeichnissen erheblich steigern.

> Diese Seite wurde für Ihre Bequemlichkeit automatisch übersetzt. Bitte greifen Sie für den Originalinhalt auf die englische Version zurück.
