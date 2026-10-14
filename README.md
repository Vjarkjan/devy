# devy · BlocksDS

Un crate Rust para crear aplicaciones Nintendo DS con Bevy ECS 0.16.1 y
BlocksDS. Reúne la capa de plataforma utilizada por **Digraph-dsi**: ejecución
ARM9, entrada, dos pantallas indexadas, reloj y almacenamiento FAT opcional.

Bevy sigue siendo una dependencia. Aquí se mantiene su adaptación a DS, con
una API común y sin depender del workspace `kts-nds` ni de otro repositorio
local. Es una primera versión de desarrollo; consulta
[LIMITATIONS.md](LIMITATIONS.md) antes de integrarla.

## Probar

```bash
cargo test --locked --all-features
cargo run --locked --example hello
cargo run --locked --example drawing
```

El ejemplo `drawing` genera `output/preview/top.ppm` y `bottom.ppm` usando el
mismo framebuffer que la ROM. La ejecución en host no abre una ventana.

Con Rust nightly, `rust-src`, Wonderful Toolchain y BlocksDS instalados:

```bash
bash scripts/nds.sh rom --example hello
bash scripts/nds.sh rom --example drawing
bash scripts/nds.sh rom --example storage --features storage
```

Las ROM se guardan en `output/`. Las rutas por defecto del SDK están bajo
`/opt/wonderful`; se pueden cambiar con `BLOCKSDS` y `WONDERFUL_TOOLCHAIN`.
Las instrucciones completas están en [Compilación](Doc/01-compilacion.md).

## Usarlo como dependencia

```toml
[dependencies]
devy = { path = "../devy" }
```

```rust
use devy::prelude::*;

#[derive(Resource, Default)]
struct Contador(u32);

fn contar(mut contador: ResMut<Contador>) {
    contador.0 += 1;
}

let mut app = App::new();
app.add_plugins(DsCorePlugins)
    .init_resource::<Contador>()
    .add_systems(Update, contar);
app.update();
assert_eq!(app.world().resource::<Contador>().0, 1);
```

Para una ROM se exporta `main` y se entrega la aplicación a `devy::run(app)`.
[hello.rs](examples/hello.rs) muestra el programa completo, válido en host y DS.

## Qué aporta esta rama

- Un único crate de plataforma, con importaciones desde `devy::prelude`.
- Copia de los tramos de filas modificados y actualización de paletas por cambio.
- Presentación después de VBlank, separada de la lógica de `Update`.
- ABI de libnds comprobada por el compilador C del SDK.
- Pruebas en host con botones, lápiz y tiempo inyectables.
- Montaje FAT al solicitar una operación, mediante la feature `storage`.

[Guía de programación](Doc/README.md) · [Límites y estado](LIMITATIONS.md) ·
[Origen del código](Doc/08-origen.md)
