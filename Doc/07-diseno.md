# Organización y decisiones de diseño

## Un crate de ejecución

Los antiguos `bevy_nds_*` son módulos de `devy`. La aplicación depende de un
solo crate para el motor. Bevy ECS sigue siendo una dependencia en versión
0.16.1; no se ha sustituido su implementación ni se afirma equivalencia con
todos los plugins de Bevy de escritorio.

| Módulo | Responsabilidad |
| --- | --- |
| `runtime`, `platform` | Bucle ARM9, allocator, sección crítica y adaptadores del renderer indexado |
| `input`, `gesture`, `stroke` | Controles, reconocimiento y geometría de trazos |
| `video` | Framebuffers y configuración alternativa por capas |
| `text`, `background`, `sprite` | Consolas ECS, fondos, canvas y OAM |
| `render3d`, `culling`, `math` | Geometría hardware, visibilidad y punto fijo |
| `nitrofs`, `storage`, `save` | Assets de ROM, FAT acotado y guardado por slots |
| `audio`, `scene` | Maxmod y carga de escenas |
| `time`, `rtc`, `diagnostics` | Duraciones, fecha local y medición |

La macro procedural de OBJ requiere un crate separado por las reglas de Rust.
Tanto ella como los conversores de `tools/` se ejecutan en el ordenador. No
son motores duplicados ni dependencias de ejecución adicionales en ARM9.

## Dos configuraciones de vídeo

`DsPlugins` instala el renderer indexado. `HardwarePlugins` conserva la
configuración por capas de kts-nds y sus servicios comunes. Ambos comparten
`DsScreen`, entrada, tiempo y runtime; la configuración de vídeo es exclusiva.
No se pueden instalar los dos grupos en una misma aplicación.

| Región | Renderer indexado | Renderer por capas |
| --- | --- | --- |
| VRAM A | Consola principal | Fuente BG0, tiles/mapa BG1 |
| VRAM B | BG3 RGB555, 256 × 256 | Bitmap BG3 opcional, 256 × 256 |
| VRAM C | BG3 indexado, 256 × 256 | Consola secundaria y tiles BG1 **o** canvas BG2 |
| VRAM I | Disponible para OAM secundario | OAM secundario, 16 KiB |
| BG0 principal | Consola opcional | Consola **o** salida 3D |

Las consolas usan mapa en 44–46 KiB y fuente en 48–64 KiB de A/C.
Los tiles principales ocupan 0–16 KiB y los secundarios 16–32 KiB; sus mapas,
32–34 KiB. El canvas secundario ocupa 0–32 KiB: por eso el crate rechaza su
combinación con tiles secundarios. El bitmap principal empieza en B.

`DsScreen` sigue la asociación habitual principal/superior y
secundario/inferior. Cambiar `Display3d` intercambia los motores completos entre
LCDs; también se desplazan sus capas 2D y su OAM. El touch siempre corresponde
al LCD físico inferior.

Los buffers indexados usan 98 304 bytes de píxeles en RAM, 1 024 de paletas y
1 536 de intervalos por fila, además de metadatos. El texto compara dos grids
32 × 24 y escribe sólo diferencias. El renderer 3D mantiene display lists y
recalcula matrices/cajas cuando cambian las transformaciones. Los assets se
preparan fuera de la consola.

## Secuencia del frame

```text
App::update()
  First: tiempo y RTC
  PreStartup/Startup: inicialización, una sola vez
  PreUpdate: entrada, gestos, trazos y cargas pendientes
  Update/PostUpdate/Last: lógica, composición y envío de geometría 3D
ceder a tareas cooperativas hasta VBlank
Present: framebuffers, texto, OAM, scroll, brillo y sistemas registrados aquí
```

`Present` usa un ejecutor de un solo hilo. La geometría 3D conserva su envío
en `Last`; el motor de geometría sincroniza su intercambio con el hardware.
Las cargas y el trabajo por frame siguen consumiendo tiempo. Mover una función
a una tarea sólo ayuda si cede el control durante su ejecución.

## Hardware y host

El código DS se selecciona con `target_vendor = "nintendo"`. La familia Unix
no sirve para distinguirlo: el target JSON DS también pertenece a esa familia.

En host se ejecutan el ECS, los parsers, las matemáticas y la entrada inyectada.
Los plugins de hardware mantienen recursos sin acceder a MMIO. NitroFS y audio
indican que no están disponibles; las tareas se resuelven sincrónicamente y el
RTC parte del epoch. Este comportamiento permite pruebas deterministas.

El adaptador C conserva los tipos reales del SDK para consolas y entrada.
Los módulos heredados de kts-nds mantienen bindings privados de libnds/Maxmod
y escrituras de registros. Las consolas comparten una declaración opaca de
`PrintConsole` y NitroFS/guardado comparten los bindings de stdio. Los envíos a
VRAM emplean escrituras de 16 o 32 bits.

La sección crítica protege ARM9 mediante IME; no sincroniza el ECS con ARM7.
El singleton de vídeo impide volver a inicializar sus bancos desde otra App.
