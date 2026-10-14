# Verificar cambios

Ejecuta desde la raíz:

```bash
cargo fmt --all --check
cargo test --locked --all-features
cargo test --locked --no-default-features
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo doc --locked --all-features --no-deps
bash scripts/nds.sh clippy --examples --all-features
bash scripts/nds.sh rom --examples --all-features
```

Los ejemplos de ECS, entrada, pantallas, tiempo y almacenamiento de `Doc/`
también se ejecutan como doctests. Se compilan con las features que necesiten;
el bloque de almacenamiento requiere `--all-features`.

Las pruebas cubren la secuencia de pulsación y liberación, el último punto del
lápiz, el delta inyectado, límites del framebuffer, efectos de cambios de paleta,
frames sin cambios y escritura empaquetada con guardas alrededor del destino.
La referencia escalar comprueba las copias RGB555 para todas las anchuras de
0 a 256 y ambas alineaciones de halfword.

La prueba de asignaciones sólo cubre operaciones de `IndexedFrame` después de
crearlo. No afirma que todos los sistemas ECS o el SDK estén libres de heap.

## Dependencia externa

```bash
bash scripts/check-consumer.sh
bash scripts/check-consumer.sh --nds
```

El script crea una aplicación temporal cuya única dependencia directa es
`devy`. Comprueba que los derives del prelude, el ECS y el archivo nativo se
resuelven desde otro paquete. La segunda variante también enlaza ARM9. No
modifica los proyectos de origen.

## Pantallas

```bash
cargo run --locked --example drawing
```

Comprueba los PPM en `output/preview/`. Para validar hardware hace falta arrancar
la ROM: el host no reproduce la paleta física, el barrido, las escrituras MMIO
ni la sincronización ARM7/ARM9.

Al probar en consola o emulador, revisa las dos pantallas, el movimiento del
cuadro, el cambio de dirección con A y el seguimiento del lápiz. Para FAT usa
una tarjeta de prueba con el directorio del ejemplo; comprueba lectura después
de reiniciar y los errores de archivo inexistente o buffer insuficiente.

La comprobación gráfica automatizada se ejecuta después de generar las ROM y
la vista previa:

```bash
python3 scripts/emulator-smoke.py --emulator /ruta/desmume-cli --xvfb /ruta/Xvfb
```

Necesita DeSmuME CLI con SDL2, Xvfb, xdotool, ImageMagick, un compilador C y
Python con Pillow. Crea su propio servidor X y termina únicamente los procesos
que inicia. `scripts/emulator_sdl.c` selecciona presentación por software para
Xvfb; no cambia el núcleo emulado ni forma parte de la ROM. Algunas instalaciones
con compatibilidad SDL2 sobre SDL3 no funcionan con este adaptador; usa un build
SDL2 nativo, como el utilizado en la validación.

Compara la pantalla superior con el PPM, comprueba animación y lápiz, y envía A
al ejemplo `hello`. Guarda capturas y hashes de las ROM en
`output/emulator/checks.json`. No comprueba FAT ni mide velocidad física.

Los resultados de la versión entregada se registran en
[LIMITATIONS.md](../LIMITATIONS.md). Una ROM enlazada no demuestra por sí sola
su arranque, el modo DSi, el rendimiento físico o la persistencia de archivos.
