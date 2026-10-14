# Limitaciones y estado actual

Devy 0.1.0 es una extracción funcional de la capa Bevy/BlocksDS usada por
Digraph-dsi. Reúne esa integración en un crate; Bevy ECS 0.16.1 continúa siendo
una dependencia externa. La API todavía puede cambiar.

## Disponible

| Parte | Estado |
| --- | --- |
| ECS | App, entidades, componentes, recursos, sistemas y eventos de Bevy |
| Runtime ARM9 | Allocator del SDK, panic handler, critical-section y espera VBlank |
| Entrada | Doce botones, lápiz, transiciones y mapeo según la mano |
| Vídeo | Dos buffers 256 × 192 indexados, paletas, líneas, rectángulos y blit |
| Presentación | Intervalos por fila, RGB555 arriba e índices empaquetados abajo |
| Texto | Consola superior de libnds; oculta inicialmente |
| Tiempo | Time&lt;()&gt; con contador del SDK y delta inyectable en host |
| Diagnóstico | Regiones copiadas, tiempo de subida DS y final fuera de VBlank |
| FAT | Feature opcional; montaje perezoso, lectura acotada y escritura bloqueante |
| Host | Pruebas sin hardware y generación de PPM |

## Límites de diseño

- Sólo se implementa el backend **ARM9 de Nintendo DS**. El ARM7 lo aporta el
  ejecutable estándar de BlocksDS. No hay backend de ventana, Android ni Web.
- No se incluye el renderer de Bevy, wgpu, assets de Bevy, audio, 3D, sprites
  OAM, fondos por tiles, gestos, tareas cooperativas, NitroFS ni RTC.
- Se usa `no_std` con **alloc**. Bevy y los framebuffers requieren heap; no es
  una biblioteca sin asignaciones. El framebuffer ocupa 96 KiB de píxeles,
  más paletas, regiones, metadatos y el estado ECS.
- La pantalla superior reserva VRAM A para consola y B para bitmap. La inferior
  reserva C. Una aplicación no debe reconfigurar esos bancos mientras use
  VideoPlugin. Sólo se admite un propietario físico durante la ejecución.
- Cada pantalla permite 256 colores elegidos por su propia paleta. La salida
  superior usa RGB555, pero la API de dibujo sigue siendo indexada.
- Un intervalo por fila puede incluir píxeles sin cambios. Cambiar la paleta
  superior o usar `pixels_mut()` provoca una subida completa de esa pantalla.
- VBlank sincroniza el inicio de Present. No hay doble buffer, DMA ni límite
  automático de trabajo: una copia grande puede terminar en la zona visible.
- `TimePlugin` no instala tiempo virtual ni pasos fijos. El contador del SDK
  reserva el timer de sistema; no lo reutilices en otro subsistema.
- `run` no retorna ni gestiona AppExit, suspensión, reanudación o salida al
  cargador. El pánico muestra rojo y detiene la aplicación, sin mensaje ni log.
- FAT depende del cargador y la unidad disponible. La escritura no es atómica,
  el I/O bloquea y el SDK puede reservar memoria. En host devuelve Unsupported.
- El crate no es compatible con todos los plugins de `bevy_nds`. La migración
  de Digraph-dsi requiere adaptar su vídeo y retirar el MMIO que ahora posee devy.

## Optimización y evidencia

Se evita copiar píxeles en frames sin cambios. Las subidas superiores combinan
dos colores por escritura alineada de 32 bits; abajo se mantienen las
escrituras de 16 bits requeridas por VRAM. Los intervalos usan memoria fija y
el almacenamiento sólo se monta cuando se utiliza.

Estas mejoras reducen trabajo concreto, pero **no hay una comparación de FPS
o latencia contra Digraph-dsi en una consola física**. No se atribuyen a devy
mediciones de otras aplicaciones. El coste depende del ECS, de la lógica del
programa y del área modificada.

## Verificación de esta versión

Comprobado el **13 de octubre de 2026** con Rust estable 1.95.0, nightly
1.100.0 (`420ed2a0c`, 2026-09-18), libnds 1.8.0 y ndstool 1.23.0 del SDK local.
El target JSON y las opciones inestables pueden necesitar revisión con otro
nightly; `NDS_TOOLCHAIN` permite seleccionar el que vayas a utilizar.

| Comprobación | Resultado |
| --- | --- |
| `cargo test --locked --offline --all-features` | 12 pruebas de código y 14 doctests correctos |
| `cargo test --locked --offline --no-default-features` | 11 pruebas de código y 13 doctests correctos |
| Formato y Clippy de host | Sin errores ni advertencias |
| Rustdoc con `-D warnings` | Documentación generada correctamente |
| Clippy ARM9, todos los ejemplos y features | Correcto |
| ROMs `hello`, `drawing`, `storage` | Compiladas y empaquetadas |
| Consumidor con sólo `devy` como dependencia directa | Ejecución host y enlace ARM9 correctos |
| DeSmuME 0.9.13, modo DS | Arranque de hello/drawing, botón A y lápiz comprobados |
| Pantalla superior de drawing | Coincide píxel por píxel con el PPM del host |
| Animación inferior | Movimiento observado y respuesta táctil comprobados |

En el frame 120 de `drawing`, el host registra **80 píxeles en 8 filas** en
lugar de una subida completa de 98 304 píxeles entre las dos pantallas. Es una
medición del trabajo de transferencia de ese ejemplo, no de FPS. Las pruebas
también verifican cero píxeles transferidos en un frame sin cambios.

Tamaños de las ROM generadas con el perfil release y todas las features:
`hello.nds`: 368 128 bytes; `drawing.nds`: 370 176 bytes;
`storage.nds`: 417 792 bytes. Las capturas y el resultado automatizado quedan
en `output/emulator/`, fuera de Git y regenerables con el script de verificación.

No se ha probado una DS/DSi física, el modo DSi de un cargador, suspensión ni
lectura/escritura FAT en tarjeta. La licencia de distribución del código
derivado está pendiente; Cargo mantiene `publish = false`.

## Ejemplos

```bash
cargo run --locked --example hello
cargo run --locked --example drawing -- output/preview
bash scripts/nds.sh rom --example hello
bash scripts/nds.sh rom --example drawing
bash scripts/nds.sh rom --example storage --features storage
```

`hello` muestra texto y cambia el fondo con A. `drawing` dibuja una gráfica
superior y un cuadro móvil inferior, controlable con el lápiz. `storage`
guarda con A y lee con B en `fat:/devy/demo.bin`.

La [guía en Doc/](Doc/README.md) explica los módulos, los patrones de uso y las
decisiones de diseño. Los comandos de comprobación están en
[Doc/09-verificacion.md](Doc/09-verificacion.md).
