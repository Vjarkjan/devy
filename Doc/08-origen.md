# Origen y migración

La extracción parte del checkout local `Digraph-dsi` 0.5.10 y de su dependencia
`bevy_nds`, fijada al commit
`c8a157ce660af200f7283ba09c713a9ba91d384e` de
[ChristopherJMiller/kts-nds](https://github.com/ChristopherJMiller/kts-nds/tree/c8a157ce660af200f7283ba09c713a9ba91d384e).

Se consultó ese checkout fijado de Cargo para el modelo de runtime, entrada y
plugins. Del editor proceden el reparto de pantallas, el framebuffer indexado,
la separación por target y la necesidad de montaje FAT perezoso. No se han
copiado sus algoritmos de grafos, interfaz, formatos de proyecto ni recursos.
El repositorio original queda intacto.

Los hashes de los archivos de referencia están en [SOURCES.md](SOURCES.md).
El adaptador C, los buffers con seguimiento de cambios y la interfaz de devy
se han escrito para esta extracción. El crate no depende de rutas a los
proyectos de origen ni de un checkout Git de `kts-nds`.

Los checkouts consultados no incluyen un archivo de licencia para esa capa.
Se conserva su atribución y `publish = false`; falta definir la licencia de
distribución del código derivado antes de publicar el paquete.

## Desde una aplicación con bevy_nds

| Uso anterior | Uso en devy |
| --- | --- |
| `bevy_nds::run(app)` | `devy::run(app)` |
| `bevy_nds::InputPlugin`, `DsButton` | Mismos nombres en `devy` |
| `ButtonInput<DsButton>` y `Touches` | Tipos originales de Bevy 0.16.1 |
| VideoPlugin y MMIO propio | VideoPlugin, Frames y NonSendMut&lt;Video&gt; |
| Frame indexado superior e inferior | `frames.top` y `frames.bottom` |
| Conversión de índices en cada subida | `set_color` y presentación interna |
| Temporizadores privados | `TimePlugin` y `Time<()>` |
| FFI de archivos en Rust | `StoragePlugin` y `Storage`, feature `storage` |

Pasos recomendados para Digraph-dsi:

1. Sustituir la dependencia `bevy_nds` por `devy`. Mantener las dependencias
   Bevy directas en 0.16.1 si el código todavía las importa por su nombre.
2. Adaptar el arranque para añadir `DsPlugins` o sus plugins individuales.
3. Retirar la inicialización propia de BG3/VRAM antes de activar VideoPlugin:
   ambas implementaciones reservarían los mismos bancos.
4. Conectar los renderizadores a `Frames`. `pixels_mut()` permite una primera
   migración con copia completa; después se pueden introducir regiones.
5. Trasladar las paletas mediante `Color` y `set_color`. Mantener la lógica de
   temas, edición, serialización y análisis en la aplicación.
6. Reemplazar el montaje y las operaciones FAT por `Storage` si interesa usar
   el módulo compartido. La política `sd:/`/`fat:/` sigue siendo de la aplicación.
7. Compilar con los scripts nuevos y probar entrada, gráficos y archivos.

No es una sustitución binaria de toda la API de `bevy_nds`. `Consoles`,
`Backgrounds`, tareas cooperativas, texto ECS, gestos, audio y 3D no se exportan.
La migración completa del editor no forma parte de esta extracción.
