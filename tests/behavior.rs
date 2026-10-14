use core::time::Duration;
use devy::prelude::*;

fn video_app() -> App {
    let mut app = App::new();
    app.add_plugins(DsPlugins);
    app.update();
    present(&mut app);
    app
}

#[test]
fn input_edges_and_touch_release_survive_full_schedule() {
    let mut app = App::new();
    app.add_plugins(DsCorePlugins);
    app.insert_resource(InputSample {
        touch: Some((18, 29)),
        ..InputSample::default().with_button(DsButton::A)
    });
    app.update();
    assert!(
        app.world()
            .resource::<ButtonInput<DsButton>>()
            .just_pressed(DsButton::A)
    );
    assert_eq!(
        app.world()
            .resource::<Touches>()
            .get_pressed(0)
            .unwrap()
            .position(),
        Vec2::new(18.0, 29.0)
    );
    app.update();
    assert!(
        !app.world()
            .resource::<ButtonInput<DsButton>>()
            .just_pressed(DsButton::A)
    );
    app.insert_resource(InputSample::default());
    app.update();
    assert!(
        app.world()
            .resource::<ButtonInput<DsButton>>()
            .just_released(DsButton::A)
    );
    assert_eq!(
        app.world()
            .resource::<Touches>()
            .get_released(0)
            .unwrap()
            .position(),
        Vec2::new(18.0, 29.0)
    );
    app.update();
    assert!(
        !app.world()
            .resource::<ButtonInput<DsButton>>()
            .just_released(DsButton::A)
    );
    assert!(app.world().resource::<Touches>().get_released(0).is_none());
}

#[test]
fn every_button_and_handed_control_maps_to_expected_input() {
    let mut app = App::new();
    app.add_plugins(InputPlugin);
    let mut sample = InputSample::default();
    for button in DsButton::ALL {
        sample = sample.with_button(button);
    }
    assert_eq!(sample.held, 0xfff);
    app.insert_resource(sample);
    app.update();
    for button in DsButton::ALL {
        assert!(
            app.world()
                .resource::<ButtonInput<DsButton>>()
                .pressed(button)
        );
    }
    assert_eq!(Cluster::Up.button(Handedness::Left), DsButton::X);
    assert_eq!(Shoulder::Primary.button(Handedness::Right), DsButton::L);
}

#[test]
fn idle_frames_copy_nothing_and_odd_bottom_pixel_preserves_neighbor() {
    let mut app = video_app();
    assert_eq!(
        app.world().resource::<FrameStats>().pixels_copied,
        2 * WIDTH * HEIGHT
    );
    present(&mut app);
    assert_eq!(app.world().resource::<FrameStats>().pixels_copied, 0);
    app.world_mut()
        .resource_mut::<Frames>()
        .bottom
        .plot(255, 191, 9);
    present(&mut app);
    let stats = app.world().resource::<FrameStats>();
    assert_eq!((stats.pixels_copied, stats.rows_copied), (2, 1));
    let frame = &app.world().resource::<Frames>().bottom;
    assert_eq!(&frame.pixels()[WIDTH * HEIGHT - 2..], &[0, 9]);
    present(&mut app);
    assert_eq!(app.world().resource::<FrameStats>().pixels_copied, 0);
}

#[test]
fn palette_changes_invalidate_only_the_required_transfers() {
    let mut app = video_app();
    app.world_mut()
        .resource_mut::<Frames>()
        .bottom
        .set_color(9, Color::WHITE);
    present(&mut app);
    let stats = app.world().resource::<FrameStats>();
    assert_eq!((stats.pixels_copied, stats.palette_entries), (0, 256));
    app.world_mut()
        .resource_mut::<Frames>()
        .top
        .set_color(9, Color::WHITE);
    present(&mut app);
    assert_eq!(
        app.world().resource::<FrameStats>().pixels_copied,
        WIDTH * HEIGHT
    );
}

#[test]
fn drawing_rejects_invalid_input_before_mutating_pixels() {
    let mut frame = IndexedFrame::default();
    assert_eq!(
        Rect::new(usize::MAX, 0, 1, 1),
        Err(DrawError::OutsideScreen)
    );
    assert_eq!(
        Rect::new(0, 0, usize::MAX, 1),
        Err(DrawError::OutsideScreen)
    );
    let rect = Rect::new(254, 191, 2, 1).unwrap();
    assert_eq!(frame.blit(rect, &[8]), Err(DrawError::WrongLength));
    assert!(frame.pixels().iter().all(|p| *p == 0));
    frame.blit(rect, &[8, 9]).unwrap();
    assert_eq!(&frame.pixels()[WIDTH * HEIGHT - 2..], &[8, 9]);
    assert_eq!(
        frame.line((-1, 0), (10, 10), 4),
        Err(DrawError::OutsideScreen)
    );
    assert!(!frame.plot(i32::MAX, i32::MIN, 4));
    frame.blit(Rect::new(256, 192, 0, 0).unwrap(), &[]).unwrap();
}

#[test]
fn lines_cover_endpoints_in_each_direction() {
    for end in [(0, 0), (255, 0), (0, 191), (255, 191), (128, 96)] {
        let mut frame = IndexedFrame::default();
        frame.line((128, 96), end, 1).unwrap();
        assert_eq!(frame.pixels()[96 * WIDTH + 128], 1);
        assert_eq!(frame.pixels()[end.1 as usize * WIDTH + end.0 as usize], 1);
        assert!(frame.pixels().iter().filter(|p| **p == 1).count() <= WIDTH);
    }
}

#[test]
fn host_time_is_deterministic() {
    let mut app = App::new();
    app.add_plugins(TimePlugin)
        .insert_resource(FrameDelta(Duration::from_millis(20)));
    for _ in 0..5 {
        app.update();
    }
    assert_eq!(
        app.world().resource::<Time>().elapsed(),
        Duration::from_millis(100)
    );
    assert_eq!(
        app.world().resource::<Time>().delta(),
        Duration::from_millis(20)
    );
}

#[cfg(feature = "storage")]
#[test]
fn host_storage_reports_unsupported_without_mutating_buffer() {
    let mut storage = Storage::default();
    let mut buffer = [0xa5; 16];
    assert_eq!(
        storage.read_into(c"fat:/missing", &mut buffer),
        Err(StorageError::Unsupported)
    );
    assert_eq!(buffer, [0xa5; 16]);
}
