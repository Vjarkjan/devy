# Gráficos 3D y escenas

`Ds3dPlugin` utiliza el motor de geometría de DS. Una entidad combina `DsMesh`
y `Transform3d`; la cámara es el recurso `Camera3d`. Se mantienen iluminación,
materiales, cajas de visibilidad, picking táctil, niebla y contornos de kts-nds.

```rust
use devy::prelude::*;
use devy::render3d::prelude::*;

fn preparar(mut commands: Commands) {
    commands.spawn((DsMesh::cube(0.6), Transform3d::default()));
}
fn girar(time: Res<Time>, mut objetos: Query<&mut Transform3d>) {
    for mut objeto in &mut objetos { objeto.rotation.y += time.delta_secs(); }
}
let mut app = App::new();
app.add_plugins(HardwarePlugins).add_plugins(Ds3dPlugin)
    .add_systems(Startup, preparar).add_systems(Update, girar);
```

La cámara inicial mira al origen desde `(0, 0, 3)`. `Camera3d` permite ajustar
posición, orientación, campo de visión y planos de recorte. `Hidden` excluye
una entidad del dibujo. `RenderRange` limita la distancia visible;
`DsLights`, `DsMaterial`, `DsRenderStyle` y `Stylized` controlan la apariencia.
`TouchPick` expone la selección calculada por la pasada de picking.

## Modelos OBJ

```rust
use devy::render3d::DsMesh;
let modelo: DsMesh = devy::include_obj!("assets/cube.obj", center);
```

La macro genera una display list durante la compilación. La ruta parte del
`Cargo.toml` de la aplicación; admite `center`, `offset = [x, y, z]` y
`compress`. La variante comprimida reduce el tamaño con menor precisión.
Para cargar desde ROM, convierte con `obj2dl` y usa
`DsMesh::load(b"nitro:/cube.dl\0")`, que devuelve `Option<DsMesh>`.

`Display3d.screen` selecciona el LCD del motor principal. El cambio intercambia
también las capas 2D entre pantallas. BG0 principal pertenece al 3D mientras
esté activo; el texto de esa consola no se superpone al modelo.

## Escenas

`scene2bin` convierte niveles y prefabs RON a `.scene`. La escena guarda
instancias, transformaciones, materiales, roles, rutas y conexiones. La carga
no decide la lógica del juego: ésta interpreta `SceneInstance.role`.

```rust
use devy::prelude::*;
use devy::scene;

fn cargar(mut commands: Commands) {
    let ruta = scene::level_space_path("facility", "atrium");
    if let Some(datos) = scene::load(&ruta) { scene::spawn(&mut commands, datos); }
}
```

`ScenePlugin` también atiende eventos `LoadSpace` para cambiar de espacio.
Los modelos referidos se resuelven como `nitro:/<nombre>.dl`. Revisa los
resultados de carga: una ruta inválida devuelve `None`, y una instancia cuyo
modelo no se encuentra conserva sus datos sin malla visible.
