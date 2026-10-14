# Punto fijo, relojes y diagnóstico

`Fx32` representa números con doce bits fraccionarios. Las operaciones de
vectores y matrices comparten ese formato con el motor 3D. En DS, división y
raíz utilizan el coprocesador; en host se ejecutan por software.

```rust
use devy::math::{Fx32, FxVec2};
let paso = Fx32::from_f32(0.25);
let velocidad = Fx32::from_int(8);
assert_eq!((paso * velocidad).to_f32(), 2.0);
let p = FxVec2::from_f32(3.0, 4.0);
assert_eq!(p.length().to_f32(), 5.0);
```

El rango y la precisión son finitos. Evita acumular magnitudes cuyo cuadrado
exceda el formato; punto fijo no es aritmética de precisión arbitraria.
`math::radial` selecciona sectores y `math::stick` calcula un joystick táctil
con zona muerta. `culling` prueba cajas contra el volumen visible de la cámara.

## Tiempo y fecha

`Time` mide duraciones entre frames. `WallClock`, actualizado por `RtcPlugin`,
representa la fecha y hora configuradas en la consola. No sustituyas el delta
del juego por el RTC: el usuario puede cambiar la fecha.

```rust
use devy::rtc::WallClock;
let fecha = WallClock::from_unix(946_684_800);
assert_eq!((fecha.year, fecha.month, fecha.day), (2000, 1, 1));
```

`DiagnosticsPlugin` publica `Fps`, una media suavizada, y un registro circular
`PerfBlob` de deltas. Se puede leer desde el gdbstub de un emulador mediante
`tools/perfread`. `FrameStats` pertenece al renderer indexado y mide píxeles,
filas, paleta y tiempo de transferencia. Son métricas distintas: el coste de
una copia no es la tasa de frames del programa completo.
