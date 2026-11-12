//! Navigation turns the current [`super::NpcMode`] into a destination and drives
//! grounded movement toward it.

use super::*;
use crate::levels::{
    move_and_slide, GroundTransform, HeightSample, IsoGrid, LevelInstance, LevelVertical,
    ObstacleCollider, STEP_DOWN, STEP_UP,
};
use bevy::prelude::*;

pub const PATROL_ARRIVAL_DISTANCE: f32 = 10.0;
pub const INVESTIGATE_ARRIVAL_DISTANCE: f32 = 12.0;
pub const CHASE_STOP_DISTANCE: f32 = 26.0;
pub const ROUTE_ARRIVAL_DISTANCE: f32 = 0.0;

/// Pick the next destination from the NPC's mode. Patrol points advance when the
/// NPC arrives, while chase and investigate both use tracked positions. If the
/// destination lives on another floor, the destination is redirected toward an
/// access endpoint first.
pub(super) fn npc_navigation(
    grid: Option<Res<IsoGrid>>,
    instances: Query<(Entity, &LevelInstance, &LevelVertical)>,
    mut npcs: Query<(
        &Transform,
        &NpcMode,
        &NpcHeight,
        &Parent,
        &mut NpcPatrol,
        &NpcMemory,
        &NpcPerception,
        &mut NpcNavigation,
    )>,
) {
    for (transform, mode, height, parent, mut patrol, memory, perception, mut navigation) in
        &mut npcs
    {
        let position = transform.translation.xy();
        let mut destination = None;
        let mut stop_distance = PATROL_ARRIVAL_DISTANCE;

        match *mode {
            NpcMode::Idle => {}
            NpcMode::Patrol => {
                if let Some(point) = patrol.current() {
                    if position.distance(point) <= PATROL_ARRIVAL_DISTANCE {
                        patrol.advance();
                    }
                }
                destination = patrol.current();
            }
            NpcMode::Investigate => {
                destination = memory.last_known;
                stop_distance = INVESTIGATE_ARRIVAL_DISTANCE;
            }
            NpcMode::Chase => {
                destination = perception.target.or(memory.last_known);
                stop_distance = CHASE_STOP_DISTANCE;
            }
        }

        if let (Some(grid), Some(target)) = (grid.as_deref(), destination) {
            if let Ok((_, _, vertical)) = instances.get(parent.get()) {
                if let Some(route) =
                    route_destination(grid, vertical, position, height.elevation, target)
                {
                    destination = Some(route);
                    stop_distance = ROUTE_ARRIVAL_DISTANCE;
                }
            }
        }

        navigation.destination = destination;
        navigation.stop_distance = stop_distance;
    }
}

/// If `destination` is not reachable from the current elevation band, steer to
/// an access that leads toward the target floor instead.
fn route_destination(
    grid: &IsoGrid,
    vertical: &LevelVertical,
    current: Vec2,
    elevation: f32,
    destination: Vec2,
) -> Option<Vec2> {
    if vertical.sample_band(grid, destination, elevation).is_some() {
        return None;
    }

    let target = vertical.sample_any(grid, destination)?;
    if (target.elevation - elevation).abs() <= STEP_UP {
        return None;
    }

    access_route_target(grid, vertical, current, elevation, target.elevation)
}

/// Choose an access endpoint to walk toward so the NPC can change floors.
///
/// When already standing on a relevant access, the far endpoint is returned so
/// the NPC keeps walking up or down the ramp. Otherwise the nearest reachable
/// endpoint of an access heading toward the target elevation is returned.
pub(crate) fn access_route_target(
    grid: &IsoGrid,
    vertical: &LevelVertical,
    current: Vec2,
    elevation: f32,
    target_elevation: f32,
) -> Option<Vec2> {
    if (target_elevation - elevation).abs() <= STEP_UP {
        return None;
    }

    let remaining = (target_elevation - elevation).abs();

    for access in &vertical.accesses {
        if access.height_at(grid, current).is_none() {
            continue;
        }

        let (far, far_elevation) = if (access.from_elevation - elevation).abs()
            <= (access.to_elevation - elevation).abs()
        {
            (access.to, access.to_elevation)
        } else {
            (access.from, access.from_elevation)
        };

        if (target_elevation - far_elevation).abs() < remaining {
            return Some(far);
        }
    }

    let mut best: Option<(f32, Vec2)> = None;
    for access in &vertical.accesses {
        for (near, near_elevation, far_elevation) in [
            (access.from, access.from_elevation, access.to_elevation),
            (access.to, access.to_elevation, access.from_elevation),
        ] {
            if near_elevation > elevation + STEP_UP || near_elevation < elevation - STEP_DOWN {
                continue;
            }
            if (target_elevation - far_elevation).abs() >= remaining {
                continue;
            }

            let distance = current.distance(near);
            if best
                .map(|(best_distance, _)| distance < best_distance)
                .unwrap_or(true)
            {
                best = Some((distance, near));
            }
        }
    }

    best.map(|(_, point)| point)
}

