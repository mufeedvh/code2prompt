---
title: Entendiendo patrones Glob
description: Una explicación detallada de los patrones Glob y cómo se utilizan en Code2Prompt.
---

Los patrones Glob son una forma sencilla pero poderosa de coincidir nombres de archivos y rutas utilizando caracteres comodín. Se utilizan comúnmente en interfaces de línea de comandos y lenguajes de programación para especificar conjuntos de nombres de archivos o directorios. A continuación, se presenta un desglose de los patrones Glob más comúnmente utilizados:

## Comodines básicos

- `*`: Coincide con cualquier número de caracteres, incluidos cero caracteres.
  - Ejemplo: `*.txt` coincide con todos los archivos que terminan con `.txt`.

- `?`: Coincide exactamente con un carácter.
  - Ejemplo: `file?.txt` coincide con `file1.txt`, `fileA.txt`, pero no con `file10.txt`.

- `[]`: Coincide con cualquiera de los caracteres incluidos.
  - Ejemplo: `file[1-3].txt` coincide con `file1.txt`, `file2.txt`, `file3.txt`.

- `[!]` o `[^]`: Coincide con cualquier carácter no incluido.
  - Ejemplo: `file[!1-3].txt` coincide con `file4.txt`, `fileA.txt`, pero no con `file1.txt`.

## Patrones avanzados

- `**`: Coincide con cualquier número de directorios y subdirectorios de forma recursiva.
  - Ejemplo: `**/*.txt` coincide con todos los archivos `.txt` en el directorio actual y todos los subdirectorios.

- `{}`: Coincide con cualquiera de los patrones separados por comas incluidos.
  - Ejemplo: `file{1,2,3}.txt` coincide con `file1.txt`, `file2.txt`, `file3.txt`.

## Ejemplos

1. **Coincidir con todos los archivos de texto en un directorio:**

   ```sh
   *.txt
   ```

2. **Coincidir con todos los archivos con un solo dígito antes de la extensión:**

   ```sh
   file?.txt
   ```

3. **Coincidir con archivos con extensiones `.jpg` o `.png`:**

   ```sh
   *.{jpg,png}
   ```

4. **Coincidir con todos los archivos `.txt` en cualquier subdirectorio:**

   ```sh
   **/*.txt
   ```

5. **Coincidir con archivos que comienzan con `a` o `b` y terminan con `.txt`:**

   ```sh
   {a,b}*.txt
   ```

## Casos de uso

- **Herramientas de línea de comandos:** Los patrones Glob se utilizan ampliamente en herramientas de línea de comandos como `ls`, `cp`, `mv` y `rm` para especificar varios archivos o directorios.
- **Lenguajes de programación:** Lenguajes como Python, JavaScript y Ruby admiten patrones Glob para la coincidencia de archivos a través de bibliotecas como `glob` en Python.
- **Sistemas de compilación:** Herramientas como Makefile utilizan patrones Glob para especificar archivos fuente y dependencias.

## Conclusión

Los patrones Glob proporcionan una forma flexible e intuitiva de coincidir nombres de archivos y rutas, lo que los hace invaluables para tareas de scripting, automatización y administración de archivos. Comprender y utilizar estos patrones puede mejorar significativamente su productividad y eficiencia en el manejo de archivos y directorios.

> Esta página ha sido traducida automáticamente para su conveniencia. Consulte la versión en inglés para ver el contenido original.
