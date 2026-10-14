//! 3D, texto, tiles, OAM, NitroFS, audio, RTC y tareas sobre kts-nds.
#![cfg_attr(target_vendor = "nintendo", no_std)]
#![cfg_attr(target_vendor = "nintendo", no_main)]
extern crate alloc;
use alloc::format;
use devy::audio::{Audio, AudioPlugin, Music, PlaySfx, SoundId};
use devy::prelude::*;
use devy::render3d::prelude::*;
use devy::scene::{self, ScenePlugin};

#[derive(Component)]
struct Status;
#[derive(Resource, Default)]
struct Job(Option<Task<usize>>);
#[derive(Resource, Default)]
struct Loaded(usize);

fn setup(
    mut commands: Commands,
    mut backgrounds: ResMut<Backgrounds>,
    tasks: Res<Tasks>,
    mut job: ResMut<Job>,
) {
    commands.spawn((
        include_obj!("assets/cube.obj", center),
        Transform3d::default(),
    ));
    commands.spawn(Sprite::new(b"nitro:/sprites/player.sprite\0").at(112, 130));
    commands.spawn((
        DsScreen::Bottom,
        TilePos { x: 1, y: 1 },
        DsText("Devy / kts-nds".into()),
    ));
    commands.spawn((
        Status,
        DsScreen::Bottom,
        TilePos { x: 1, y: 4 },
        DsText("Cargando...".into()),
    ));
    backgrounds.set_tile(DsScreen::Bottom, b"nitro:/backgrounds/tiled/grid.bg\0");
    backgrounds.set_bitmap(DsScreen::Top, b"nitro:/backgrounds/bitmap/sunset.bbg\0");
    job.0 = Some(tasks.spawn(|| {
        let path = scene::level_space_path("facility", "atrium");
        scene::load(&path).map_or(0, |scene| scene.instances.len())
    }));
}
fn animate(
    time: Res<Time>,
    mut transforms: Query<&mut Transform3d>,
    touches: Res<Touches>,
    mut sprites: Query<&mut Sprite>,
) {
    for mut t in &mut transforms {
        t.rotation.y = time.elapsed_secs() * 0.5;
    }
    if let Some(touch) = touches.iter().next() {
        for mut sprite in &mut sprites {
            sprite.x = touch.position().x as i16;
            sprite.y = touch.position().y as i16;
        }
    }
}
fn status(
    clock: Res<WallClock>,
    nitro: Res<NitroFs>,
    audio: Res<Audio>,
    sprites: Res<devy::sprite::SpriteAssets>,
    mut text: Query<&mut DsText, With<Status>>,
    mut job: ResMut<Job>,
    mut loaded: ResMut<Loaded>,
) {
    if let Some(task) = &mut job.0
        && let Some(count) = task.poll()
    {
        loaded.0 = count;
        job.0 = None;
    }
    for mut text in &mut text {
        text.0 = format!(
            "NitroFS={} Audio={}\nHora {:02}:{:02}:{:02}\nEscena: {} objetos\nSprites: {}\nA: efecto  B: musica\nToca para mover el sprite",
            nitro.ready,
            audio.ready,
            clock.hour,
            clock.minute,
            clock.second,
            loaded.0,
            sprites.loaded_count()
        );
    }
}
fn sound(
    buttons: Res<ButtonInput<DsButton>>,
    mut effects: EventWriter<PlaySfx>,
    mut music: ResMut<Music>,
) {
    // IDs determinados por wav2bank para los dos WAV incluidos; ver output/sounds.rs.
    if buttons.just_pressed(DsButton::A) {
        effects.write(PlaySfx::new(SoundId(1)));
    }
    if buttons.just_pressed(DsButton::B) {
        music.play(SoundId(0));
    }
}
fn application() -> App {
    let mut app = App::new();
    app.add_plugins(HardwarePlugins)
        .add_plugins((
            BackgroundPlugin,
            SpritePlugin,
            Ds3dPlugin,
            AudioPlugin,
            ScenePlugin,
        ))
        .init_resource::<Job>()
        .init_resource::<Loaded>()
        .add_systems(Startup, setup)
        .add_systems(Update, (animate, status, sound));
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
    println!("Sistemas y recursos de hardware inicializados en host; sin emulación.");
}
