//! NPC creation. Declarations become children of their owning level instance so
//! they are unloaded together with that level.

use super::*;
use crate::actions::CursorWorldPosition;
use crate::levels::{
    elevation_screen_offset, level_z, rank_from_elevation, CurrentLevel, IsoGrid, LevelInstance,
    LevelVertical, NpcKind, RankedVisual, VisualFade, ACTOR_Z_LAYER,
};
use bevy::prelude::*;

/// Spawn a fully initialized NPC under `parent`.
///
/// The root entity stores world position and height; the child visual stores the
/// screen-space elevation offset, mirroring how the player is rendered.
pub fn spawn_npc(
    commands: &mut Commands,
    parent: Entity,
    position: Vec2,
    elevation: f32,
    patrol: Vec<Vec2>,
    kind: NpcKind,
) -> Entity {
    let rank = rank_from_elevation(elevation);
    let body = NpcBody::for_kind(kind);
    let mode = if patrol.is_empty() {
        NpcMode::Idle
    } else {
        NpcMode::Patrol
    };

    let npc = commands
        .spawn((
            Npc { kind },
            body,
            NpcHeight { elevation, rank },
            mode,
            NpcPatrol::new(patrol),
            NpcVision::for_kind(kind),
            NpcPerception::default(),
            NpcMemory::default(),
            NpcNavigation::default(),
            NpcVelocity::default(),
            NpcFacing::default(),
            Transform::from_xyz(
                position.x,
                position.y,
                level_z(rank, ACTOR_Z_LAYER, position.y),
            ),
        ))
        .id();

    commands.entity(parent).add_child(npc);

    let visual = commands
        .spawn((
            NpcVisual { kind },
            RankedVisual { rank },
            VisualFade { alpha: 1.0 },
            Transform::from_xyz(0.0, elevation_screen_offset(elevation), 0.0),
            Visibility::default(),
        ))
        .id();
    commands.entity(npc).add_child(visual);

    npc
}

/// Developer helper: press `N` to place a guard at the cursor on the current level.
pub(super) fn spawn_debug_npc(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    cursor: Res<CursorWorldPosition>,
    grid: Option<Res<IsoGrid>>,
    current: Option<Res<CurrentLevel>>,
    instances: Query<(Entity, &LevelInstance, &LevelVertical)>,
) {
    if !keys.just_pressed(KeyCode::KeyN) {
        return;
    }

    let Some(current) = current else {
        return;
    };
    let Some(label) = current.0.as_ref() else {
        return;
    };
    let Some((instance, _, vertical)) = instances
        .iter()
        .find(|(_, instance, _)| &instance.prefab == label)
    else {
        return;
    };

    let position = cursor.0;
    let elevation = grid
        .as_deref()
        .map(|grid| debug_spawn_elevation(vertical, grid, position))
        .unwrap_or(0.0);

    spawn_npc(
        &mut commands,
        instance,
        position,
        elevation,
        vec![
            position + Vec2::new(64.0, 0.0),
            position + Vec2::new(0.0, 64.0),
            position + Vec2::new(-64.0, 0.0),
            position + Vec2::new(0.0, -64.0),
        ],
        NpcKind::Guard,
    );
}

/// Prefer the highest floor under the cursor so debugging works on upper levels.
fn debug_spawn_elevation(vertical: &LevelVertical, grid: &IsoGrid, position: Vec2) -> f32 {
    let cell = grid.snap_cell(position);
    vertical
        .floors
        .iter()
        .filter(|floor| floor.cells.contains(&cell))
        .map(|floor| floor.elevation)
        .max_by(f32::total_cmp)
        .or_else(|| {
            vertical
                .accesses
                .iter()
                .find_map(|access| access.height_at(grid, position))
        })
        .or_else(|| {
            vertical
                .sample(grid, position, 0.0)
                .map(|sample| sample.elevation)
        })
        .unwrap_or(0.0)
}
