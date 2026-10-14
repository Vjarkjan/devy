//! Inicialización por capas de kts-nds. Las estructuras del SDK viven en C.
use super::{DsScreen, MasterBright, VideoPlugin};
use crate::Present;
use bevy_app::prelude::*;
use bevy_ecs::{prelude::*, schedule::ExecutorKind};

/// Tipo opaco. Solo el SDK conoce su tamaño y contenido.
#[repr(C)]
pub struct PrintConsole {
    _opaque: [u8; 0],
}

#[derive(Clone, Copy)]
pub struct ConsoleHandle(usize);
impl ConsoleHandle {
    /// Puntero prestado al SDK; no se debe liberar ni usar desde otro hilo.
    pub fn as_ptr(self) -> *mut PrintConsole {
        self.0 as *mut PrintConsole
    }
}
#[derive(Resource, Clone, Copy)]
pub struct Consoles {
    top: ConsoleHandle,
    bottom: ConsoleHandle,
}
impl Consoles {
    pub fn handle(&self, screen: DsScreen) -> ConsoleHandle {
        match screen {
            DsScreen::Top => self.top,
            DsScreen::Bottom => self.bottom,
        }
    }
}
#[cfg(target_vendor = "nintendo")]
unsafe extern "C" {
    fn devy_hardware_init() -> i32;
    fn devy_get_console(screen: u32) -> *mut PrintConsole;
}
pub(crate) fn init_screens(mut commands: Commands) {
    #[cfg(target_vendor = "nintendo")]
    let (top, bottom) = unsafe {
        assert!(
            devy_hardware_init() != 0,
            "La pantalla DS ya tiene un propietario"
        );
        (devy_get_console(0) as usize, devy_get_console(1) as usize)
    };
    #[cfg(not(target_vendor = "nintendo"))]
    let (top, bottom) = (0, 0);
    commands.insert_resource(Consoles {
        top: ConsoleHandle(top),
        bottom: ConsoleHandle(bottom),
    });
}
pub fn master_bright_bits(level: i8) -> u16 {
    let level = level.clamp(-16, 16);
    match level {
        0 => 0,
        n if n < 0 => (2 << 14) | (-(n as i16)) as u16,
        n => (1 << 14) | n as u16,
    }
}
fn apply_brightness(bright: Res<MasterBright>) {
    #[cfg(target_vendor = "nintendo")]
    if bright.is_changed() {
        unsafe {
            crate::platform::ffi::devy_brightness(
                bright.top.clamp(-16, 16) as i32,
                bright.bottom.clamp(-16, 16) as i32,
            );
        }
    }
    #[cfg(not(target_vendor = "nintendo"))]
    let _ = bright;
}
/// Consolas BG0 en ambos motores, sin reservar framebuffers en RAM.
pub struct HardwareVideoPlugin;
impl Plugin for HardwareVideoPlugin {
    fn build(&self, app: &mut App) {
        assert!(
            !app.is_plugin_added::<VideoPlugin>(),
            "VideoPlugin y HardwareVideoPlugin reservan la misma VRAM"
        );
        app.init_resource::<MasterBright>()
            .init_schedule(Present)
            .edit_schedule(Present, |s| {
                s.set_executor_kind(ExecutorKind::SingleThreaded);
            })
            .add_systems(PreStartup, init_screens)
            .add_systems(Present, apply_brightness);
    }
}
