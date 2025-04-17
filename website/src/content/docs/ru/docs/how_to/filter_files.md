---
title: Фильтрация файлов в Code2Prompt
description: Пошаговое руководство по включению или исключению файлов с помощью различных методов фильтрации.
---


## Использование

Сгенерировать запрос из директории codebase:

```sh
code2prompt path/to/codebase
```

Использовать пользовательский файл шаблона Handlebars:

```sh
code2prompt path/to/codebase -t path/to/template.hbs
```

Фильтровать файлы с помощью шаблонов glob:

```sh
code2prompt path/to/codebase --include="*.rs,*.toml"
```

Исключить файлы с помощью шаблонов glob:

```sh
code2prompt path/to/codebase --exclude="*.txt,*.md"
```

Исключить файлы/папки из дерева исходных файлов на основе шаблонов исключения:

```sh
code2prompt path/to/codebase --exclude="*.npy,*.wav" --exclude-from-tree
```

Отобразить количество токенов сгенерированного запроса:

```sh
code2prompt path/to/codebase --tokens
```

Указать токенизатор для подсчета токенов:

```sh
code2prompt path/to/codebase --tokens --encoding=p50k
```

Поддерживаемые токенизаторы: `cl100k`, `p50k`, `p50k_edit`, `r50k_bas`.
> [!ПРИМЕЧАНИЕ]  
> См. [Токенизаторы](#tokenizers) для более подробной информации.

Сохранить сгенерированный запрос в выходной файл:

```sh
code2prompt path/to/codebase --output=output.txt
```

Вывести результат в формате JSON:

```sh
code2prompt path/to/codebase --json
```

Выходные данные в формате JSON будут иметь следующую структуру:

```json
{
  "prompt": "<Сгенерированный запрос>", 
  "directory_name": "codebase",
  "token_count": 1234,
  "model_info": "Модели ChatGPT, text-embedding-ada-002",
  "files": []
}
```

Сгенерировать сообщение коммита Git (для staged файлов):

```sh
code2prompt path/to/codebase --diff -t templates/write-git-commit.hbs
```

Сгенерировать запрос на Pull Request с сравнением веток (для staged файлов):

```sh
code2prompt path/to/codebase --git-diff-branch 'main, development' --git-log-branch 'main, development' -t templates/write-github-pull-request.hbs
```

Добавить номера строк к блокам исходного кода:

```sh
code2prompt path/to/codebase --line-number
```

Отключить оборачивание кода внутри блоков markdown:

```sh
code2prompt path/to/codebase --no-codeblock
```

- Переписать код на другой язык.
- Найти ошибки/уязвимости безопасности.
- Документировать код.
- Реализовать новые функции.

> Изначально я написал это для личного использования, чтобы использовать окно контекста Claude 3.0 размером 200K, и оно оказалось довольно полезным, поэтому я решил сделать его открытым!

> Эта страница была автоматически переведена для вашего удобства. Обратитесь к английской версии для получения оригинального содержания.
