---
title: Uso de Code2prompt CLI con SSH
description: Una guía para usar Code2Prompt CLI con SSH para análisis remoto de base de código.
---

## ¿Por qué no funciona?

Cuando intentas ejecutar el CLI de `code2prompt` en un servidor remoto a través de SSH, el comando no puede encontrar el portapapeles. Esto se debe a que el CLI de `code2prompt` utiliza el portapapeles para copiar el mensaje generado, y las sesiones de SSH normalmente no tienen acceso al portapapeles local.

## Solución

Para usar el CLI de `code2prompt` con SSH, puedes redirigir la salida a un archivo en lugar de copiarla al portapapeles. De esta manera, aún puedes generar el mensaje y guardarlo para su uso posterior.

Utiliza la opción `--output-file` para especificar el archivo de salida donde se guardará el mensaje generado. Por ejemplo:

```sh
ssh user@remote-server "code2prompt path/to/codebase -O output.txt"
```

> Esta página ha sido traducida automáticamente para su conveniencia. Consulte la versión en inglés para ver el contenido original.
