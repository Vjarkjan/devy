use devy::audio::{Audio, AudioPlugin};
use devy::prelude::*;
use devy::render3d::{Ds3dPlugin, DsMesh, Transform3d};
use devy::scene::ScenePlugin;
use devy::stroke::{LoopPlugin, StrokeCompleted};

#[test]
fn hardware_plugins_keep_logic_testable_without_mmio() {
    let mut app = App::new();
    app.add_plugins(HardwarePlugins).add_plugins((
        BackgroundPlugin,
        SpritePlugin,
        Ds3dPlugin,
        AudioPlugin,
        ScenePlugin,
        LoopPlugin,
        SavePlugin::default(),
    ));
    app.world_mut().spawn((
        devy::include_obj!("assets/cube.obj", center),
        Transform3d::default(),
    ));
    app.insert_resource(InputSample {
        held: 0,
        touch: Some((40, 50)),
    });
    app.update();
    present(&mut app);
    app.insert_resource(InputSample::default());
    app.update();
    present(&mut app);
    assert!(!app.world().resource::<NitroFs>().ready);
    assert!(!app.world().resource::<Audio>().ready);
    assert_eq!(app.world().resource::<WallClock>().unix_secs, 0);
    assert_eq!(
        app.world().resource::<SaveStorage>().status(),
        StorageStatus::Unavailable
    );
    assert_eq!(app.world().resource::<Events<StrokeCompleted>>().len(), 1);
    assert!(
        app.world()
            .resource::<Events<GestureEvent>>()
            .iter_current_update_events()
            .any(|event| matches!(event.0, Gesture::Tap(_)))
    );
    assert_eq!(
        app.world_mut().query::<&DsMesh>().iter(app.world()).count(),
        1
    );
}

#[test]
#[should_panic(expected = "reservan la misma VRAM")]
fn renderer_configurations_cannot_share_vram() {
    let mut app = App::new();
    app.add_plugins(VideoPlugin)
        .add_plugins(HardwareVideoPlugin);
}

#[test]
#[should_panic(expected = "comparten VRAM")]
fn canvas_cannot_overwrite_loaded_tile_region() {
    let mut backgrounds = Backgrounds::default();
    backgrounds.set_tile(DsScreen::Bottom, b"nitro:/grid.bg\0");
    backgrounds.enable_paint_canvas();
}

#[test]
fn invalid_nitrofs_paths_fail_without_reading_past_the_slice() {
    assert!(devy::nitrofs::read_file(b"nitro:/missing").is_none());
    assert!(devy::nitrofs::read_file(b"nitro:/a\0b\0").is_none());
    assert!(devy::nitrofs::read_file(&[]).is_none());
}
