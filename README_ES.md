# code2prompt

[![crates.io](https://img.shields.io/crates/v/code2prompt.svg)](https://crates.io/crates/code2prompt)
[![LICENSE](https://img.shields.io/github/license/mufeedvh/code2prompt.svg#cache1)](https://github.com/mufeedvh/code2prompt/blob/master/LICENSE)

<h1 align="center">
  <a href="https://github.com/mufeedvh/code2prompt"><img src=".assets/code2prompt-screenshot.png" alt="code2prompt"></a>
</h1>

`code2prompt` es una herramienta de línea de comandos (CLI) que convierte tu base de código en un único prompt para LLM, incluyendo un árbol de archivos fuente, plantillas de prompts y conteo de tokens.

## Tabla de Contenidos

- [Características](#features)
- [Instalación](#installation)
- [Uso](#usage)
- [Plantillas](#templates)
- [Variables Definidas por el Usuario](#user-defined-variables)
- [Tokenizadores](#tokenizers)
- [Contribución](#contribution)
- [Licencia](#license)
- [Apoya al Autor](#support-the-author)

## Características

Puedes ejecutar esta herramienta en un directorio completo, y generará un prompt bien formateado en Markdown que detalla la estructura del árbol de archivos fuente y todo el código. Luego puedes cargar este documento en modelos como GPT o Claude con ventanas de contexto amplias y pedirles que:

- Generen prompts para LLM rápidamente a partir de bases de código de cualquier tamaño.
- Personalicen la generación de prompts usando plantillas de Handlebars (ver la [plantilla predeterminada](src/default_template.hbs))
- Respete los archivos `.gitignore`.
- Filtren y excluyan archivos utilizando patrones glob.
- Muestren el conteo de tokens del prompt generado (Ver [Tokenizadores](#tokenizers) para más detalles).
- Incluyan opcionalmente salidas de `git diff` (archivos en estado staged) en el prompt generado.
- Copien automáticamente el prompt generado al portapapeles.
- Guarden el prompt generado en un archivo de salida.
- Excluyan archivos y carpetas por nombre o ruta.
- Añadan números de línea a los bloques de código fuente.

Puedes personalizar las plantillas de prompts para lograr cualquier caso de uso deseado. Básicamente, recorre una base de código y crea un prompt con todos los archivos fuente combinados. En resumen, automatiza la tarea de copiar y formatear múltiples archivos fuente en un único prompt y te informa cuántos tokens consume.

## Instalación

### Lanzamiento de binarios
Descarga el binario más reciente para tu sistema operativo desde [Releases](https://github.com/mufeedvh/code2prompt/releases).

### Construcción desde código fuente
Requisitos:

- [Git](https://git-scm.org/downloads), [Rust](https://rust-lang.org/tools/install) y Cargo.

```sh
git clone https://github.com/mufeedvh/code2prompt.git
cd code2prompt/
cargo build --release
```

## cargo

Instala desde el registro [`crates.io`](https://crates.io).

```sh
cargo install code2prompt
```

Para versiones no publicadas:

```sh
cargo install --git https://github.com/mufeedvh/code2prompt
```

### AUR
`code2prompt` está disponible en [`AUR`](https://aur.archlinux.org/packages?O=0&K=code2prompt).  Instálalo usando cualquier gestor AUR.

```sh
paru/yay -S code2prompt
```

### Nix
Si utilizas Nix, puedes instalarlo con `nix-env` o `profile`:

```sh
# Sin flakes:
nix-env -iA nixpkgs.code2prompt
# Con flakes:
nix profile install nixpkgs#code2prompt
```

## Uso

Genera un prompt desde un directorio de código:

```sh
code2prompt path/to/codebase
```

Usa un archivo de plantilla Handlebars personalizado:

```sh
code2prompt path/to/codebase -t path/to/template.hbs
```

Filtrar archivos usando patrones glob:

```sh
code2prompt path/to/codebase --include="*.rs,*.toml"
```

Excluir archivos usando patrones glob:

```sh
code2prompt path/to/codebase --exclude="*.txt,*.md"
```

Excluir archivos/carpetas del árbol de origen basándose en patrones de exclusión:

```sh
code2prompt path/to/codebase --exclude="*.npy,*.wav" --exclude-from-tree
```

Mostrar el conteo de tokens del prompt generado:

```sh
code2prompt path/to/codebase --tokens
```

Especificar un tokenizador para el conteo de tokens:

```sh
code2prompt path/to/codebase --tokens --encoding=p50k
```

Tokenizadores soportados: `cl100k`, `p50k`, `p50k_edit`, `r50k_bas`.
> [!NOTE]  
> Ver [Tokenizadores](#tokenizers) para más detalles.

Guardar el prompt generado en un archivo de salida:

```sh
code2prompt path/to/codebase --output=output.txt
```

Imprimir salida como JSON:

```sh
code2prompt path/to/codebase --json
```

La salida JSON tendrá la siguiente estructura:

```json
{
  "prompt": "<Generated Prompt>", 
  "directory_name": "codebase",
  "token_count": 1234,
  "model_info": "Modelos de ChatGPT, text-embedding-ada-002",
  "files": []
}
```

Generar un mensaje de commit de Git (para archivos en estado staged):

```sh
code2prompt path/to/codebase --diff -t templates/write-git-commit.hbs
```

Generar una Pull Request comparando ramas (para archivos en estado staged):

```sh
code2prompt path/to/codebase --git-diff-branch 'main, development' --git-log-branch 'main, development' -t templates/write-github-pull-request.hbs
```

Añadir números de línea a los bloques de código fuente:

```sh
code2prompt path/to/codebase --line-number
```

Desactivar el envoltorio de código dentro de bloques de código markdown:

```sh
code2prompt path/to/codebase --no-codeblock
```

- Reescribir el código a otro idioma.
- Encontrar errores/vulnerabilidades de seguridad.
- Documentar el código.
- Implementar nuevas características.

> Inicialmente escribí esto para uso personal para utilizar la ventana de contexto de 200K de Claude 3.0 y ha resultado ser bastante útil, ¡así que decidí hacerlo de código abierto!

## Plantillas

`code2prompt` viene con un conjunto de plantillas integradas para casos de uso comunes. Puedes encontrarlas en el directorio [`templates`](templates).

### [`document-the-code.hbs`](templates/document-the-code.hbs)

Usa esta plantilla para generar prompts para documentar el código. Añadirá comentarios de documentación a todas las funciones, métodos, clases y módulos públicos en la base de código.

### [`find-security-vulnerabilities.hbs`](templates/find-security-vulnerabilities.hbs)

Usa esta plantilla para generar prompts para encontrar posibles vulnerabilidades de seguridad en la base de código. Buscará problemas de seguridad comunes y proporcionará recomendaciones sobre cómo solucionarlos o mitigarlos.

### [`clean-up-code.hbs`](templates/clean-up-code.hbs)

Usa esta plantilla para generar prompts para limpiar y mejorar la calidad del código. Buscará oportunidades para mejorar la legibilidad, adherencia a las mejores prácticas, eficiencia, manejo de errores, y más.

### [`fix-bugs.hbs`](templates/fix-bugs.hbs)

Usa esta plantilla para generar prompts para corregir errores en la base de código. Ayudará a diagnosticar problemas, proporcionar sugerencias de corrección y actualizar el código con las correcciones propuestas.

### [`write-github-pull-request.hbs`](templates/write-github-pull-request.hbs)

Usa esta plantilla para crear una descripción de Pull Request de GitHub en markdown comparando el git diff y el git log de dos ramas.

### [`write-github-readme.hbs`](templates/write-github-readme.hbs)

Usa esta plantilla para generar un archivo README de alta calidad para el proyecto, adecuado para alojar en GitHub. Analizará la base de código para entender su propósito y funcionalidad, y generará el contenido del README en formato Markdown.

### [`write-git-commit.hbs`](templates/write-git-commit.hbs)

Usa esta plantilla para generar commits de git a partir de los archivos en estado staged en tu directorio git. Analizará la base de código para entender su propósito y funcionalidad, y generará el contenido del mensaje de commit de git en formato Markdown.

### [`improve-performance.hbs`](templates/improve-performance.hbs)

Usa esta plantilla para generar prompts para mejorar el rendimiento de la base de código. Buscará oportunidades de optimización, proporcionará sugerencias específicas y actualizará el código con los cambios.

Puedes usar estas plantillas pasando el flag `-t` seguido de la ruta al archivo de plantilla. Por ejemplo:

```sh
code2prompt path/to/codebase -t templates/document-the-code.hbs
```

## Variables Definidas por el Usuario

`code2prompt` soporta el uso de variables definidas por el usuario en las plantillas de Handlebars. Cualquier variable en la plantilla que no sea parte del contexto predeterminado (`absolute_code_path`, `source_tree`, `files`) será tratada como una variable definida por el usuario.

Durante la generación del prompt, `code2prompt` solicitará al usuario que ingrese valores para estas variables definidas por el usuario. Esto permite una mayor personalización de los prompts generados basados en la entrada del usuario.

Por ejemplo, si tu plantilla incluye `{{challenge_name}}` y `{{challenge_description}}`, se te pedirá que ingreses valores para estas variables al ejecutar `code2prompt`.

Esta característica permite crear plantillas reutilizables que pueden adaptarse a diferentes escenarios basados en la información proporcionada por el usuario.

## Tokenizadores

La tokenización se implementa usando [`tiktoken-rs`](https://github.com/zurawiki/tiktoken-rs). `tiktoken` soporta estas codificaciones utilizadas por los modelos de OpenAI:

| Nombre de codificación  | Modelos de OpenAI                                                          |
| ----------------------- | ------------------------------------------------------------------------- |
| `cl100k_base`           | Modelos de ChatGPT, `text-embedding-ada-002`                              |
| `p50k_base`             | Modelos de código, `text-davinci-002`, `text-davinci-003`                 |
| `p50k_edit`             | Usar para modelos de edición como `text-davinci-edit-001`, `code-davinci-edit-001` |
| `r50k_base` (o `gpt2`)  | Modelos GPT-3 como `davinci`                                              |
| `o200k_base`            | Modelos GPT-4o                                                            |

Para más contexto sobre los diferentes tokenizadores, ver el [OpenAI Cookbook](https://github.com/openai/openai-cookbook/blob/66b988407d8d13cad5060a881dc8c892141f2d5c/examples/How_to_count_tokens_with_tiktoken.ipynb)

## ¿Cómo es útil?

`code2prompt` facilita la generación de prompts para LLMs desde tu base de código. Recorre el directorio, construye una estructura de árbol y recopila información sobre cada archivo. Puedes personalizar la generación de prompts usando plantillas de Handlebars. El prompt generado se copia automáticamente en tu portapapeles y también se puede guardar en un archivo de salida. `code2prompt` ayuda a agilizar el proceso de creación de prompts para análisis de código, generación y otras tareas.

## Contribución

Formas de contribuir:

- Sugerir una característica
- Reportar un error  
- Arreglar algo y abrir un pull request
- Ayudarme a documentar el código
- Difundir la palabra

## Licencia

Licenciado bajo la Licencia MIT, ver <a href="https://github.com/mufeedvh/code2prompt/blob/master/LICENSE">LICENSE</a> para más información.

## ¿Te gustó el proyecto?

Si te gustó el proyecto y lo encontraste útil, por favor dale una :star: y considera apoyar a los autores!
