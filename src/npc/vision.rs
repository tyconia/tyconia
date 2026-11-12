//! Vision cones and line-of-sight checks.

use super::navigation::obstacle_polygons;
use super::*;
use crate::levels::{point_in_polygon, GroundTransform, ObstacleCollider};
use crate::player::{Player, PlayerHeight};
use bevy::prelude::*;

const EPSILON: f32 = 0.0001;
const MAX_SIGHT_ELEVATION_DELTA: f32 = 1.25;

/// Whether `target` sits inside the NPC's cone.
pub(crate) fn in_vision_cone(origin: Vec2, facing: Vec2, target: Vec2, vision: NpcVision) -> bool {
    let delta = target - origin;
    let distance = delta.length();
    if distance > vision.range || distance <= EPSILON {
        return distance <= vision.range;
    }

    let facing = facing.normalize_or_zero();
    if facing == Vec2::ZERO {
        return true;
    }

    facing.dot(delta / distance) >= vision.half_angle.cos()
}

/// Whether a sight line is interrupted by any obstacle polygon.
pub(crate) fn segment_blocked(start: Vec2, end: Vec2, obstacles: &[Vec<Vec2>]) -> bool {
    obstacles
        .iter()
        .any(|polygon| segment_intersects_polygon(start, end, polygon))
}

fn segment_intersects_polygon(start: Vec2, end: Vec2, vertices: &[Vec2]) -> bool {
    if vertices.len() < 3 {
        return false;
    }
    if point_in_polygon(start, vertices) || point_in_polygon(end, vertices) {
        return true;
    }

    (0..vertices.len()).any(|index| {
        let edge_start = vertices[index];
        let edge_end = vertices[(index + 1) % vertices.len()];
        segments_intersect(start, end, edge_start, edge_end)
    })
}

fn segments_intersect(a: Vec2, b: Vec2, c: Vec2, d: Vec2) -> bool {
    let r = b - a;
    let s = d - c;
    let denominator = r.perp_dot(s);

    if denominator.abs() <= EPSILON {
        if r.perp_dot(c - a).abs() > EPSILON {
            return false;
        }
        return point_on_segment(c, a, b)
            || point_on_segment(d, a, b)
            || point_on_segment(a, c, d)
            || point_on_segment(b, c, d);
    }

    let t = (c - a).perp_dot(s) / denominator;
    let u = (c - a).perp_dot(r) / denominator;
    (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u)
}

fn point_on_segment(point: Vec2, start: Vec2, end: Vec2) -> bool {
    let cross = (end - start).perp_dot(point - start);
    if cross.abs() > EPSILON {
        return false;
    }

    let min = start.min(end) - Vec2::splat(EPSILON);
    let max = start.max(end) + Vec2::splat(EPSILON);
    point.x >= min.x && point.x <= max.x && point.y >= min.y && point.y <= max.y
}

pub(super) fn npc_perception(
    player: Query<(&GlobalTransform, &PlayerHeight), With<Player>>,
    mut npcs: Query<
        (
            &GlobalTransform,
            &NpcHeight,
            &NpcBody,
            &NpcVision,
            &NpcFacing,
            &mut NpcPerception,
        ),
        Without<Player>,
    >,
    obstacles: Query<(
        &ObstacleCollider,
        Option<&GroundTransform>,
        &GlobalTransform,
    )>,
) {
    let player = player.get_single().ok();

    for (transform, height, body, vision, facing, mut perception) in &mut npcs {
        let Some((player_transform, player_height)) = player else {
            *perception = NpcPerception::default();
            continue;
        };

        if (height.elevation - player_height.elevation).abs() > MAX_SIGHT_ELEVATION_DELTA {
            *perception = NpcPerception::default();
            continue;
        }

        let origin = transform.translation().xy();
        let target = player_transform.translation().xy();

        if !in_vision_cone(origin, facing.0, target, *vision) {
            *perception = NpcPerception {
                sees_target: false,
                target: None,
            };
            continue;
        }

        let polygons = obstacle_polygons(obstacles.iter(), height.elevation, body.height);
        let sees_target = !segment_blocked(origin, target, &polygons);

        *perception = NpcPerception {
            sees_target,
            target: sees_target.then_some(target),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vision() -> NpcVision {
        NpcVision {
            range: 100.0,
            half_angle: 45.0_f32.to_radians(),
        }
    }

    fn wall() -> Vec<Vec2> {
        vec![
            Vec2::new(40.0, -20.0),
            Vec2::new(60.0, -20.0),
            Vec2::new(60.0, 20.0),
            Vec2::new(40.0, 20.0),
        ]
    }

    #[test]
    fn vision_cone_checks_range_and_direction() {
        let vision = vision();

        assert!(in_vision_cone(
            Vec2::ZERO,
            Vec2::X,
            Vec2::new(50.0, 0.0),
            vision
        ));
        assert!(!in_vision_cone(
            Vec2::ZERO,
            Vec2::X,
            Vec2::new(-50.0, 0.0),
            vision
        ));
        assert!(!in_vision_cone(
            Vec2::ZERO,
            Vec2::X,
            Vec2::new(0.0, 50.0),
            vision
        ));
        assert!(!in_vision_cone(
            Vec2::ZERO,
            Vec2::X,
            Vec2::new(150.0, 0.0),
            vision
        ));
    }

    #[test]
    fn sight_line_is_stopped_by_polygons() {
        let wall = wall();

        assert!(segment_blocked(
            Vec2::new(0.0, 0.0),
            Vec2::new(100.0, 0.0),
            std::slice::from_ref(&wall)
        ));
        assert!(!segment_blocked(
            Vec2::new(0.0, 80.0),
            Vec2::new(100.0, 80.0),
            std::slice::from_ref(&wall)
        ));
    }
}
