# Dibujar en las dos pantallas

`Frames` contiene `top` y `bottom`, ambos de 256 × 192 píxeles. Cada píxel es
un índice `u8` en una paleta propia de 256 colores. El programa dibuja en RAM;
el presentador convierte o empaqueta los cambios al copiarlos a VRAM.

```rust
use devy::prelude::*;

let mut frame = IndexedFrame::default();
frame.set_color(0, Color::rgb8(16, 24, 40));
frame.set_color(1, Color::rgb8(96, 224, 176));
frame.set_color(2, Color::WHITE);
frame.fill(0);
frame.plot(20, 30, 2);
frame.line((20, 30), (100, 80), 1).unwrap();
frame.fill_rect(Rect::new(40, 100, 32, 16).unwrap(), 1);
```

`plot` ignora puntos fuera de la pantalla. `Rect::new` y `line` devuelven un
error si la geometría queda fuera; no recortan segmentos automáticamente.
`rgb8` reduce cada canal a cinco bits, el formato de color de la DS.

## Imágenes y renderizadores existentes

```rust
use devy::prelude::*;

let mut frame = IndexedFrame::default();
let indices = [1, 2, 1, 2, 1, 2]; // Imagen de 3 × 2, por filas.
frame.blit(Rect::new(10, 12, 3, 2).unwrap(), &indices).unwrap();
assert_eq!(&frame.pixels()[12 * WIDTH + 10..12 * WIDTH + 13], &[1, 2, 1]);
```

`blit` exige exactamente `ancho × alto` índices y valida la longitud antes de
escribir. Para conectar un renderizador como el de Digraph-dsi:

```rust
use devy::prelude::*;

fn renderizar(mut frames: ResMut<Frames>) {
    let salida = frames.bottom.pixels_mut();
    // El renderizador de la aplicación escribe sus 256 × 192 índices aquí.
    salida[80 * WIDTH + 120] = 1;
}
```

`pixels_mut()` marca toda la pantalla como modificada. Para conservar la
actualización parcial usa `plot`, `line`, `fill_rect` o `blit` con las regiones
que realmente cambien. Si borras y vuelves a dibujar una región, ésta cuenta
como modificada aunque termine con sus colores anteriores.

No hay decodificador PNG, JPEG ni fuentes dinámicas. Convierte tus recursos
previamente y carga índices y paleta; `include_bytes!` sirve para recursos
pequeños integrados en ROM.

## Consola y brillo

```rust
use devy::prelude::*;

fn mensaje(mut video: NonSendMut<Video>, mut brillo: ResMut<MasterBright>) {
    video.show_console(true);
    video.write_console(c"Hola desde Bevy\n");
    brillo.bottom = -8;
}

let mut app = App::new();
app.add_plugins(DsPlugins).add_systems(Startup, mensaje);
app.update();
present(&mut app);
```

La consola superior usa el texto del SDK y está oculta inicialmente. Su fondo
puede tapar el bitmap; ocúltala con `show_console(false)` al volver al dibujo.
El texto de ejemplo usa ASCII, acorde con la fuente incluida por libnds.
En host la consola no dibuja ni escribe a stdout.

`MasterBright` admite de -16 a 16. Sólo se envía al hardware cuando cambia.
`Video` es `NonSend`: el acceso físico pertenece al hilo principal.

## Qué se transfiere

- Arriba se expande la paleta a RGB555 y se escriben dos píxeles por palabra
  de 32 bits cuando la alineación lo permite. Los extremos usan 16 bits.
- Abajo se conserva el bitmap de ocho bits. Cada escritura de 16 bits contiene
  dos índices; una región impar incluye el píxel vecino del framebuffer.
- Se conserva un intervalo por fila. Dos cambios alejados en la misma fila
  hacen que también se copie el espacio intermedio.
- Cambiar la paleta superior exige actualizar sus píxeles. Abajo basta con
  copiar las 256 entradas de paleta.

En DS `run` calcula `Update`, espera VBlank y ejecuta `Present`. En pruebas de
host llama a `app.update()` y después a `present(&mut app)`. La primera
presentación llena ambas pantallas; las siguientes pueden no copiar píxeles.
No hay doble buffer de VRAM: una carga grande puede terminar fuera de VBlank.
