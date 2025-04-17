---
title: Filtrado de Archivos en Code2Prompt
description: Una guía paso a paso para incluir o excluir archivos utilizando diferentes métodos de filtrado.
---


## Uso

Generar un prompt desde un directorio de base de código:

```sh
code2prompt path/to/codebase
```

Utilizar un archivo de plantilla Handlebars personalizado:

```sh
code2prompt path/to/codebase -t path/to/template.hbs
```

Filtrar archivos utilizando patrones glob:

```sh
code2prompt path/to/codebase --include="*.rs,*.toml"
```

Excluir archivos utilizando patrones glob:

```sh
code2prompt path/to/codebase --exclude="*.txt,*.md"
```

Excluir archivos/carpeta del árbol de origen según patrones de exclusión:

```sh
code2prompt path/to/codebase --exclude="*.npy,*.wav" --exclude-from-tree
```

Mostrar el recuento de tokens del prompt generado:

```sh
code2prompt path/to/codebase --tokens
```

Especificar un tokenizador para el recuento de tokens:

```sh
code2prompt path/to/codebase --tokens --encoding=p50k
```

Tokenizadores compatibles: `cl100k`, `p50k`, `p50k_edit`, `r50k_bas`.
> [!NOTE]  
> Consulte [Tokenizadores](#tokenizadores) para obtener más detalles.

Guardar el prompt generado en un archivo de salida:

```sh
code2prompt path/to/codebase --output=output.txt
```

Imprimir la salida como JSON:

```sh
code2prompt path/to/codebase --json
```

La salida JSON tendrá la siguiente estructura:

```json
{
  "prompt": "<Prompt generado>", 
  "directory_name": "codebase",
  "token_count": 1234,
  "model_info": "Modelos ChatGPT, text-embedding-ada-002",
  "files": []
}
```

Generar un mensaje de commit de Git (para archivos en staging):

```sh
code2prompt path/to/codebase --diff -t templates/write-git-commit.hbs
```

Generar una solicitud de Pull Request con comparación de ramas (para archivos en staging):

```sh
code2prompt path/to/codebase --git-diff-branch 'main, development' --git-log-branch 'main, development' -t templates/write-github-pull-request.hbs
```

Agregar números de línea a bloques de código fuente:

```sh
code2prompt path/to/codebase --line-number
```

Deshabilitar el ajuste de código dentro de bloques de código markdown:

```sh
code2prompt path/to/codebase --no-codeblock
```

- Reescribir el código en otro lenguaje.
- Encontrar errores/vulnerabilidades de seguridad.
- Documentar el código.
- Implementar nuevas características.

> Inicialmente escribí esto para uso personal para aprovechar la ventana de contexto de 200K de Claude 3.0 y resultó ser bastante útil, así que decidí open-sourcearlo.

> Esta página ha sido traducida automáticamente para su conveniencia. Consulte la versión en inglés para ver el contenido original.
