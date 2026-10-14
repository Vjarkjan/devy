# Procedencia y migración desde kts-nds

La fuente de esta iteración es el **árbol de trabajo local**
`/home/yfjor/prgm/kts-nds`, cuya base Git es
`c8a157ce660af200f7283ba09c713a9ba91d384e` de
[ChristopherJMiller/kts-nds](https://github.com/ChristopherJMiller/kts-nds).
Ese directorio contiene modificaciones locales de Digraph Explorer 0.5.10;
no se ha supuesto que el commit represente por sí solo todo su contenido.

Los módulos del motor se integran en `src/`. El editor de digráficas activo,
con su análisis, interfaz, renderer y persistencia, queda como aplicación
compilable en `examples/digraph/`. Los assets de referencia y las herramientas
también se conservan. El juego y el editor no se convierten en dependencias
obligatorias de cada aplicación que use el crate.

La aplicación independiente se mantiene en
[Vjarkjan/digraph-ds](https://github.com/Vjarkjan/digraph-ds) y fija una revisión
de devy como dependencia. El ejemplo de este repositorio conserva la base de
referencia de la integración.

La primera extracción del renderer indexado se basó en `Digraph-dsi`.
Ahora convive con las implementaciones por capas del árbol `kts-nds`. `cred`
queda fuera, conforme al alcance solicitado. Ningún proyecto de origen se
modifica ni se necesita como dependencia local para compilar devy.

El [inventario](source-inventory.json) registra rutas de origen, SHA-256 y
destinos. [SOURCES.md](SOURCES.md) describe las adaptaciones y conserva las
referencias de la extracción inicial. La licencia de distribución requiere
resolverse antes de publicar; los paquetes derivados mantienen `publish = false`.

## Correspondencia de módulos

| Crate anterior | Ruta actual |
| --- | --- |
| `bevy_nds_runtime`, `bevy_nds_input`, `bevy_nds_time` | `devy::runtime`, `input`, `time` |
| `bevy_nds_video` | `devy::video::HardwareVideoPlugin` y tipos compartidos |
| `bevy_nds_gesture`, `bevy_nds_loop` | `devy::gesture`, `devy::stroke` |
| `bevy_nds_text`, `bevy_nds_bg`, `bevy_nds_sprite` | `devy::text`, `background`, `sprite` |
| `bevy_nds_3d`, `bevy_nds_3d_cull`, `bevy_nds_math` | `devy::render3d`, `culling`, `math` |
| `bevy_nds_audio`, `bevy_nds_scene` | `devy::audio`, `scene` |
| `bevy_nds_nitrofs`, `bevy_nds_save` | `devy::nitrofs`, `save` |
| `bevy_nds_cothread`, `bevy_nds_rtc`, `bevy_nds_diagnostics` | `devy::cothread`, `rtc`, `diagnostics` |
| `bevy_nds_3d_macros`, `bevy_nds_3d_obj` | Soporte de `devy::include_obj!` en `tools/` |

Para conservar una aplicación por capas, cambia los imports y usa
`HardwarePlugins`. Añade `SavePlugin` si quieres montar el almacenamiento al
arrancar; no se incluye automáticamente en ese grupo. Para conservar MMIO
propio como el ejemplo Digraph, instala únicamente `HardwareVideoPlugin` y
los servicios necesarios, sin el renderer indexado ni plugins que reutilicen
sus regiones.

Para adoptar la presentación parcial, usa `DsPlugins`, retira la inicialización
MMIO propia de pantallas y dibuja sobre `Frames`. `pixels_mut()` permite una
primera adaptación con copia completa; `plot`, `line`, `fill_rect` y `blit`
permiten después limitar la transferencia a las filas modificadas.
