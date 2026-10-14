# Leer y guardar en FAT

Activa el módulo en la aplicación:

```toml
devy = { path = "../devy", features = ["storage"] }
```

`StoragePlugin` crea el recurso, pero no monta la tarjeta. La primera operación
llama a `fatInitDefault`; si falla, una operación posterior puede reintentarlo.
El montaje depende del modo DS/DSi, del cargador y del acceso DLDI o SD disponible.

```rust
use devy::prelude::*;

fn guardar(mut storage: ResMut<Storage>) {
    let resultado = storage.create_dir(c"fat:/mi-juego")
        .and_then(|()| storage.write(c"fat:/mi-juego/partida.bin", &[1, 2, 3, 4]));
    if let Err(error) = resultado {
        // Muestra el error o conserva los datos pendientes en RAM.
    }
}

fn cargar(mut storage: ResMut<Storage>) {
    let mut datos = [0u8; 256];
    match storage.read_into(c"fat:/mi-juego/partida.bin", &mut datos) {
        Ok(longitud) => {
            let contenido = &datos[..longitud];
            // Valida versión, longitud y contenido antes de usarlo.
        }
        Err(error) => {
            // Mantén el estado actual si no se pudo cargar.
        }
    }
}

let mut app = App::new();
app.add_plugins((DsCorePlugins, StoragePlugin));
```

El ejemplo no ejecuta las funciones: conéctalas a una acción explícita del
usuario, como en [`storage.rs`](../examples/storage.rs). En una DSi con acceso
a SD puede corresponder `sd:/` en lugar de `fat:/`; la biblioteca no cambia de
unidad silenciosamente.

Los caminos son `&CStr`, por eso los literales llevan prefijo `c`. Para nombres
dinámicos usa `alloc::ffi::CString` y maneja el error de bytes NUL interiores.
La política de nombres, directorios y formatos pertenece a tu aplicación.

| Error | Causa |
| --- | --- |
| `Unavailable` | No se pudo montar FAT |
| `Io` | Error de apertura, lectura, escritura, cierre o directorio |
| `BufferTooSmall` | El archivo completo no cabe en el buffer proporcionado |
| `Unsupported` | Se llamó al backend FAT desde el host |

`read_into` comprueba el tamaño antes de llenar el buffer. Si falla a mitad de
una lectura por I/O, parte del buffer puede haber cambiado; sólo úsalo cuando
el resultado sea `Ok`. No se asigna un `Vec` del tamaño del archivo en Rust,
aunque el SDK puede reservar memoria internamente.

La escritura reemplaza el archivo y comprueba `fflush` y `fclose`. No es una
transacción ni garantiza recuperación ante apagados. Las operaciones son
bloqueantes: evita ejecutarlas cada frame. Para assets de ROM utiliza
[`nitrofs`](12-assets-audio.md); para guardado por slots y tareas, la API
`save` descrita a continuación. La serialización pertenece a la aplicación;
el ejemplo Digraph usa serde y postcard.

## Guardado por slots de kts-nds

`devy::save` conserva la API de slots, disponible también sin la feature
`storage`. Monta FAT al arrancar cuando se añade `SavePlugin`; `StoragePlugin`
continúa ofreciendo montaje perezoso y lectura en un buffer de tamaño elegido.

```rust
use devy::prelude::*;
fn guardar(save: Res<SaveStorage>) {
    if save.status().is_ready() {
        let guardado = save.write("partida1", b"datos");
        let _ = guardado;
    }
}
let mut app = App::new();
app.add_plugins(SavePlugin { base_dir: Some("fat:/mi-juego/".into()) });
```

Los nombres de slot no aceptan rutas ni `..`. `read`, `write`, `exists` y
`delete` actúan sobre `<directorio>/<slot>.sav`. `read_async` y `write_async`
devuelven un `Task`; conserva el handle hasta recoger su resultado. Cambia el
directorio a `sd:/mi-juego/` sólo cuando ese volumen esté disponible en tu
cargador. Una escritura confirmada no garantiza recuperación ante cortes.
