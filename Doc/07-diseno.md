# Organización y decisiones de diseño

## Mapa del código

| Archivo | Responsabilidad |
| --- | --- |
| `src/lib.rs` | API pública, reexports de Bevy y grupos de plugins |
| `src/runtime.rs` | Schedule Present, bucle DS, allocator, panic y critical-section |
| `src/input.rs` | Lectura por frame, botones, contactos y controles según la mano |
| `src/time.rs` | Reloj Bevy y conversión del contador |
| `src/video/frame.rs` | Píxeles, paletas, geometría y regiones modificadas |
| `src/video/copy.rs` | Escrituras de 16/32 bits, con pruebas de límites |
| `src/video/mod.rs` | Recursos de vídeo, inicialización y presentación |
| `src/platform.rs` | Límite Rust/SDK; backend inerte en host |
| `src/storage.rs` | Interfaz FAT opcional |
| `native/nds.c` | Tipos, macros y llamadas reales de libnds |
| `build.rs` | Compilación y archivo estático del adaptador C |
| `scripts/nds-linker.sh` | CRT, script de enlace y bibliotecas de BlocksDS |
| `scripts/nds.sh` | Compilación y empaquetado de aplicaciones |

## Un crate de plataforma

El workspace original separa capacidades en muchos `bevy_nds_*`. Aquí se
concentran las que necesita el editor de digráficas y las operaciones comunes
para aplicaciones 2D. Los módulos internos mantienen la separación del código;
las aplicaciones sólo necesitan una dependencia `devy`.

Se conserva Bevy ECS 0.16.1 sin modificar su implementación. Fijar esa versión
permite reutilizar el modelo del proyecto de origen. Actualizar Bevy requiere
revisar API, features `no_std`, tamaño de ROM y pruebas DS conjuntamente.

## Hardware elegido por target

La condición es `target_vendor = "nintendo"`. No se usa `cfg(unix)` para
distinguir host y consola, porque el target DS también declara la familia Unix.
`build.rs` rechaza otros targets Nintendo que no sean el JSON ARM9 suministrado.

El host utiliza los recursos y sistemas reales, pero no enlaza libnds ni accede
a direcciones MMIO. La entrada y el delta se inyectan. Esto permite probar la
lógica sin convertir el backend de host en un emulador de hardware.

## Memoria y vídeo

El reparto conserva el usado por Digraph-dsi:

| Recurso | Ubicación |
| --- | --- |
| Consola superior BG0 | VRAM A, 128 KiB reservados |
| Bitmap superior BG3 RGB555 | VRAM B, 128 KiB reservados |
| Bitmap inferior BG3 indexado | VRAM C; bitmap 256 × 256, 64 KiB |
| Píxeles visibles compartidos | RAM, 48 KiB por pantalla |

Las dos pantallas consumen **98 304 bytes de píxeles en RAM**, más 1 024 bytes
de paletas y 1 536 bytes de intervalos por fila. Se añaden metadatos, allocator
y el estado de Bevy. Este cálculo no representa el consumo total del programa.
Los píxeles se construyen directamente en el heap mediante `Vec` y `Box` para
evitar temporales de 48 KiB en la pila.

Cada fila mantiene mínimo y máximo modificados. Es un coste fijo, sin listas
dinámicas de rectángulos. Cambios distantes en una misma fila pueden copiar
píxeles intermedios; a cambio, no se asigna memoria al dibujar ni al calcular
las regiones de transferencia.

Las referencias a VRAM nunca se exponen como slices mutables. El presentador
usa escrituras volátiles de 16 o 32 bits: la VRAM de DS no admite escrituras
ordinarias de un byte. No se usa DMA ni se presenta doble buffer.

## Secuencia de un frame

```text
App::update()
  First: avanzar Time
  PreStartup/Startup: inicializar, sólo en la primera actualización
  PreUpdate: escanear controles y actualizar Touches
  Update/PostUpdate/Last: lógica de la aplicación y dibujo en RAM
esperar VBlank
Present: transferir regiones, paleta y brillo; registrar FrameStats
```

`Present` está fuera del schedule `Main` de Bevy. Así, una lógica lenta no
consume directamente el inicio del siguiente intervalo de presentación.
Si el propio envío a VRAM tarda demasiado, todavía puede alcanzar la zona
visible. `run` nunca retorna y no procesa `AppExit` como salida al cargador.

## FFI y exclusividad

El compilador C interpreta `PrintConsole`, `touchPosition`, macros de vídeo y
`FILE`. Rust intercambia punteros a buffers acotados y escalares de tamaño
conocido. No se adivina el tamaño de las estructuras del SDK.

`Video` es un recurso no transferible entre hilos. El adaptador C sólo concede
un propietario DS durante toda la ejecución; otro intento de inicialización
falla. No hay `unsafe impl Send/Sync` sobre punteros de pantalla.

La sección crítica guarda y restaura IME, con barreras de compilador. Esto
sirve para el ARM9; no sincroniza memoria compartida con ARM7. El allocator
usa el heap preparado por el CRT de BlocksDS. El panic handler necesita cero
formateo y puede mostrar el error aun antes de inicializar la consola.