/// Collect obstacle polygons that overlap an NPC's vertical range.
pub(crate) fn obstacle_polygons<'a>(
    obstacles: impl Iterator<
        Item = (
            &'a ObstacleCollider,
            Option<&'a GroundTransform>,
            &'a GlobalTransform,
        ),
    >,
    feet: f32,
    body_height: f32,
) -> Vec<Vec<Vec2>> {
    obstacles
        .filter(|(obstacle, _, _)| obstacle.overlaps_elevation(feet, body_height))
        .map(|(obstacle, ground, transform)| match ground {
            Some(ground) => ground.world_corners(obstacle),
            None => obstacle.world_corners(transform),
        })
        .collect()
}

/// Grounded movement with a few slide fallbacks, so an NPC can still traverse
/// access edges and slide along walls instead of stopping dead.
pub(crate) fn resolve_grounded_step(
    grid: &IsoGrid,
    vertical: &LevelVertical,
    current: Vec2,
    displacement: Vec2,
    feet: f32,
    radius: f32,
    obstacles: &[Vec<Vec2>],
) -> Option<(Vec2, HeightSample)> {
    let intended = displacement.normalize_or_zero();
    let mut best: Option<(f32, Vec2, HeightSample)> = None;

    for candidate in
        std::iter::once(displacement).chain(slide_candidates(grid, vertical, current, displacement))
    {
        let moved = move_and_slide(current, candidate, radius, obstacles);
        let progress = (moved - current).dot(intended);
        if progress <= f32::EPSILON {
            continue;
        }
        let Some(sample) = vertical.sample(grid, moved, feet) else {
            continue;
        };

        if best
            .map(|(best_progress, _, _)| progress > best_progress)
            .unwrap_or(true)
        {
            best = Some((progress, moved, sample));
        }
    }

    best.map(|(_, position, sample)| (position, sample))
}

