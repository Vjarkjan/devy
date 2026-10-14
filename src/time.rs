//! Tiempo de frame. Comparte el contador del SDK y no reserva dos timers.
use bevy_app::{App, First, Plugin};
use bevy_ecs::prelude::*;
use bevy_time::Time;
use core::time::Duration;

/// Delta inyectable en host. En DS se mide el contador de hardware.
#[derive(Resource, Clone, Copy, Debug)]
pub struct FrameDelta(pub Duration);

impl Default for FrameDelta {
    fn default() -> Self {
        Self(Duration::from_nanos(16_715_113))
    }
}

/// Frecuencia truncada del contador, igual a la conversión del SDK instalado.
pub const COUNTER_HZ: u64 = 33_513_982 / 64;

/// Convierte sin multiplicar el contador entero, evitando overflow intermedio.
pub fn ticks_to_duration(ticks: u64) -> Duration {
    Duration::new(
        ticks / COUNTER_HZ,
        ((ticks % COUNTER_HZ) * 1_000_000_000 / COUNTER_HZ) as u32,
    )
}

#[derive(Resource, Default)]
struct Clock {
    previous: Option<u64>,
}

/// Avanza `Time<()>`. No instala tiempo virtual ni un ejecutor de paso fijo.
pub struct TimePlugin;

impl Plugin for TimePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Time>()
            .init_resource::<FrameDelta>()
            .init_resource::<Clock>()
            .add_systems(First, advance);
    }
}

fn advance(mut time: ResMut<Time>, delta: Res<FrameDelta>, mut clock: ResMut<Clock>) {
    if let Some(now) = crate::platform::ticks() {
        let elapsed = clock
            .previous
            .replace(now)
            .map_or(Duration::ZERO, |previous| {
                ticks_to_duration(now.wrapping_sub(previous))
            });
        time.advance_by(elapsed);
    } else {
        time.advance_by(delta.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn counter_conversion_covers_wrap_scale_and_long_uptimes() {
        assert_eq!(ticks_to_duration(0), Duration::ZERO);
        assert_eq!(ticks_to_duration(COUNTER_HZ), Duration::from_secs(1));
        assert_eq!(ticks_to_duration(u64::MAX).as_secs(), u64::MAX / COUNTER_HZ);
    }
}
