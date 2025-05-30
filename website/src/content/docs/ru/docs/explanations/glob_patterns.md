---
title: Понимание шаблонов Glob
description: Подробное объяснение шаблонов glob и их использования в Code2Prompt.
---

Шаблоны glob - это простой, но мощный способ сопоставления имен файлов и путей с использованием символов-заменителей. Они обычно используются в интерфейсах командной строки и языках программирования для указания наборов имен файлов или директорий. Вот разбор наиболее часто используемых шаблонов glob:

## Базовые шаблоны-заменители

- `*`: Сопоставляется с любым количеством символов, включая нулевое количество символов.
  - Пример: `*.txt` сопоставляется со всеми файлами, оканчивающимися на `.txt`.

- `?`: Сопоставляется ровно с одним символом.
  - Пример: `file?.txt` сопоставляется с `file1.txt`, `fileA.txt`, но не с `file10.txt`.

- `[]`: Сопоставляется с любым из заключенных внутри скобок символов.
  - Пример: `file[1-3].txt` сопоставляется с `file1.txt`, `file2.txt`, `file3.txt`.

- `[!]` или `[^]`: Сопоставляется с любым символом, не заключенным внутри скобок.
  - Пример: `file[!1-3].txt` сопоставляется с `file4.txt`, `fileA.txt`, но не с `file1.txt`.

## Расширенные шаблоны

- `**`: Сопоставляется с любым количеством директорий и поддиректорий рекурсивно.
  - Пример: `**/*.txt` сопоставляется со всеми файлами `.txt` в текущей директории и всех поддиректориях.

- `{}`: Сопоставляется с любым из шаблонов, перечисленных через запятую внутри скобок.
  - Пример: `file{1,2,3}.txt` сопоставляется с `file1.txt`, `file2.txt`, `file3.txt`.

## Примеры

1. **Сопоставление всех текстовых файлов в директории:**

   ```sh
   *.txt
   ```

2. **Сопоставление всех файлов с одним цифровым символом перед расширением:**

   ```sh
   file?.txt
   ```

3. **Сопоставление файлов с расширениями `.jpg` или `.png`:**

   ```sh
   *.{jpg,png}
   ```

4. **Сопоставление всех файлов `.txt` в любой поддиректории:**

   ```sh
   **/*.txt
   ```

5. **Сопоставление файлов, начинающихся с `a` или `b` и оканчивающихся на `.txt`:**

   ```sh
   {a,b}*.txt
   ```

## Варианты использования

- **Инструменты командной строки:** Шаблоны glob широко используются в инструментах командной строки, таких как `ls`, `cp`, `mv` и `rm`, для указания нескольких файлов или директорий.
- **Языки программирования:** Языки, такие как Python, JavaScript и Ruby, поддерживают шаблоны glob для сопоставления файлов через библиотеки, такие как `glob` в Python.
- **Системы сборки:** Инструменты, такие как Makefile, используют шаблоны glob для указания исходных файлов и зависимостей.

## Заключение

Шаблоны glob обеспечивают гибкий и интуитивный способ сопоставления имен файлов и путей, что делает их незаменимыми для задач сценариев, автоматизации и управления файлами. Понимание и использование этих шаблонов может существенно повысить вашу производительность и эффективность при работе с файлами и директориями.

> Эта страница была автоматически переведена для вашего удобства. Обратитесь к английской версии для получения оригинального содержания.
