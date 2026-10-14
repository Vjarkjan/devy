# Entidades, recursos y sistemas

Usa un componente para datos que pertenecen a cada entidad; un recurso para
estado único de la aplicación. Un sistema es una función cuyos parámetros
declaran los datos que necesita.

```rust
use devy::prelude::*;

#[derive(Component)]
struct Posicion { x: i32, y: i32 }

#[derive(Component)]
struct Velocidad(i32);

#[derive(Resource, Default)]
struct Pausa(bool);

fn crear(mut commands: Commands) {
    commands.spawn((Posicion { x: 12, y: 80 }, Velocidad(1)));
}

fn mover(pausa: Res<Pausa>, mut objetos: Query<(&mut Posicion, &Velocidad)>) {
    if pausa.0 { return; }
    for (mut posicion, velocidad) in &mut objetos {
        posicion.x = (posicion.x + velocidad.0).rem_euclid(256);
    }
}

let mut app = App::new();
app.add_plugins(DsCorePlugins)
    .init_resource::<Pausa>()
    .add_systems(Startup, crear)
    .add_systems(Update, mover);
app.update();
```

`Res<T>` permite leer un recurso; `ResMut<T>` permite modificarlo. `Query`
selecciona entidades por sus componentes. `Commands` deja cambios estructurales
pendientes, como crear o eliminar entidades. Bevy los aplica en sus puntos de
sincronización. Para conservar la entidad usa `commands.spawn(...).id()`.

`use devy::prelude::*` importa también el nombre `bevy_ecs` que requieren los
derives de Bevy 0.16. Si eliges importaciones explícitas, añade
`use devy::bevy_ecs;` en cada módulo que declare derives ECS.

## Orden

`Startup` se ejecuta una vez. La entrada se actualiza en `PreUpdate`; la lógica
habitual va en `Update`. La presentación de devy usa el schedule `Present`.

```rust
use devy::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum Fase { Editar, Analizar, Dibujar }

fn editar() {}
fn analizar() {}
fn dibujar() {}

let mut app = App::new();
app.configure_sets(Update, (Fase::Editar, Fase::Analizar, Fase::Dibujar).chain())
    .add_systems(Update, editar.in_set(Fase::Editar))
    .add_systems(Update, analizar.in_set(Fase::Analizar))
    .add_systems(Update, dibujar.in_set(Fase::Dibujar));
app.update();
```

El orden de registro de sistemas independientes no establece una dependencia.
Usa `.chain()`, `.before()` o `.after()` cuando uno consuma el resultado de otro.

## Eventos

```rust
use devy::prelude::*;

#[derive(Event)]
struct Guardar;

fn solicitar(botones: Res<ButtonInput<DsButton>>, mut eventos: EventWriter<Guardar>) {
    if botones.just_pressed(DsButton::Start) { eventos.write(Guardar); }
}

fn guardar(mut eventos: EventReader<Guardar>) {
    for _ in eventos.read() {
        // Serializa aquí el estado de tu aplicación.
    }
}

let mut app = App::new();
app.add_plugins(DsCorePlugins).add_event::<Guardar>()
    .add_systems(Update, (solicitar, guardar).chain());
app.update();
```

Los eventos de esta versión son los de Bevy **0.16.1**. Evita mezclar ejemplos
de otra versión con los nombres de esta API.

## Elegir plugins

`DsPlugins` instala entrada, reloj y vídeo. `DsCorePlugins` instala entrada y
reloj, sin los 96 KiB de píxeles de las dos pantallas. También puedes añadir
`InputPlugin`, `TimePlugin` y `VideoPlugin` por separado. Añade cada uno una vez.

Para encapsular una funcionalidad propia, implementa `Plugin::build` y registra
allí sus recursos y sistemas. El código de la aplicación puede seguir siendo
independiente del hardware mientras trabaje con estos recursos.
