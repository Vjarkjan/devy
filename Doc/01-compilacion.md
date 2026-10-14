# Compilar y crear una aplicación

## Host

Rust estable permite ejecutar pruebas y ejemplos sin BlocksDS. El archivo
`rust-toolchain.toml` selecciona ese canal para el trabajo habitual.

```bash
cargo test --locked --all-features
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo doc --locked --all-features --no-deps
cargo run --locked --example drawing -- output/preview
```

Añade `--offline` si las dependencias del lockfile ya están en la caché.

## Nintendo DS

Necesitas Rust nightly con fuentes de la biblioteca estándar, Wonderful
Toolchain y BlocksDS con libnds y `ndstool`.

```bash
rustup toolchain install nightly --component rust-src
export WONDERFUL_TOOLCHAIN=/opt/wonderful
export BLOCKSDS="$WONDERFUL_TOOLCHAIN/thirdparty/blocksds/core"
bash scripts/nds.sh rom --example hello
```

El script usa `armv5te-nintendo-ds.json`, reconstruye `core` y `alloc`, enlaza
el CRT de BlocksDS y empaqueta el ejecutable ARM9 con `arm7_maxmod.elf` del
SDK. Python 3 sólo se necesita para el empaquetado. No hace falta Just ni Nix.

`build` produce ELF; `rom` produce ELF y NDS; `check` revisa Rust sin completar
el enlace; `clippy` comprueba también la rama de código de Nintendo DS.

```bash
bash scripts/nds.sh check --example drawing
bash scripts/nds.sh clippy --examples --all-features
bash scripts/nds.sh rom --example storage --features storage
```

`NDS_TOOLCHAIN` selecciona otro nightly ya instalado. `DEVY_CC` y `DEVY_AR`
permiten indicar las herramientas ARM con rutas completas. `--target-dir`
se pasa a Cargo y el empaquetador encuentra el ELF en su salida JSON.

## Otro repositorio

En el `Cargo.toml` de tu aplicación:

```toml
[package]
name = "mi-juego"
version = "0.1.0"
edition = "2024"

[dependencies]
devy = { path = "../devy" }

[profile.release]
panic = "abort"
opt-level = "s"
lto = true
codegen-units = 1
```

Los perfiles deben estar en el paquete raíz de la aplicación: Cargo no hereda
los perfiles de una dependencia. Copia la estructura de
[`hello.rs`](../examples/hello.rs) a `src/main.rs` y genera el lockfile:

```bash
cargo generate-lockfile
bash ../devy/scripts/nds.sh rom --manifest-path Cargo.toml --bin mi-juego
```

La ROM queda en `devy/output/mi-juego.nds`. El script admite rutas con espacios.
No basta con pasar el JSON directamente a Cargo: también hacen falta las
opciones de `.cargo/nds.toml` y `scripts/nds-linker.sh`.

## Features

| Feature | Función |
| --- | --- |
| `runtime` | Predeterminada. Activa `allocator` y `panic-handler` en DS |
| `allocator` | Allocator basado en el heap del SDK |
| `panic-handler` | Pantallas rojas y detención ante un panic |
| `storage` | Añade la API FAT acotada y perezosa `StoragePlugin` |

`default-features = false` permite que el ejecutable aporte su allocator y
panic handler. No elimina Bevy ni su uso de `alloc`. La implementación de
`critical-section` sigue siendo de devy y debe existir una sola en la ROM.

El resto de los módulos de kts-nds está disponible sin features adicionales.
Los plugins activan sus sistemas y recursos; añadir la dependencia por sí sola
no inicia audio ni reserva framebuffers. El enlace elimina código sin usar.

Para incluir archivos en la ROM, define `DEVY_NITROFS` con el directorio que
se empaquetará. `DEVY_ARM7` admite una ruta a otro ELF ARM7 compatible con los
servicios que utilices. El valor predeterminado soporta Maxmod.

```bash
bash scripts/assets.sh
DEVY_NITROFS="$PWD/output/nitrofs" bash scripts/nds.sh rom --example hardware
```

El editor visual se compila aparte:

```bash
cargo run --manifest-path tools/scene-editor/Cargo.toml
```

Su workspace independiente mantiene las dependencias de ventana fuera de la
compilación de ROM y de las pruebas normales del motor.
