# Assets, NitroFS y audio

NitroFS guarda archivos de sólo lectura dentro de la ROM. `NitroFsPlugin`
monta el sistema y publica `NitroFs.ready`. `HardwarePlugins` ya lo incluye;
con `DsPlugins` se añade expresamente.

```rust
use devy::nitrofs;
if let Some(bytes) = nitrofs::read_file(b"nitro:/datos.bin\0") {
    // Interpretar bytes según el formato de la aplicación.
    let longitud = bytes.len();
    let _ = longitud;
}
```

`read_file` comprueba la terminación de la ruta y devuelve `None` ante errores.
Lee el archivo completo en RAM; el tamaño de los assets cuenta en el presupuesto
del heap. `flush_dcache` prepara buffers que vaya a leer DMA, como display lists;
los cargadores del motor lo hacen cuando corresponde.

## Conversores

| Fuente | Herramienta | Archivo de ejecución |
| --- | --- | --- |
| OBJ | `obj2dl` / `include_obj!` | `.dl` / datos estáticos |
| PNG de sprite | `png2sprite` | `.sprite` |
| PNG de fondo | `png2bg` | `.bg` o `.bbg` |
| WAV | `wav2bank` y `mmutil` | `soundbank.bin` e IDs Rust |
| Niveles/prefabs RON | `scene2bin` | `.scene` |

`bash scripts/assets.sh` convierte los recursos incluidos a `output/nitrofs/`.
Los comandos individuales sirven para un proyecto propio:

```bash
cargo run -p obj2dl -- --input assets/cube.obj --output output/cube.dl --center
cargo run -p png2sprite -- assets/sprites/player.png output/player.sprite
cargo run -p png2bg -- tiled assets/backgrounds/tiled/grid.png output/grid.bg
cargo run -p wav2bank -- --input audio --output output/soundbank.bin --ids output/sounds.rs
cargo run -p scene2bin -- --check assets/levels
```

`grit` y `mmutil` se buscan en BlocksDS, en `PATH` o mediante `GRIT` y `MMUTIL`.
El archivo debe existir antes de empaquetar la ROM:

```bash
DEVY_NITROFS="$PWD/output/nitrofs" bash scripts/nds.sh rom --example hardware
```

## Audio

`AudioPlugin` inicializa Maxmod después de NitroFS. El empaquetador selecciona
`arm7_maxmod.elf` y el enlace incorpora `mm9`. Comprueba `Audio.ready` antes de
presentar reproducción como disponible.

```rust
use devy::prelude::*;
use devy::audio::{AudioPlugin, Music, PlaySfx, SoundId};

fn iniciar(mut music: ResMut<Music>) {
    music.set_volume(0.5);
    music.play(SoundId(0)); // Sustituir por el ID generado en sounds.rs.
}
fn efecto(buttons: Res<ButtonInput<DsButton>>, mut events: EventWriter<PlaySfx>) {
    if buttons.just_pressed(DsButton::A) {
        events.write(PlaySfx::new(SoundId(1)).with_volume(0.8));
    }
}
let mut app = App::new();
app.add_plugins(HardwarePlugins).add_plugins(AudioPlugin)
    .add_systems(Startup, iniciar).add_systems(Update, efecto);
```

`Music` conserva una muestra en bucle hasta cambiarla o llamar a `stop()`.
`PlaySfx` dispara un efecto con volumen y panorama. Los IDs dependen del banco
generado; no se deben reutilizar números de otro soundbank. Consulta
`output/sounds.rs` tras ejecutar los conversores.
