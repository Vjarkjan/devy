# Texto, fondos, sprites y canvas

El renderer por capas conserva la organización de kts-nds. Se inicia con
`HardwarePlugins`; después se añaden los plugins gráficos que se utilicen.
El renderer indexado de [pantallas](04-pantallas.md) es otra configuración.

## Texto y fondos

`TilePos` usa celdas de 8 × 8 píxeles. `DsText` compone texto y `Glyph` coloca
un carácter. El renderer compara el resultado con el frame anterior antes de
escribir la consola. `\n` pasa a la siguiente fila; no hay ajuste automático
de líneas ni composición tipográfica Unicode.

```rust
use devy::prelude::*;

fn preparar(mut commands: Commands, mut fondos: ResMut<Backgrounds>) {
    commands.spawn((DsScreen::Bottom, TilePos { x: 1, y: 2 },
                    DsText("Inventario\nPulsa A".into())));
    fondos.set_tile(DsScreen::Bottom, b"nitro:/backgrounds/tiled/grid.bg\0");
    fondos.set_bitmap(DsScreen::Top, b"nitro:/backgrounds/bitmap/sunset.bbg\0");
}
let mut app = App::new();
app.add_plugins(HardwarePlugins).add_plugins(BackgroundPlugin)
    .add_systems(Startup, preparar);
```

Los setters guardan una petición. `PreUpdate` carga el asset y configura la
capa; `Present` aplica el scroll. `set_tile_scroll` mueve el fondo y
`clear_tile`/`clear_bitmap` ocultan las capas. Las rutas son cadenas C terminadas
en cero; los archivos deben estar dentro del NitroFS empaquetado.

## Sprites OAM

```rust
use devy::prelude::*;

fn jugador(mut commands: Commands) {
    commands.spawn(Sprite::new(b"nitro:/sprites/player.sprite\0").at(100, 120));
}
fn mover(touches: Res<Touches>, mut sprites: Query<&mut Sprite>) {
    if let Some(touch) = touches.iter().next() {
        for mut sprite in &mut sprites {
            sprite.x = touch.position().x as i16;
            sprite.y = touch.position().y as i16;
        }
    }
}
let mut app = App::new();
app.add_plugins(HardwarePlugins).add_plugins(SpritePlugin)
    .add_systems(Startup, jugador).add_systems(Update, mover);
```

`SpriteAssets` carga cada imagen y conserva sus gráficos y paleta. OAM permite
mover instancias sin redibujar sus píxeles en RAM. El backend usa el motor
secundario, que normalmente aparece en la pantalla inferior. Al intercambiar
los LCDs con `Display3d`, sus sprites se desplazan con ese motor.

## Canvas táctil

`enable_paint_canvas()` solicita un bitmap RGB555 de 128 × 128 ampliado al doble
por hardware. Se ven las primeras 96 filas. El fondo de tiles inferior y este
canvas ocupan la misma región, por lo que no se pueden activar juntos.

```rust
use devy::prelude::*;

fn iniciar(mut fondos: ResMut<Backgrounds>) { fondos.enable_paint_canvas(); }
fn dibujar(touches: Res<Touches>, mut fondos: ResMut<Backgrounds>) {
    if let (Some(touch), Some(mut canvas)) = (touches.iter().next(), fondos.paint()) {
        canvas.plot(touch.position().x as i32 / 2,
                    touch.position().y as i32 / 2, 0xffff);
    }
}
let mut app = App::new();
app.add_plugins(HardwarePlugins).add_plugins(BackgroundPlugin)
    .add_systems(Startup, iniciar).add_systems(Present, dibujar);
```

El canvas escribe directamente en VRAM. Usa `Present` para sincronizarlo y
consulta `paint()` porque la inicialización se realiza en el siguiente
`PreUpdate`. `blit_coverage` y `blit_coverage_rect` convierten una máscara de
cobertura a colores mediante una paleta. El ejemplo completo está en
[`paint.rs`](../examples/paint.rs).
