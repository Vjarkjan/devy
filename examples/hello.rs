#![cfg_attr(target_vendor = "nintendo", no_std)]
#![cfg_attr(target_vendor = "nintendo", no_main)]

use devy::prelude::*;

#[derive(Resource, Default)]
struct Clicks(u32);

fn setup(mut frames: ResMut<Frames>, mut video: NonSendMut<Video>) {
    frames.top.set_color(0, Color::rgb8(16, 32, 56));
    frames.bottom.set_color(0, Color::rgb8(24, 48, 80));
    video.show_console(true);
    video.write_console(c"Devy / BlocksDS\n\nPulsa A para cambiar el fondo.\n");
}

fn press(
    buttons: Res<ButtonInput<DsButton>>,
    mut clicks: ResMut<Clicks>,
    mut frames: ResMut<Frames>,
) {
    if buttons.just_pressed(DsButton::A) {
        clicks.0 += 1;
        let color = if clicks.0.is_multiple_of(2) {
            Color::rgb8(24, 48, 80)
        } else {
            Color::rgb8(24, 136, 104)
        };
        frames.bottom.set_color(0, color);
    }
}

fn application() -> App {
    let mut app = App::new();
    app.add_plugins(DsPlugins)
        .init_resource::<Clicks>()
        .add_systems(Startup, setup)
        .add_systems(Update, press);
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
    app.insert_resource(InputSample::default().with_button(DsButton::A));
    app.update();
    present(&mut app);
    assert_eq!(app.world().resource::<Clicks>().0, 1);
    println!(
        "Entrada y ECS correctos; {} entradas de paleta transferidas.",
        app.world().resource::<FrameStats>().palette_entries
    );
}
