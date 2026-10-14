//! Dibujo táctil sobre el canvas de kts-nds, magnificado por hardware.
#![cfg_attr(target_vendor = "nintendo", no_std)]
#![cfg_attr(target_vendor = "nintendo", no_main)]
use devy::prelude::*;
use devy::stroke::{LoopPlugin, StrokePath, rasterize_polyline};
fn setup(mut bgs: ResMut<Backgrounds>, mut commands: Commands) {
    bgs.enable_paint_canvas();
    commands.spawn((
        DsScreen::Top,
        TilePos { x: 1, y: 2 },
        DsText("Dibuja en la pantalla tactil".into()),
    ));
}
fn paint(stroke: Res<StrokePath>, mut bgs: ResMut<Backgrounds>) {
    if let Some(mut canvas) = bgs.paint() {
        canvas.clear();
        rasterize_polyline(&stroke.points, |x, y| canvas.plot(x / 2, y / 2, 0xffff));
    }
}
fn application() -> App {
    let mut app = App::new();
    app.add_plugins(HardwarePlugins)
        .add_plugins((BackgroundPlugin, LoopPlugin))
        .add_systems(Startup, setup)
        .add_systems(devy::Present, paint);
    app
}
#[cfg(target_vendor = "nintendo")]
#[unsafe(no_mangle)]
pub extern "C" fn main() -> core::ffi::c_int {
    devy::run(application())
}
#[cfg(not(target_vendor = "nintendo"))]
fn main() {
    let mut app = application();
    app.update();
    present(&mut app);
}
