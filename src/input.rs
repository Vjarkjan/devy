//! Botones y lápiz sobre `ButtonInput` y `Touches` de Bevy.
//! Conserva el modelo de entrada de bevy_nds usado por Digraph-dsi.
use bevy_app::{App, Plugin, PreUpdate};
use bevy_ecs::prelude::*;
use bevy_input::{
    ButtonInput,
    touch::{TouchInput, TouchPhase, Touches, touch_screen_input_system},
};
use bevy_math::Vec2;

/// Los doce botones físicos; el lápiz se consulta mediante `Touches`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DsButton {
    A,
    B,
    X,
    Y,
    L,
    R,
    Start,
    Select,
    Up,
    Down,
    Left,
    Right,
}

impl DsButton {
    pub const ALL: [Self; 12] = [
        Self::A,
        Self::B,
        Self::X,
        Self::Y,
        Self::L,
        Self::R,
        Self::Start,
        Self::Select,
        Self::Up,
        Self::Down,
        Self::Left,
        Self::Right,
    ];

    /// Máscara KEY_* de libnds.
    pub const fn mask(self) -> u32 {
        1 << match self {
            Self::A => 0,
            Self::B => 1,
            Self::Select => 2,
            Self::Start => 3,
            Self::Right => 4,
            Self::Left => 5,
            Self::Up => 6,
            Self::Down => 7,
            Self::R => 8,
            Self::L => 9,
            Self::X => 10,
            Self::Y => 11,
        }
    }
}

/// Mano que sostiene el lápiz. Los controles auxiliares quedan en la otra.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Handedness {
    #[default]
    Right,
    Left,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cluster {
    Up,
    Down,
    Left,
    Right,
}

impl Cluster {
    pub const fn button(self, hand: Handedness) -> DsButton {
        match (hand, self) {
            (Handedness::Right, Self::Up) => DsButton::Up,
            (Handedness::Right, Self::Down) => DsButton::Down,
            (Handedness::Right, Self::Left) => DsButton::Left,
            (Handedness::Right, Self::Right) => DsButton::Right,
            (Handedness::Left, Self::Up) => DsButton::X,
            (Handedness::Left, Self::Down) => DsButton::B,
            (Handedness::Left, Self::Left) => DsButton::Y,
            (Handedness::Left, Self::Right) => DsButton::A,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shoulder {
    Primary,
    Secondary,
}

impl Shoulder {
    pub const fn button(self, hand: Handedness) -> DsButton {
        match (self, hand) {
            (Self::Primary, Handedness::Right) | (Self::Secondary, Handedness::Left) => DsButton::L,
            _ => DsButton::R,
        }
    }
}

/// Lectura de un frame. En host puedes insertarla para simular controles.
/// DS la reemplaza al comienzo de cada PreUpdate. Coordenadas en píxeles.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct InputSample {
    pub held: u32,
    pub touch: Option<(u16, u16)>,
}

impl InputSample {
    pub fn with_button(mut self, button: DsButton) -> Self {
        self.held |= button.mask();
        self
    }
}

/// Permite ordenar sistemas adicionales antes o después del escaneo.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputSystems;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputSample>()
            .init_resource::<ButtonInput<DsButton>>()
            .init_resource::<Touches>()
            .init_resource::<Handedness>()
            .add_event::<TouchInput>()
            .add_systems(
                PreUpdate,
                (sample, buttons, touch, touch_screen_input_system)
                    .chain()
                    .in_set(InputSystems),
            );
    }
}

fn sample(mut input: ResMut<InputSample>) {
    if let Some(current) = crate::platform::read_input() {
        *input = current;
    }
}

fn buttons(sample: Res<InputSample>, mut buttons: ResMut<ButtonInput<DsButton>>) {
    buttons.clear();
    for button in DsButton::ALL {
        if sample.held & button.mask() != 0 {
            buttons.press(button);
        } else {
            buttons.release(button);
        }
    }
}

fn touch(
    sample: Res<InputSample>,
    mut previous: Local<Option<Vec2>>,
    mut events: EventWriter<TouchInput>,
) {
    let current = sample
        .touch
        .map(|(x, y)| Vec2::new(x.min(255) as f32, y.min(191) as f32));
    let event = match (*previous, current) {
        (None, Some(p)) => Some((TouchPhase::Started, p)),
        (Some(old), Some(p)) if old != p => Some((TouchPhase::Moved, p)),
        (Some(p), None) => Some((TouchPhase::Ended, p)),
        _ => None,
    };
    *previous = current;
    if let Some((phase, position)) = event {
        events.write(TouchInput {
            phase,
            position,
            window: Entity::PLACEHOLDER,
            force: None,
            id: 0,
        });
    }
}
