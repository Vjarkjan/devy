# Estado y verificación

Esta versión integra los módulos de ejecución de kts-nds y conserva el editor
Digraph como ejemplo. Los servicios gráficos y de archivos tienen backend
BlocksDS; la ejecución host permite probar lógica y formatos. La API sigue
siendo de desarrollo.

## Resultados del 13 de octubre de 2026

Entorno: Rust estable 1.95.0, nightly 1.100.0 (`420ed2a0c`), libnds 1.8.0,
ndstool 1.23.0 y DeSmuME 0.9.13 en modo DS.

| Comprobación | Resultado |
| --- | --- |
| Pruebas del crate y su integración | 173 correctas |
| Pruebas de conversores y utilidades del workspace | 52 correctas |
| Ejemplos ejecutables de la guía y API | 28 doctests correctos; 8 fragmentos heredados ilustrativos se omiten |
| Ejemplo Digraph | 8 pruebas de análisis y dibujo correctas |
| Compilación sin features predeterminadas | Pruebas de host correctas |
| Clippy de devy, pruebas y ejemplos en host | Sin advertencias |
| Clippy de ejemplos y biblioteca ARM9 | Sin advertencias |
| Rustdoc con advertencias como errores | Correcto |
| Dependencia externa, llamada también `ds` | Derives, ECS y macro OBJ; ejecución host y enlace ARM9 correctos |
| Conversión de assets | OBJ, sprites PNG, fondos PNG, WAV y escenas RON convertidos |
| Editor de escenas | `cargo check --locked` correcto en su workspace independiente |
| ROMs | `hello`, `drawing`, `storage`, `hardware`, `paint` y `digraph` enlazadas y empaquetadas |

En DeSmuME se comprueban el dibujo indexado, la animación, el botón A y el
lápiz. La pantalla superior de `drawing` se compara píxel por píxel con el PPM
producido en host. El ejemplo `hardware` muestra un cubo girando, tiles, texto
y un sprite que sigue al lápiz. Su estado confirma NitroFS y Maxmod disponibles,
una imagen OAM cargada y cinco instancias de escena leídas en una tarea. El RTC
muestra la hora del emulador. `paint` recibe trazos y mantiene visible su consola;
`digraph` muestra el grafo y la interfaz en ambas pantallas.

Esto verifica arranque e interacción gráfica en emulación. La inicialización
de Maxmod y la carga del soundbank no constituyen una prueba de salida audible.
No se han ejecutado pruebas sobre una DS/DSi física, SD/DLDI real, persistencia
tras reiniciar, modo DSi ni una sesión interactiva del editor de escritorio.
No hay una medición comparativa de FPS frente al proyecto de origen.

## Reproducir las comprobaciones

```bash
cargo fmt --all --check
cargo test --locked --workspace --all-features
cargo test --locked --no-default-features
cargo test --locked --example digraph
cargo clippy --locked --all-targets --all-features -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --locked --all-features --no-deps
cargo check --locked --manifest-path tools/scene-editor/Cargo.toml
bash scripts/check-consumer.sh --nds
bash scripts/assets.sh
bash scripts/nds.sh clippy --examples --all-features
bash scripts/nds.sh rom --examples --all-features
DEVY_NITROFS="$PWD/output/nitrofs" bash scripts/nds.sh rom --example hardware --all-features
cargo run --locked --example drawing
```

Los parsers, la geometría, el punto fijo, los gestos y los slots tienen pruebas
puras. Las pruebas de integración ejecutan juntos los plugins con entrada
inyectada y comprueban que host no acceda a MMIO. También rechazan configuraciones
que comparten VRAM y rutas NitroFS inválidas. Las copias indexadas/RGB555 se
comparan con una referencia escalar y guardas de memoria.

La prueba de asignaciones sólo cubre `IndexedFrame` después de crearlo.
En el frame 120 del ejemplo, se transfieren 80 píxeles en 8 filas; un frame sin
cambios no envía píxeles. Esto mide trabajo de transferencia, no velocidad
física ni ausencia de asignaciones en todo el ECS.

## Emulación

```bash
python3 scripts/emulator-smoke.py \
  --emulator /ruta/desmume-cli --xvfb /ruta/Xvfb \
  --examples drawing hello hardware paint digraph
```

Se requiere DeSmuME CLI con SDL2 nativo, Xvfb, xdotool, ImageMagick, compilador
C y Python con Pillow. El script crea su propio servidor X y cierra los procesos
que inicia. El adaptador SDL sólo selecciona presentación por software; no
modifica la ROM ni el núcleo emulado.

Las capturas y los hashes de las ROM se guardan en `output/emulator/`, junto
con un archivo `checks-<ejemplos>.json`. Son artefactos regenerables fuera de
Git. El script comprueba gráficos y entrada; no escribe partidas ni confirma
operaciones FAT.

Para comprobar una ROM construida desde el repositorio independiente, añade
`--examples digraph --digraph-rom ../digraph-ds/output/digraph-ds.nds`.

La procedencia se puede auditar con [source-inventory.json](source-inventory.json).
[LIMITATIONS.md](../LIMITATIONS.md) describe las restricciones de hardware y
los contratos de las configuraciones, sin utilizarlos como lista de pendientes.