fn slide_candidates(
    grid: &IsoGrid,
    vertical: &LevelVertical,
    current: Vec2,
    displacement: Vec2,
) -> Vec<Vec2> {
    let mut vectors = vec![
        Vec2::new(displacement.x, 0.0),
        Vec2::new(0.0, displacement.y),
        displacement * 0.5,
    ];

    for direction in [grid.step_u(), grid.step_v()] {
        let direction = direction.normalize_or_zero();
        let amount = displacement.dot(direction);
        if amount.abs() > f32::EPSILON {
            vectors.push(direction * amount);
        }
    }

    for access in &vertical.accesses {
        if access.height_at(grid, current).is_none() {
            continue;
        }
        let direction = (access.to - access.from).normalize_or_zero();
        let amount = displacement.dot(direction);
        if amount.abs() > f32::EPSILON {
            vectors.push(direction * amount);
        }
    }

    vectors
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::levels::{access_side_direction, AccessDeclared, AccessKind, FloorDeclared};

    fn vertical_with_ramp(grid: &IsoGrid) -> (Vec2, Vec2, LevelVertical) {
        let from = grid.cell_to_world(Vec2::ZERO);
        let to = grid.cell_to_world(Vec2::new(3.0, 0.0));
        let vertical = LevelVertical::from_parts(
            vec![
                FloorDeclared {
                    elevation: 0.0,
                    rank: 0,
                    cells: vec![IVec2::new(0, 0)],
                },
                FloorDeclared {
                    elevation: 1.0,
                    rank: 1,
                    cells: vec![IVec2::new(3, 0)],
                },
            ],
            vec![AccessDeclared {
                from,
                to,
                width: grid.step_u().length(),
                from_elevation: 0.0,
                to_elevation: 1.0,
                kind: AccessKind::Ramp,
            }],
        );
        (from, to, vertical)
    }

    #[test]
    fn access_routing_approaches_the_lower_end_from_the_floor() {
        let grid = IsoGrid::default();
        let (from, _, vertical) = vertical_with_ramp(&grid);
        let current = grid.cell_to_world(Vec2::new(-2.0, 0.0));

        let route = access_route_target(&grid, &vertical, current, 0.0, 1.0);

        assert_eq!(route, Some(from));
    }

    #[test]
    fn access_routing_walks_to_the_far_end_while_on_a_ramp() {
        let grid = IsoGrid::default();
        let (from, to, vertical) = vertical_with_ramp(&grid);
        let midpoint = (from + to) * 0.5;

        let route = access_route_target(&grid, &vertical, midpoint, 0.5, 1.0);

        assert_eq!(route, Some(to));
    }

    #[test]
    fn access_routing_is_skipped_when_the_target_is_on_the_same_band() {
        let grid = IsoGrid::default();
        let (_, _, vertical) = vertical_with_ramp(&grid);
        let current = grid.cell_to_world(Vec2::new(-2.0, 0.0));

        let route = access_route_target(&grid, &vertical, current, 0.0, 0.15);

        assert_eq!(route, None);
    }

    #[test]
    fn route_destination_redirects_to_an_access_when_floors_differ() {
        let grid = IsoGrid::default();
        let (from, to, vertical) = vertical_with_ramp(&grid);
        let current = grid.cell_to_world(Vec2::new(-2.0, 0.0));

        let route = route_destination(&grid, &vertical, current, 0.0, to);

        assert_eq!(route, Some(from));
    }

    #[test]
    fn route_destination_keeps_direct_paths_on_the_same_floor() {
        let grid = IsoGrid::default();
        let (_, _, vertical) = vertical_with_ramp(&grid);
        let current = grid.cell_to_world(Vec2::ZERO);
        let destination = grid.cell_to_world(Vec2::ZERO);

        let route = route_destination(&grid, &vertical, current, 0.0, destination);

        assert_eq!(route, None);
    }

    #[test]
    fn slide_candidates_include_access_direction() {
        let grid = IsoGrid::default();
        let from = grid.cell_to_world(Vec2::ZERO);
        let to = grid.cell_to_world(Vec2::new(1.0, 0.0));
        let vertical = LevelVertical::from_parts(
            Vec::new(),
            vec![AccessDeclared {
                from,
                to,
                width: grid.step_u().length(),
                from_elevation: 0.0,
                to_elevation: 1.0,
                kind: AccessKind::Stairs,
            }],
        );
        let current = (from + to) * 0.5;
        let run = (to - from).normalize();
        let side = access_side_direction(&grid, from, to);
        let vectors = slide_candidates(&grid, &vertical, current, run * 3.0 + side * 10.0);

        assert!(vectors
            .iter()
            .any(|vector| vector.normalize_or_zero().dot(run).abs() > 0.999));
    }

    #[test]
    fn grounded_step_follows_a_ramp() {
        let grid = IsoGrid::default();
        let from = grid.cell_to_world(Vec2::ZERO);
        let to = grid.cell_to_world(Vec2::new(1.0, 0.0));
        let vertical = LevelVertical::from_parts(
            Vec::new(),
            vec![AccessDeclared {
                from,
                to,
                width: grid.step_u().length(),
                from_elevation: 0.0,
                to_elevation: 1.0,
                kind: AccessKind::Ramp,
            }],
        );

        let (position, sample) =
            resolve_grounded_step(&grid, &vertical, from, (to - from) * 0.1, 0.0, 8.0, &[])
                .expect("the ramp should accept movement");

        assert!(position.distance(from) > 1.0);
        assert!(sample.elevation > 0.0);
    }
}
