# Límites de devy

Devy es una iteración de **kts-nds**, incluida su evolución local como Digraph.
Tiene renderer 2D, gráficos 3D por hardware, touch, almacenamiento FAT/SD,
NitroFS y audio. Estos servicios están implementados; no son funcionalidades
pendientes. Este documento describe las restricciones al utilizarlos.

## Capacidades disponibles

| Área | Implementación |
| --- | --- |
| Bevy | ECS, entidades, componentes, recursos, sistemas, eventos y plugins |
| 2D | Framebuffers indexados, dibujo de líneas, rectángulos y blit; presentación parcial |
| Capas 2D | Texto ECS, fondos de tiles, bitmap RGB555 y canvas táctil |
| Sprites | OAM del motor secundario, posiciones, paletas y carga desde NitroFS |
| 3D | Display lists, OBJ, cámara, transformaciones, iluminación, culling, picking, niebla y contornos |
| Entrada | Botones, touch, controles por mano, gestos y trazos |
| Archivos | Assets en NitroFS; lectura y escritura FAT/SD; guardado por slots |
| Audio | Efectos y muestras musicales en bucle mediante Maxmod en ARM7 |
| Ejecución | Runtime ARM9, tareas cooperativas, tiempo monotónico y RTC |
| Herramientas | Conversores OBJ/PNG/WAV/RON, editor de escenas y lectura de rendimiento |

La [guía](Doc/README.md) muestra las APIs y los ejemplos. El
[estado de verificación](Doc/09-verificacion.md) indica qué se ha probado.

## Hardware y memoria

- El target es ARM9 de Nintendo DS. El ECS y la lógica del juego comparten ese
  procesador. Las tareas cooperativas no añaden núcleos para ejecutar sistemas
  en paralelo; ARM7 atiende los servicios del SDK y el audio.
- La DS dispone de 4 MiB de RAM principal. Código, heap, pilas, ECS y assets
  deben caber juntos. `no_std` no significa ausencia de heap: este crate usa
  `alloc`. Que un modelo compile no garantiza que quepa en la consola.
- Cada LCD tiene 256 × 192 píxeles. Sólo la pantalla inferior tiene touch y
  proporciona un contacto; no permite gestos multitáctiles reales.
- La GPU 3D tiene una canalización fija. No puede ejecutar shaders de wgpu,
  OpenGL moderno o el renderer de Bevy de escritorio. Devy utiliza sus propios
  renderizadores sobre libnds.
- Hay un único motor 3D. `Display3d` permite enviarlo a una u otra pantalla,
  intercambiando los motores completos. No genera dos vistas 3D independientes
  simultáneas con esta configuración.

## Combinaciones de vídeo

`VideoPlugin` reserva A/B/C para el renderer de framebuffers.
`HardwareVideoPlugin` prepara las consolas y deja que los plugins de capas
asignen sus regiones. **No se pueden instalar ambos en la misma aplicación**;
el crate rechaza esa combinación.

| Configuración | Restricción |
| --- | --- |
| `VideoPlugin` | 256 índices por pantalla; la salida superior convierte la paleta a RGB555 |
| `HardwareVideoPlugin` + `BackgroundPlugin` | BG1 usa tiles de 4 bpp; el bitmap RGB555 completo usa BG3 del motor principal |
| Canvas inferior | Comparte los primeros 32 KiB de C con los tiles inferiores: no pueden coexistir |
| Texto + 3D | El 3D ocupa BG0 del motor principal, la misma capa que su consola de texto |
| `SpritePlugin` | Usa OAM secundario y VRAM I; admite hasta 128 sprites y 16 imágenes/paletas, sujetos a los 16 KiB de gráficos disponibles |
| Formato `.sprite` | Imágenes cuadradas de 8, 16, 32 o 64 píxeles, 4 bpp y hasta 16 colores por imagen |

Las regiones asignadas no pueden reutilizarse por MMIO externo sin invalidar
el estado del renderer. Los límites de estas configuraciones no son
prohibiciones universales del hardware: otro reparto de VRAM requiere cambiar
el backend. [El mapa de memoria](Doc/07-diseno.md) detalla las asignaciones.

El framebuffer indexado ocupa 96 KiB de píxeles en RAM, más paletas y seguimiento
de cambios. Una paleta nueva en la pantalla superior obliga a convertirla
completa. VBlank sincroniza `Present`, pero no garantiza que todo su trabajo
termine antes de la siguiente zona visible. No se puede prometer una tasa de
frames fija independientemente de la carga.

## Archivos, audio y ejecución

- NitroFS es de sólo lectura. Para guardar se necesita FAT y una unidad que
  el cargador exponga: DLDI/flashcart o SD en un entorno DSi compatible.
  Una ROM no puede crear acceso a una tarjeta que el cargador no ofrece.
- Los assets deben tener los formatos de los conversores incluidos. Los PNG,
  WAV y OBJ fuente se convierten en el ordenador; no se entregan directamente
  a OAM, a Maxmod o al motor de geometría.
- Maxmod necesita su biblioteca ARM9, un ARM7 compatible y un soundbank válido.
  `Music` reproduce una muestra en bucle; el formato `.wav` fuente no es un
  flujo de audio que se decodifique desde SD durante la reproducción.
- Guardar no es una transacción resistente a cortes de alimentación. Una
  escritura interrumpida puede dejar un archivo incompleto. Las APIs de tareas
  no convierten una operación FAT en una escritura atómica.
- Una tarea que no cede el control puede detener los frames. Destruir un
  `Task` pendiente espera su finalización; no equivale a cancelarlo.
- El RTC refleja la fecha local configurada en la consola. No proporciona
  sincronización de red, zona horaria ni una fuente monotónica; para duraciones
  se usa `Time`.
- `run` es un bucle sin retorno. Un pánico detiene la aplicación. El scheduler
  no convierte `AppExit` en un regreso al cargador.

Las restricciones anteriores son de hardware, formato o contrato de la API.
Las pruebas en host verifican lógica y formatos; no sustituyen la ejecución de
la ROM en un emulador o una consola.
