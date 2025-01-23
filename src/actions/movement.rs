use crate::actions::*;
use bevy::prelude::*;

/// [`Resource`] for enabling movement vector adjusted for isometric movement during adjacent directions.
/// [`true`] by default
#[derive(Resource)]
pub struct IsometricMovement(pub bool);

impl Default for IsometricMovement {
    fn default() -> Self {
        Self(true)
    }
}

/// [`Event`] representing 8 cardinal directions for player movement with [`Vec2`] unit conversion.
/// Diagonal movement is isometric adjusted by default
#[derive(Event, Reflect, Eq, PartialEq, Hash, Clone, Debug, Copy)]
pub enum MovementAction {
    North,
    South,
    East,
    West,

    NorthWest,
    NorthEast,
    SouthWest,
    SouthEast,
}

impl InputAction for MovementAction {
    fn display(&self) -> String {
        match *self {
            Self::North => "Move north",
            Self::South => "Move south",
            Self::East => "Move east",
            Self::West => "Move west",
            Self::NorthEast => "Move north-east",
            Self::SouthEast => "Move south-east",
            Self::NorthWest => "Move north-west",
            Self::SouthWest => "Move south-west",
        }
        .into()
    }

    fn desktop_mapping(&self, input_mapping: &Res<InputMappings>) -> Option<InputMappingEntry> {
        input_mapping.movement_actions.get(&*self).cloned()
    }
}

impl MovementAction {
    /// unit north direction
    pub const NORTH: Vec2 = Vec2::Y;
    /// unit south direction
    pub const SOUTH: Vec2 = Vec2::NEG_Y;
    /// unit east direction
    pub const EAST: Vec2 = Vec2::X;
    /// unit west direction
    pub const WEST: Vec2 = Vec2::NEG_X;

    /// unit north-east direction
    pub const NORTH_EAST: Vec2 = Vec2::new(0.70710677, 0.70710677);
    /// unit south-east direction
    pub const SOUTH_EAST: Vec2 = Vec2::new(0.70710677, -0.70710677);
    /// unit south-west direction
    pub const SOUTH_WEST: Vec2 = Vec2::new(-0.70710677, -0.70710677);
    /// unit north-west direction
    pub const NORTH_WEST: Vec2 = Vec2::new(-0.70710677, 0.70710677);

    ///// isometric adjusted north-west direction
    pub const NORTH_WEST_ISO: Vec2 = Vec2::new(-0.8660, 0.432222);
    ///// isometric adjusted north-east direction
    pub const NORTH_EAST_ISO: Vec2 = Vec2::new(0.865, 0.432222);
    ///// isometric adjusted south-west direction
    pub const SOUTH_WEST_ISO: Vec2 = Vec2::new(-0.8660, -0.432222);
    ///// isometric adjusted south-east direction
    pub const SOUTH_EAST_ISO: Vec2 = Vec2::new(0.8660, -0.432222);
}

impl From<&MovementAction> for Vec2 {
    fn from(action: &MovementAction) -> Self {
        match *action {
            MovementAction::North => MovementAction::NORTH,
            MovementAction::South => MovementAction::SOUTH,
            MovementAction::East => MovementAction::EAST,
            MovementAction::West => MovementAction::WEST,
            MovementAction::NorthWest => MovementAction::NORTH_WEST_ISO,
            MovementAction::NorthEast => MovementAction::NORTH_EAST_ISO,
            MovementAction::SouthWest => MovementAction::SOUTH_WEST_ISO,
            MovementAction::SouthEast => MovementAction::SOUTH_EAST_ISO,
            //MovementAction::NorthWest => MovementAction::NORTH_WEST,
            //MovementAction::NorthEast => MovementAction::NORTH_EAST,
            //MovementAction::SouthWest => MovementAction::SOUTH_WEST,
            //MovementAction::SouthEast => MovementAction::SOUTH_EAST,
        }
    }
}

/// Writes to [`EventWriter<MovementAction>`] based on [`Res<InputMappings>`].
pub(crate) fn dispatch_movement_actions(
    mut movement_actions_event: EventWriter<MovementAction>,
    input_mappings: Res<InputMappings>,
    mouse_scroll: MouseScrollEvent,
    mouse_button: MouseButtonResource,
    key_button: KeyButtonResource,

    mut held_actions: Local<HashSet<MovementAction>>,
) {
    for (
        action,
        InputMappingEntry {
            primary, secondary, ..
        },
    ) in input_mappings.movement_actions.iter()
    {
        let primary_fired = primary.iter().fold(true, |acc, ctrl| {
            acc && match ctrl {
                DesktopControl::Mouse(m) => match m {
                    Mouse::Button(mb) => mouse_button.pressed(*mb),
                    Mouse::Scroll { y, .. } => *y != 0,
                },
                DesktopControl::Key(k) => key_button.pressed(*k),
            }
        });

        let secondary_fired = secondary.iter().fold(true, |acc, ctrl| {
            acc && match ctrl {
                DesktopControl::Mouse(m) => match m {
                    Mouse::Button(mb) => mouse_button.pressed(*mb),
                    Mouse::Scroll { y, .. } => *y != 0,
                },
                DesktopControl::Key(k) => key_button.pressed(*k),
            }
        });

        if primary_fired || secondary_fired {
            held_actions.insert(*action);
        } else {
            held_actions.remove(&*action);
        }
    }

    fn override_movement(
        buffer: &mut HashSet<MovementAction>,
        lateral: &MovementAction,
        adjacent: &MovementAction,
        solution: MovementAction,
    ) {
        if buffer.contains(lateral) && buffer.contains(adjacent) {
            buffer.remove(lateral);
            buffer.remove(adjacent);
            buffer.insert(solution);
        } else {
            buffer.remove(&solution);
        }
    }

    override_movement(
        &mut held_actions,
        &MovementAction::North,
        &MovementAction::West,
        MovementAction::NorthWest,
    );

    override_movement(
        &mut held_actions,
        &MovementAction::North,
        &MovementAction::East,
        MovementAction::NorthEast,
    );

    override_movement(
        &mut held_actions,
        &MovementAction::South,
        &MovementAction::West,
        MovementAction::SouthWest,
    );

    override_movement(
        &mut held_actions,
        &MovementAction::South,
        &MovementAction::East,
        MovementAction::SouthEast,
    );

    for ma in held_actions.iter() {
        movement_actions_event.send(*ma);
    }
}
