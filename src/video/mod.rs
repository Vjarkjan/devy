//! Dos framebuffers en RAM, consola superior y presentación parcial en VRAM.
#[cfg(any(target_vendor = "nintendo", test))]
pub(crate) mod copy;
mod frame;
mod hardware;
pub use frame::{Color, DrawError, HEIGHT, IndexedFrame, PIXELS, Rect, WIDTH};
#[cfg(target_vendor = "nintendo")]
pub(crate) use hardware::init_screens;
pub use hardware::{
    ConsoleHandle, Consoles, HardwareVideoPlugin, PrintConsole, master_bright_bits,
};

use crate::{platform::VideoBackend, runtime::Present};
use bevy_app::{App, Plugin, PreStartup};
use bevy_ecs::{prelude::*, schedule::ExecutorKind};
use core::{ffi::CStr, time::Duration};

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DsScreen {
    Top,
    Bottom,
}

#[derive(Resource, Default)]
pub struct Frames {
    pub top: IndexedFrame,
    pub bottom: IndexedFrame,
}

impl Frames {
    pub fn get(&self, screen: DsScreen) -> &IndexedFrame {
        match screen {
            DsScreen::Top => &self.top,
            DsScreen::Bottom => &self.bottom,
        }
    }
    pub fn get_mut(&mut self, screen: DsScreen) -> &mut IndexedFrame {
        match screen {
            DsScreen::Top => &mut self.top,
            DsScreen::Bottom => &mut self.bottom,
        }
    }
}

/// -16: negro; 0: normal; 16: blanco. Los valores se limitan a ese rango.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MasterBright {
    pub top: i8,
    pub bottom: i8,
}

/// Última presentación. Los contadores incluyen el redondeo de filas inferiores.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct FrameStats {
    pub frames: u64,
    pub pixels_copied: usize,
    pub rows_copied: usize,
    pub palette_entries: usize,
    /// None en host; mide sólo la presentación, no Update ni la espera.
    pub upload_time: Option<Duration>,
    pub vblank_overruns: u64,
}

/// Acceso exclusivo a hardware. Se obtiene con `NonSendMut<Video>`.
/// En host sólo conserva el estado lógico; no abre una ventana.
pub struct Video {
    backend: VideoBackend,
}

impl Video {
    pub fn show_console(&mut self, visible: bool) {
        self.backend.show_console(visible);
    }
    /// La consola cubre la pantalla superior y usa la fuente del SDK.
    pub fn write_console(&mut self, text: &CStr) {
        self.backend.write_console(text);
    }
    pub fn clear_console(&mut self) {
        self.backend.clear_console();
    }
}

pub struct VideoPlugin;

impl Plugin for VideoPlugin {
    fn build(&self, app: &mut App) {
        assert!(
            !app.is_plugin_added::<HardwareVideoPlugin>(),
            "VideoPlugin y HardwareVideoPlugin reservan la misma VRAM"
        );
        app.init_resource::<Frames>()
            .init_resource::<MasterBright>()
            .init_resource::<FrameStats>()
            .init_schedule(Present)
            .edit_schedule(Present, |schedule| {
                schedule.set_executor_kind(ExecutorKind::SingleThreaded);
            })
            .add_systems(PreStartup, initialize)
            .add_systems(Present, upload);
    }
}

fn initialize(world: &mut World) {
    world.insert_non_send_resource(Video {
        backend: VideoBackend::take().expect("La pantalla DS ya tiene un propietario"),
    });
}

fn upload(
    mut frames: ResMut<Frames>,
    mut video: NonSendMut<Video>,
    bright: Res<MasterBright>,
    mut stats: ResMut<FrameStats>,
) {
    let start = crate::platform::ticks();
    stats.pixels_copied = 0;
    stats.rows_copied = 0;
    stats.palette_entries = 0;
    // Arriba, la paleta está expandida a RGB555 en VRAM.
    if frames.top.palette_dirty {
        frames.top.invalidate();
    }
    for screen in [DsScreen::Top, DsScreen::Bottom] {
        let frame = frames.get_mut(screen);
        if screen == DsScreen::Bottom && frame.palette_dirty {
            video.backend.palette(frame.palette());
            stats.palette_entries += 256;
        }
        for y in 0..HEIGHT {
            let span = frame.dirty[y];
            if span.start >= span.end {
                continue;
            }
            let (start, end) = if screen == DsScreen::Bottom {
                (
                    usize::from(span.start) & !1,
                    (usize::from(span.end) + 1) & !1,
                )
            } else {
                (usize::from(span.start), usize::from(span.end))
            };
            video.backend.row(
                screen,
                y,
                start,
                &frame.pixels()[y * WIDTH + start..y * WIDTH + end],
                frame.palette(),
            );
            stats.pixels_copied += end - start;
            stats.rows_copied += 1;
        }
        frame.mark_presented();
    }
    if bright.is_changed() {
        video.backend.brightness(bright.top, bright.bottom);
    }
    video.backend.show();
    stats.frames = stats.frames.wrapping_add(1);
    stats.upload_time = start
        .zip(crate::platform::ticks())
        .map(|(a, b)| crate::time::ticks_to_duration(b.wrapping_sub(a)));
    if crate::platform::outside_vblank() && stats.pixels_copied > 0 {
        stats.vblank_overruns = stats.vblank_overruns.wrapping_add(1);
    }
}
