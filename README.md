# devy · BlocksDS

Devy reúne en **un crate de ejecución** la integración Rust/Bevy/BlocksDS de
`kts-nds` y su evolución al desarrollar la applicacion [Digraph DS](https://github.com/Vjarkjan/digraph-ds).

Incluye 2D por framebuffer y por capas, sprites OAM, 3D por hardware, touch,
gestos, trazos, FAT/SD, NitroFS, Maxmod, tareas cooperativas y RTC.
Los conversores de assets y el editor de escenas están en `tools/` y se
compilan para el ordenador. `cred` no forma parte de esta integración.

El editor [Digraph DS](https://github.com/Vjarkjan/digraph-ds) tiene su propio
repositorio y consume devy como dependencia. Aquí se conserva una versión de
referencia en `examples/digraph/` para comprobar la integración del motor.
El desarrollo de la aplicación continúa en `digraph-ds`.

## Probar

```bash
cargo test --locked --workspace --all-features
cargo run --locked --example hello
cargo run --locked --example drawing
bash scripts/nds.sh rom --example digraph
```

Con BlocksDS y Wonderful Toolchain instalados, el ejemplo completo prepara
assets, construye una ROM con NitroFS y utiliza el ARM7 de Maxmod:

```bash
bash scripts/assets.sh
DEVY_NITROFS="$PWD/output/nitrofs" bash scripts/nds.sh rom --example hardware
bash scripts/nds.sh rom --example paint
bash scripts/nds.sh rom --example storage --features storage
```

Las ROM quedan en `output/`. Las rutas del SDK se ajustan con `BLOCKSDS` y
`WONDERFUL_TOOLCHAIN`. La ejecución host comprueba lógica sin abrir una ventana;
`drawing` también genera imágenes PPM.

## Usarlo como dependencia

```toml
[dependencies]
devy = { git = "https://github.com/Vjarkjan/devy", branch = "blockds" }
```

```rust
use devy::prelude::*;

let mut app = App::new();
app.add_plugins(DsPlugins); // Entrada, tiempo y renderer indexado.
app.add_systems(Update, |mut frames: ResMut<Frames>| {
    frames.bottom.set_color(1, Color::rgb8(255, 180, 40));
    frames.bottom.line(20, 20, 120, 80, 1).unwrap();
});
app.update();
present(&mut app);
```

Para una ROM, el punto de entrada llama a `devy::run(app)`.
`HardwarePlugins` selecciona la configuración de kts-nds con consolas, gestos,
relojes, NitroFS y tareas. Sobre ella se añaden los plugins de fondos, sprites,
3D o audio que utilice la aplicación. No se combina con `DsPlugins`, porque
ambos grupos configuran el vídeo.

La copia por regiones del renderer indexado, el texto por diferencias, las
display lists, el culling y los cálculos en punto fijo evitan trabajo repetido.
Su efecto depende de la escena; no se presenta una mejora de FPS sin medirla.

[Guía de programación](Doc/README.md) · [Restricciones](LIMITATIONS.md) ·
[Verificación](Doc/09-verificacion.md) · [Procedencia](Doc/08-origen.md)
