# Gestos, trazos y tareas cooperativas

`GesturePlugin` transforma el touch en taps, pulsaciones largas, deslizamientos
y arrastres. Requiere entrada y tiempo; `HardwarePlugins` ya instala los tres.
`Gestures` permite consultar el frame y `GestureEvent` sirve para reaccionar
una vez a cada reconocimiento.

```rust
use devy::prelude::*;
fn gestos(mut events: EventReader<GestureEvent>, mut commands: Commands) {
    for GestureEvent(gesture) in events.read() {
        if let Gesture::Tap(pos) = gesture {
            commands.spawn((DsScreen::Bottom,
                TilePos { x: pos.x as i16 / 8, y: pos.y as i16 / 8 }, Glyph(b'+')));
        }
    }
}
```

Con `DsPlugins`, añade `GesturePlugin` expresamente. Los gestos siempre se
calculan después de actualizar `Touches`.

## Trazos

`stroke::LoopPlugin` acumula puntos en `StrokePath` y emite `StrokeCompleted`
al levantar el lápiz. El consumidor decide qué significa el trazo.

```rust
use devy::prelude::*;
use devy::stroke::{self, StrokeCompleted};
fn cerrar(mut events: EventReader<StrokeCompleted>) {
    for StrokeCompleted(points) in events.read() {
        if let Some(poligono) = stroke::find_closed_loop(points) {
            let area = stroke::area(&poligono);
            let _ = area;
        }
    }
}
```

El módulo incluye suavizado, cierre por cruce o proximidad, pruebas de inclusión,
área, perímetro y rasterización normal, gruesa, antialias y con halo. Son
funciones independientes del hardware; se pueden usar sobre `IndexedFrame` o
sobre el canvas de [capas 2D](10-capas-2d.md).

## Tareas

```rust
use devy::cothread::{spawn, yield_now};
let mut tarea = spawn(|| {
    let mut total = 0u32;
    for bloque in 0..20 { total += bloque; yield_now(); }
    total
});
if let Some(resultado) = tarea.poll() { assert_eq!(resultado, 190); }
```

En DS, `run` cede el control al scheduler mientras espera VBlank. Conserva el
`Task` en un recurso entre frames y llama a `poll()` para recoger el resultado
una sola vez. La clausura y su resultado deben ser `Send + 'static`.

`yield_now` permite continuar otra tarea; `yield_until_vblank` espera el siguiente
VBlank. No hay preempción: divide el trabajo largo en bloques. Destruir el handle
antes de terminar espera a la tarea, por lo que un `Task` local descartado al
final de un sistema puede bloquear ese frame. En host la clausura se ejecuta
sincrónicamente para comprobar la lógica.

El guardado por slots ofrece `read_async` y `write_async` sobre estas tareas.
No hay garantía de cancelación, plazo máximo ni transacción de disco.

Cada tarea reserva 16 KiB de pila por defecto. `spawn_with_stack` permite
ajustarla (mínimo 1 KiB, múltiplo de 8); las llamadas de archivos necesitan
espacio para la pila de libc y del driver, además de la clausura.
