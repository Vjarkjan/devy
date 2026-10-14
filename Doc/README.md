# Guía de programación

Devy conserva el modelo de Bevy: un `App` contiene entidades, componentes,
recursos y sistemas. BlocksDS aporta el arranque ARM9, los controles y el
acceso al hardware. El crate conecta ambas partes.

Si empiezas con el proyecto, compila `hello`, modifica un sistema y después
prueba `drawing`. Los ejemplos completos viven en `examples/`; los fragmentos
de esta guía explican cada operación por separado.

| Documento | Contenido |
| --- | --- |
| [01 · Compilación](01-compilacion.md) | Entorno, dependencia externa y ROM |
| [02 · ECS y sistemas](02-ecs.md) | Componentes, recursos, consultas, eventos y orden |
| [03 · Entrada](03-entrada.md) | Botones, lápiz, controles zurdos y simulación |
| [04 · Pantallas](04-pantallas.md) | Colores, líneas, imágenes, consola y coste de copia |
| [05 · Tiempo](05-tiempo.md) | Delta, temporizadores y diagnóstico |
| [06 · Almacenamiento](06-almacenamiento.md) | Leer, guardar y manejar errores FAT |
| [07 · Diseño](07-diseno.md) | Módulos, memoria, ABI y decisiones |
| [08 · Origen](08-origen.md) | Fuentes y migración desde Digraph-dsi |
| [09 · Verificación](09-verificacion.md) | Pruebas y alcance de los resultados |

El [estado actual](../LIMITATIONS.md) distingue funciones disponibles,
verificaciones realizadas y trabajo pendiente.
