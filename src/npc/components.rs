//! Data that describes an NPC and the state its lifecycle moves through.

use crate::levels::NpcKind;
use bevy::prelude::*;

/// Marker for a simulated non-player character.
#[derive(Component, Debug, Clone, Copy)]
pub struct Npc {
    pub kind: NpcKind,
}

/// Physical properties used by movement and line-of-sight checks.
#[derive(Component, Debug, Clone, Copy)]
pub struct NpcBody {
    pub radius: f32,
    /// Body height in elevation units.
    pub height: f32,
    pub speed: f32,
    pub turn_speed: f32,
}

impl NpcBody {
    pub fn for_kind(kind: NpcKind) -> Self {
        match kind {
            NpcKind::Civilian => Self {
                radius: 8.0,
                height: 0.9,
                speed: 92.0,
                turn_speed: 6.0,
            },
            NpcKind::Guard => Self {
                radius: 9.5,
                height: 1.0,
                speed: 132.0,
                turn_speed: 9.0,
            },
        }
    }
}

/// Ground height sampled from the owning level's [`crate::levels::LevelVertical`].
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct NpcHeight {
    pub elevation: f32,
    pub rank: i16,
}

/// The high-level lifecycle state an NPC currently occupies.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NpcMode {
    #[default]
    Idle,
    Patrol,
    Investigate,
    Chase,
}

/// Patrol loop in ground-space coordinates.
#[derive(Component, Debug, Clone, Default)]
pub struct NpcPatrol {
    pub points: Vec<Vec2>,
    pub index: usize,
}

impl NpcPatrol {
    pub fn new(points: Vec<Vec2>) -> Self {
        Self { points, index: 0 }
    }

    pub fn current(&self) -> Option<Vec2> {
        self.points.get(self.index).copied()
    }

    pub fn advance(&mut self) {
        if !self.points.is_empty() {
            self.index = (self.index + 1) % self.points.len();
        }
    }
}

/// Cone used to detect the player.
#[derive(Component, Debug, Clone, Copy)]
pub struct NpcVision {
    pub range: f32,
    pub half_angle: f32,
}

impl NpcVision {
    pub fn for_kind(kind: NpcKind) -> Self {
        match kind {
            NpcKind::Civilian => Self {
                range: 190.0,
                half_angle: 55.0_f32.to_radians(),
            },
            NpcKind::Guard => Self {
                range: 290.0,
                half_angle: 45.0_f32.to_radians(),
            },
        }
    }
}

/// What the NPC currently perceives, refreshed by the perception system.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct NpcPerception {
    pub sees_target: bool,
    pub target: Option<Vec2>,
}

/// Short-term memory that lets an NPC investigate after losing sight.
#[derive(Component, Debug, Clone, Copy)]
pub struct NpcMemory {
    pub last_known: Option<Vec2>,
    pub time_since_seen: f32,
    pub remember_for: f32,
}

impl Default for NpcMemory {
    fn default() -> Self {
        Self {
            last_known: None,
            time_since_seen: 0.0,
            remember_for: 6.0,
        }
    }
}

/// Where movement should try to go next.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct NpcNavigation {
    pub destination: Option<Vec2>,
    pub stop_distance: f32,
}

/// Smoothed ground-space velocity.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct NpcVelocity(pub Vec2);

/// Last movement direction, also used as the vision direction.
#[derive(Component, Debug, Clone, Copy)]
pub struct NpcFacing(pub Vec2);

impl Default for NpcFacing {
    fn default() -> Self {
        Self(Vec2::NEG_Y)
    }
}

/// Marks the child entity that renders the NPC trapezoid.
#[derive(Component, Debug, Clone, Copy)]
pub struct NpcVisual {
    pub kind: NpcKind,
}
