//! Motor Bevy/BlocksDS derivado de kts-nds: gráficos, entrada, assets y servicios.
//!
//! ```
//! use devy::prelude::*;
//!
//! #[derive(Resource, Default)]
//! struct Counter(u32);
//!
//! let mut app = App::new();
//! app.add_plugins(DsCorePlugins).init_resource::<Counter>();
//! app.add_systems(Update, |mut count: ResMut<Counter>| count.0 += 1);
//! app.update();
//! assert_eq!(app.world().resource::<Counter>().0, 1);
//! ```
#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]

extern crate alloc;
#[cfg(test)]
#[macro_use]
extern crate std;

pub use bevy_app;
pub use bevy_ecs;
pub use bevy_input;
pub use bevy_math;
pub use bevy_time;

pub mod audio;
pub mod background;
pub mod cothread;
pub mod culling;
pub mod diagnostics;
pub mod gesture;
pub mod input;
pub mod math;
pub mod nitrofs;
mod platform;
pub mod render3d;
pub mod rtc;
pub mod runtime;
pub mod save;
pub mod scene;
pub mod sprite;
#[cfg(feature = "storage")]
pub mod storage;
pub mod stroke;
pub mod text;
pub mod time;
pub mod video;

/// Guía de programación; sus ejemplos se verifican con `cargo test --doc`.
#[doc = include_str!("../Doc/README.md")]
pub mod guide {
    #[doc = include_str!("../Doc/02-ecs.md")]
    pub mod ecs {}
    #[doc = include_str!("../Doc/03-entrada.md")]
    pub mod input {}
    #[doc = include_str!("../Doc/04-pantallas.md")]
    pub mod video {}
    #[doc = include_str!("../Doc/10-capas-2d.md")]
    pub mod layers {}
    #[doc = include_str!("../Doc/11-graficos-3d.md")]
    pub mod graphics3d {}
    #[doc = include_str!("../Doc/12-assets-audio.md")]
    pub mod assets {}
    #[doc = include_str!("../Doc/13-gestos-tareas.md")]
    pub mod gestures_tasks {}
    #[doc = include_str!("../Doc/14-matematicas-relojes.md")]
    pub mod math_clock {}
    #[doc = include_str!("../Doc/05-tiempo.md")]
    pub mod time {}
    #[cfg(feature = "storage")]
    #[doc = include_str!("../Doc/06-almacenamiento.md")]
    pub mod storage {}
}

use bevy_app::{PluginGroup, PluginGroupBuilder};
pub use input::{Cluster, DsButton, Handedness, InputPlugin, InputSample, Shoulder};
#[cfg(target_vendor = "nintendo")]
pub use runtime::run;
pub use runtime::{Present, present};
pub use time::{FrameDelta, TimePlugin};
pub use video::{
    Color, ConsoleHandle, Consoles, DsScreen, FrameStats, Frames, HardwareVideoPlugin,
    MasterBright, PrintConsole, Video, VideoPlugin,
};

/// Entrada y reloj, sin reservar pantallas ni framebuffers.
pub struct DsCorePlugins;

impl PluginGroup for DsCorePlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(InputPlugin)
            .add(TimePlugin)
    }
}

/// Configuración habitual: entrada, reloj y las dos pantallas.
pub struct DsPlugins;

impl PluginGroup for DsPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(InputPlugin)
            .add(TimePlugin)
            .add(VideoPlugin)
    }
}

/// Configuración por capas de kts-nds: consolas, entrada, gestos y servicios.
/// Añade BackgroundPlugin, SpritePlugin y Ds3dPlugin según la escena.
pub struct HardwarePlugins;
impl PluginGroup for HardwarePlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(HardwareVideoPlugin)
            .add(InputPlugin)
            .add(TimePlugin)
            .add(gesture::GesturePlugin)
            .add(diagnostics::DiagnosticsPlugin)
            .add(nitrofs::NitroFsPlugin)
            .add(cothread::CothreadPlugin)
            .add(rtc::RtcPlugin)
            .add(text::TextRenderPlugin)
    }
}

#[doc(hidden)]
pub use bevy_nds_3d_macros::include_obj as __include_obj_data;
/// Compila un OBJ en una display list estática; la ruta parte de Cargo.toml.
#[macro_export]
macro_rules! include_obj {
    ($($args:tt)*) => {{
        let (words, min, max) = $crate::__include_obj_data!($($args)*);
        $crate::render3d::DsMesh::from_baked(words, min, max)
    }};
}

/// Importaciones habituales, incluidos los nombres que necesitan los derives.
pub mod prelude {
    pub use crate::background::{BackgroundPlugin, Backgrounds};
    pub use crate::cothread::{CothreadPlugin, Task, Tasks};
    pub use crate::diagnostics::{DiagnosticsPlugin, Fps};
    pub use crate::gesture::{Gesture, GestureEvent, GesturePlugin, Gestures, SwipeDir};
    pub use crate::math::{Fx32, FxVec2, FxVec3};
    pub use crate::nitrofs::{NitroFs, NitroFsPlugin};
    pub use crate::rtc::{RtcPlugin, WallClock};
    pub use crate::save::{SavePlugin, SaveStorage, StorageStatus};
    pub use crate::sprite::{Sprite, SpritePlugin};
    pub use crate::text::{DsText, Glyph, TextRenderPlugin, TilePos};
    pub use crate::{HardwarePlugins, HardwareVideoPlugin, include_obj};

    #[cfg(feature = "storage")]
    pub use crate::storage::{Storage, StorageError, StoragePlugin};
    pub use crate::video::{DrawError, HEIGHT, IndexedFrame, Rect, WIDTH};
    pub use crate::{
        Cluster, Color, DsButton, DsCorePlugins, DsPlugins, DsScreen, FrameDelta, FrameStats,
        Frames, Handedness, InputPlugin, InputSample, MasterBright, Present, Shoulder, TimePlugin,
        Video, VideoPlugin, bevy_app, bevy_ecs, bevy_input, bevy_math, bevy_time, present,
    };
    pub use bevy_app::prelude::*;
    pub use bevy_ecs::prelude::*;
    pub use bevy_input::{
        ButtonInput,
        touch::{TouchInput, TouchPhase, Touches},
    };
    pub use bevy_math::Vec2;
    pub use bevy_time::{Time, Timer, TimerMode};
}
