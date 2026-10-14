//! Bevy ECS sobre BlocksDS, con pantallas indexadas, entrada y reloj.
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

pub use bevy_app;
pub use bevy_ecs;
pub use bevy_input;
pub use bevy_math;
pub use bevy_time;

pub mod input;
mod platform;
pub mod runtime;
#[cfg(feature = "storage")]
pub mod storage;
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
pub use video::{Color, DsScreen, FrameStats, Frames, MasterBright, Video, VideoPlugin};

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

/// Importaciones habituales, incluidos los nombres que necesitan los derives.
pub mod prelude {
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
