# Tiempo y diagnóstico

`TimePlugin` actualiza `Time<()>` con el contador del SDK. No supone que cada
frame tarda lo mismo: una actualización lenta produce un delta mayor.

```rust
use devy::prelude::*;

#[derive(Resource)]
struct Pulso(Timer);

fn actualizar(tiempo: Res<Time>, mut pulso: ResMut<Pulso>) {
    if pulso.0.tick(tiempo.delta()).just_finished() {
        // Ejecutar una acción aproximadamente cada medio segundo.
    }
}

let mut app = App::new();
app.add_plugins(TimePlugin)
    .insert_resource(Pulso(Timer::from_seconds(0.5, TimerMode::Repeating)))
    .add_systems(Update, actualizar);
app.update();
```

Para movimiento continuo multiplica la velocidad por `tiempo.delta_secs()`.
En DS no hay FPU; para muchos objetos conviene usar enteros o punto fijo en
la lógica de la aplicación. Devy no impone una representación numérica.

## Pruebas reproducibles

```rust
use devy::prelude::*;
use core::time::Duration;

let mut app = App::new();
app.add_plugins(TimePlugin)
    .insert_resource(FrameDelta(Duration::from_millis(20)));
for _ in 0..50 { app.update(); }
assert_eq!(app.world().resource::<Time>().elapsed(), Duration::from_secs(1));
```

`FrameDelta` sólo controla el host. La primera lectura DS establece el origen
y entrega delta cero. El plugin no instala `Time<Virtual>`, `Time<Fixed>` ni
un bucle de `FixedUpdate`; si necesitas pasos fijos, añade un acumulador propio.
La fecha de calendario corresponde a `RtcPlugin` y `WallClock`, explicados
en [relojes y matemáticas](14-matematicas-relojes.md).

## Coste de presentación

Consulta `FrameStats` después de `present`, o en el siguiente `Update`:

| Campo | Significado |
| --- | --- |
| `frames` | Presentaciones ejecutadas |
| `pixels_copied` | Píxeles transferidos en la última presentación |
| `rows_copied` | Filas transferidas, sumando ambas pantallas |
| `palette_entries` | Entradas de paleta inferior transferidas |
| `upload_time` | Duración de la presentación en DS; `None` en host |
| `vblank_overruns` | Presentaciones con píxeles que terminaron en zona visible |

La medición no incluye los sistemas de `Update` ni el tiempo esperando VBlank.
El contador de overruns es una señal de diagnóstico, no una prueba exhaustiva
de ausencia de tearing: sólo inspecciona la línea de barrido al terminar.
