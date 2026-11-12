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

    let (position, elevation) = grid
        .as_deref()
        .map(|grid| debug_spawn_point(vertical, grid, cursor.0))
        .unwrap_or((cursor.0, 0.0));

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

/// Find the floor or access under the cursor. Cursor coordinates are in screen
/// space, so each candidate elevation is removed before snapping to the grid.
/// This lets the developer spawn guards on upper floors and ramps.
fn debug_spawn_point(vertical: &LevelVertical, grid: &IsoGrid, cursor: Vec2) -> (Vec2, f32) {
    let mut best: Option<(Vec2, f32)> = None;

    for floor in &vertical.floors {
        let position = cursor - Vec2::Y * elevation_screen_offset(floor.elevation);
        if !floor.cells.contains(&grid.snap_cell(position)) {
            continue;
        }
        if best
            .map(|(_, elevation)| floor.elevation > elevation)
            .unwrap_or(true)
        {
            best = Some((position, floor.elevation));
        }
    }

    if let Some(found) = best {
        return found;
    }

    for access in &vertical.accesses {
        for seed in [access.from_elevation, access.to_elevation] {
            let mut position = cursor - Vec2::Y * elevation_screen_offset(seed);
            let mut elevation = seed;

            for _ in 0..4 {
                let Some(height) = access.height_at(grid, position) else {
                    break;
                };
                elevation = height;
                position = cursor - Vec2::Y * elevation_screen_offset(height);
            }

            let Some(height) = access.height_at(grid, position) else {
                continue;
            };
            if (height - elevation).abs() > 0.1 {
                continue;
            }

            if best
                .map(|(_, elevation)| height > elevation)
                .unwrap_or(true)
            {
                best = Some((position, height));
            }
        }
    }

    best.unwrap_or((cursor, 0.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::levels::FloorDeclared;

    fn floor(elevation: f32, rank: i16, cells: &[IVec2]) -> FloorDeclared {
        FloorDeclared {
            elevation,
            rank,
            cells: cells.to_vec(),
        }
    }

    #[test]
    fn debug_spawn_picks_an_elevated_floor_under_the_cursor() {
        let grid = IsoGrid::default();
        let vertical = LevelVertical::from_parts(
            vec![
                floor(0.0, 0, &[IVec2::new(0, 0)]),
                floor(2.0, 2, &[IVec2::new(2, 0)]),
            ],
            Vec::new(),
        );
        let ground = grid.cell_to_world(Vec2::new(2.0, 0.0));
        let cursor = ground + Vec2::Y * elevation_screen_offset(2.0);

        let (position, elevation) = debug_spawn_point(&vertical, &grid, cursor);

        assert!((elevation - 2.0).abs() < 0.001);
        assert!(position.distance(ground) < 0.001);
    }

    #[test]
    fn debug_spawn_uses_the_elevated_floor_when_floors_stack() {
        let grid = IsoGrid::default();
        let vertical = LevelVertical::from_parts(
            vec![
                floor(0.0, 0, &[IVec2::new(1, 1)]),
                floor(1.0, 1, &[IVec2::new(1, 1)]),
            ],
            Vec::new(),
        );
        let ground = grid.cell_to_world(Vec2::new(1.0, 1.0));

        let (_, elevation) = debug_spawn_point(
            &vertical,
            &grid,
            ground + Vec2::Y * elevation_screen_offset(1.0),
        );

        assert!((elevation - 1.0).abs() < 0.001);
    }

    #[test]
    fn debug_spawn_falls_back_to_ground_without_a_floor() {
        let grid = IsoGrid::default();
        let vertical = LevelVertical::from_parts(Vec::new(), Vec::new());
        let cursor = Vec2::new(120.0, -40.0);

        let (position, elevation) = debug_spawn_point(&vertical, &grid, cursor);

        assert_eq!(position, cursor);
        assert_eq!(elevation, 0.0);
    }
}
