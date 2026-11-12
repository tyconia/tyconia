//! Grounded NPC movement. Navigation supplies the destination; this module turns
//! it into a smoothed velocity and applies collision + level height sampling.

use super::navigation::{obstacle_polygons, resolve_grounded_step};
use super::*;
use crate::levels::{
    move_and_slide, GroundTransform, IsoGrid, LevelInstance, LevelVertical, ObstacleCollider,
};
use bevy::prelude::*;

pub(super) fn npc_movement(
    time: Res<Time>,
    grid: Option<Res<IsoGrid>>,
    mut npcs: Query<(
        &mut Transform,
        &mut NpcHeight,
        &NpcBody,
        &NpcNavigation,
        &mut NpcVelocity,
        &mut NpcFacing,
        &Parent,
    )>,
    instances: Query<(Entity, &LevelInstance, &LevelVertical)>,
    obstacles: Query<(
        &ObstacleCollider,
        Option<&GroundTransform>,
        &GlobalTransform,
    )>,
) {
    let delta_seconds = time.delta_secs().min(0.05);

    for (mut transform, mut height, body, navigation, mut velocity, mut facing, parent) in &mut npcs
    {
        let Some(destination) = navigation.destination else {
            velocity.0 = Vec2::ZERO;
            continue;
        };

        let current = transform.translation.xy();
        let to_destination = destination - current;
        let distance = to_destination.length();
        let desired = if distance <= navigation.stop_distance {
            Vec2::ZERO
        } else {
            to_destination / distance * body.speed
        };

        let blend = 1.0 - (-body.turn_speed * delta_seconds).exp();
        let current_velocity = velocity.0;
        velocity.0 += (desired - current_velocity) * blend;
        if velocity.0.length_squared() <= 0.25 {
            velocity.0 = Vec2::ZERO;
        }

        if velocity.0 == Vec2::ZERO {
            continue;
        }

        let displacement = velocity.0 * delta_seconds;
        let polygons = obstacle_polygons(obstacles.iter(), height.elevation, body.height);
        let vertical = instances
            .get(parent.get())
            .ok()
            .map(|(_, _, vertical)| vertical);

        let accepted = if let (Some(grid), Some(vertical)) = (grid.as_deref(), vertical) {
            let Some((position, sample)) = resolve_grounded_step(
                grid,
                vertical,
                current,
                displacement,
                height.elevation,
                body.radius,
                &polygons,
            ) else {
                velocity.0 = Vec2::ZERO;
                continue;
            };
            height.elevation = sample.elevation;
            height.rank = sample.rank;
            position
        } else {
            move_and_slide(current, displacement, body.radius, &polygons)
        };

        transform.translation.x = accepted.x;
        transform.translation.y = accepted.y;

        let movement = accepted - current;
        if movement.length_squared() > 0.0001 {
            facing.0 = movement.normalize_or_zero();
        }
    }
}
