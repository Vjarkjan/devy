# Botones y pantalla táctil

`InputPlugin` llama a `scanKeys` una vez por frame y actualiza los recursos antes
de `Update`. `pressed` permanece activo mientras mantengas un botón;
`just_pressed` y `just_released` duran un frame.

```rust
use devy::prelude::*;

fn controles(botones: Res<ButtonInput<DsButton>>, tactil: Res<Touches>) {
    if botones.just_pressed(DsButton::A) {
        // Confirmar una acción una sola vez.
    }
    if botones.pressed(DsButton::Left) {
        // Desplazar mientras el botón siga pulsado.
    }
    if let Some(toque) = tactil.get_pressed(0) {
        let posicion = toque.position();
        // posicion.x: 0..255; posicion.y: 0..191, en la pantalla inferior.
    }
    if let Some(toque) = tactil.get_released(0) {
        let ultima_posicion = toque.position();
        // Al levantar el lápiz se conserva la última posición válida.
    }
}
```

La DS tiene un solo contacto, con identificador `0`. Los `TouchInput` usan
`Entity::PLACEHOLDER` en el campo de ventana: aquí no existe una ventana Bevy.
No se inventan lecturas de presión ni multitáctil.

## Controles según la mano

```rust
use devy::prelude::*;

let mano = Handedness::Left;
assert_eq!(Cluster::Up.button(mano), DsButton::X);
assert_eq!(Shoulder::Primary.button(mano), DsButton::R);
```

`Handedness` describe la mano del lápiz. Con `Right`, el grupo auxiliar es la
cruceta y el hombro principal es L. Con `Left`, el grupo pasa a ABXY y el
hombro principal es R. Este mapeo conserva el modelo de `bevy_nds`; no cambia
por sí solo las acciones de tu aplicación.

## Simular entrada en host

```rust
use devy::prelude::*;

let mut app = App::new();
app.add_plugins(InputPlugin);
app.insert_resource(InputSample {
    held: DsButton::A.mask(),
    touch: Some((120, 80)),
});
app.update();
assert!(app.world().resource::<ButtonInput<DsButton>>().just_pressed(DsButton::A));

app.insert_resource(InputSample::default());
app.update();
assert!(app.world().resource::<ButtonInput<DsButton>>().just_released(DsButton::A));
```

El host conserva la muestra hasta que la reemplazas. La DS la sobrescribe con
la lectura física. Para un sistema en `PreUpdate` que necesite la entrada ya
procesada, ordénalo con `.after(devy::input::InputSystems)`.
