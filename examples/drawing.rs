#![cfg_attr(target_vendor = "nintendo", no_std)]
#![cfg_attr(target_vendor = "nintendo", no_main)]

use devy::prelude::*;

#[derive(Resource)]
struct Cursor {
    x: i32,
    y: i32,
    dx: i32,
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            x: 24,
            y: 88,
            dx: 1,
        }
    }
}

fn setup(mut frames: ResMut<Frames>) {
    for screen in [DsScreen::Top, DsScreen::Bottom] {
        let frame = frames.get_mut(screen);
        frame.set_color(0, Color::rgb8(16, 24, 40));
        frame.set_color(1, Color::rgb8(80, 208, 168));
        frame.set_color(2, Color::rgb8(240, 192, 80));
    }
    let vertices = [(48, 56), (200, 56), (128, 140)];
    for (from, to) in [(0, 1), (1, 2), (2, 0)] {
        frames.top.line(vertices[from], vertices[to], 1).unwrap();
    }
    for (x, y) in vertices {
        frames
            .top
            .fill_rect(Rect::new(x as usize - 4, y as usize - 4, 9, 9).unwrap(), 2);
    }
}

fn draw(
    mut cursor: ResMut<Cursor>,
    touches: Res<Touches>,
    buttons: Res<ButtonInput<DsButton>>,
    mut frames: ResMut<Frames>,
) {
    let frame = &mut frames.bottom;
    frame.fill_rect(
        Rect::new(cursor.x as usize, cursor.y as usize, 8, 8).unwrap(),
        0,
    );
    if let Some(touch) = touches.iter().next() {
        cursor.x = (touch.position().x as i32).clamp(0, 248);
        cursor.y = (touch.position().y as i32).clamp(0, 184);
    } else {
        if buttons.just_pressed(DsButton::A) {
            cursor.dx = -cursor.dx;
        }
        if cursor.x + cursor.dx < 0 || cursor.x + cursor.dx > 248 {
            cursor.dx = -cursor.dx;
        }
        cursor.x += cursor.dx;
    }
    frame.fill_rect(
        Rect::new(cursor.x as usize, cursor.y as usize, 8, 8).unwrap(),
        2,
    );
}

fn application() -> App {
    let mut app = App::new();
    app.add_plugins(DsPlugins)
        .init_resource::<Cursor>()
        .add_systems(Startup, setup)
        .add_systems(Update, draw);
    app
}

#[cfg(target_vendor = "nintendo")]
#[unsafe(no_mangle)]
pub extern "C" fn main() -> core::ffi::c_int {
    devy::run(application())
}

#[cfg(not(target_vendor = "nintendo"))]
fn main() -> std::io::Result<()> {
    use std::{fs, io::Write, path::PathBuf};
    let mut app = application();
    for _ in 0..120 {
        app.update();
        present(&mut app);
    }
    let output = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("output/preview"));
    fs::create_dir_all(&output)?;
    for (screen, name) in [(DsScreen::Top, "top.ppm"), (DsScreen::Bottom, "bottom.ppm")] {
        let frame = app.world().resource::<Frames>().get(screen);
        let mut file = fs::File::create(output.join(name))?;
        write!(file, "P6\n{WIDTH} {HEIGHT}\n255\n")?;
        for &index in frame.pixels() {
            file.write_all(&frame.palette()[index as usize].to_rgb8())?;
        }
    }
    let stats = app.world().resource::<FrameStats>();
    println!(
        "{} píxeles en {} filas en el último frame; imágenes en {}",
        stats.pixels_copied,
        stats.rows_copied,
        output.display()
    );
    Ok(())
}
