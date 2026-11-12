//! NPC lifecycle: spawning, perception, tracking, mode switching, navigation,
//! grounded movement and rendering.
//!
//! The lifecycle flows in one direction each frame:
//! `perception -> tracking -> mode switching -> navigation -> movement -> visual sync`.
//! NPCs are spawned beneath a [`crate::levels::LevelInstance`], so unloading or
//! regenerating a level removes its NPCs with it.

mod behavior;
mod components;
mod movement;
mod navigation;
mod spawn;
mod tracking;
mod vision;
mod visuals;

pub use components::*;
pub use spawn::spawn_npc;

use crate::{DeveloperMode, GameState};
use bevy::prelude::*;

pub struct NpcPlugin;

impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, visuals::setup_npc_visual_assets)
            .add_systems(
                Update,
                (
                    spawn::spawn_debug_npc,
                    vision::npc_perception,
                    tracking::npc_tracking,
                    behavior::npc_mode_switching,
                    navigation::npc_navigation,
                    movement::npc_movement,
                    visuals::sync_npc_visuals,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                visuals::attach_npc_visuals.run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                visuals::npc_debug_gizmos
                    .run_if(in_state(GameState::Playing))
                    .run_if(in_state(DeveloperMode(true))),
            );
    }
}
