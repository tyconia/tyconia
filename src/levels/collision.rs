use crate::levels::{ColliderDeclared, OcclusionRole};
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Component, Debug, Clone)]
pub struct ObstacleCollider {
    pub vertices: Vec<Vec2>,
    pub screen_space: bool,
    pub z_min: f32,
    pub height: f32,
    pub role: OcclusionRole,
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct GroundTransform {
    pub position: Vec2,
    pub rotation: f32,
}

impl GroundTransform {
    pub fn from_declared(collider: &ColliderDeclared) -> Self {
        Self {
            position: collider.position,
            rotation: collider.rotation_degrees.to_radians(),
        }
    }

    pub fn world_corners(&self, collider: &ObstacleCollider) -> Vec<Vec2> {
        collider
            .vertices
            .iter()
            .map(|vertex| vertex.rotate(Vec2::from_angle(self.rotation)) + self.position)
            .collect()
    }
}

impl ObstacleCollider {
    pub fn iso(half_extents: Vec2) -> Self {
        Self {
            vertices: iso_corners(half_extents).to_vec(),
            screen_space: false,
            z_min: 0.0,
            height: crate::levels::default_collider_height(),
            role: OcclusionRole::Facade,
        }
    }

    pub fn rect(half_extents: Vec2) -> Self {
        Self {
            vertices: rect_corners(half_extents).to_vec(),
            screen_space: true,
            z_min: 0.0,
            height: crate::levels::default_collider_height(),
            role: OcclusionRole::Facade,
        }
    }

    pub fn from_declared(collider: &ColliderDeclared) -> Self {
        let vertices = if collider.vertices.len() >= 3 {
            collider.vertices.clone()
        } else if collider.screen_space {
            rect_corners(collider.half_extents).to_vec()
        } else {
            iso_corners(collider.half_extents).to_vec()
        };

        Self {
            vertices: order_ccw_vec(vertices),
            screen_space: collider.screen_space,
            z_min: collider.z_min,
            height: collider.height,
            role: collider.role,
        }
    }

    pub fn z_max(&self) -> f32 {
        self.z_min + self.height.max(0.01)
    }

    pub fn overlaps_elevation(&self, feet: f32, body_height: f32) -> bool {
        self.z_min < feet + body_height && self.z_max() > feet
    }

    pub fn world_corners(&self, transform: &GlobalTransform) -> Vec<Vec2> {
        self.vertices
            .iter()
            .map(|corner| transform.transform_point(corner.extend(0.0)).truncate())
            .collect()
    }
}

fn order_ccw_vec(mut vertices: Vec<Vec2>) -> Vec<Vec2> {
    if polygon_signed_area(&vertices) < 0.0 {
        vertices.reverse();
    }

    vertices
}

pub fn iso_corners(half_extents: Vec2) -> [Vec2; 4] {
    let u = Vec2::new(2.0, 1.0).normalize() * half_extents.x;
    let v = Vec2::new(2.0, -1.0).normalize() * half_extents.y;
    order_ccw([u + v, u - v, -u - v, -u + v])
}

pub fn rect_corners(half_extents: Vec2) -> [Vec2; 4] {
    let u = Vec2::X * half_extents.x;
    let v = Vec2::Y * half_extents.y;
    order_ccw([u + v, u - v, -u - v, -u + v])
}

fn order_ccw(mut corners: [Vec2; 4]) -> [Vec2; 4] {
    let area: f32 = (0..4)
        .map(|index| {
            let a = corners[index];
            let b = corners[(index + 1) % 4];
            a.x * b.y - b.x * a.y
        })
        .sum();

    if area < 0.0 {
        corners.reverse();
    }

    corners
}

pub fn draw_obstacle_collider_gizmos(
    mut gizmos: Gizmos,
    colliders: Query<(&GlobalTransform, &ObstacleCollider)>,
    mut edges: Local<HashMap<((i32, i32), (i32, i32), i32, i32), (u32, Vec2, Vec2, Color)>>,
) {
    edges.clear();

    for (transform, collider) in &colliders {
        let color = if collider.screen_space {
            Color::srgba(0.2, 0.7, 1.0, 0.9)
        } else {
            Color::srgba(1.0, 0.35, 0.1, 0.9)
        };
        let corners = collider.world_corners(transform);
        if corners.len() < 2 {
            continue;
        }

        for index in 0..corners.len() {
            let start = corners[index];
            let end = corners[(index + 1) % corners.len()];
            let key = collider_edge_key(start, end, collider);
            let entry = edges.entry(key).or_insert((0, start, end, color));
            entry.0 += 1;
        }
    }

    for (count, start, end, color) in edges.values() {
        if *count == 1 {
            gizmos.line_2d(*start, *end, *color);
        }
    }
}

fn collider_edge_key(
    start: Vec2,
    end: Vec2,
    collider: &ObstacleCollider,
) -> ((i32, i32), (i32, i32), i32, i32) {
    let quantize = |point: Vec2| {
        (
            (point.x * 1000.0).round() as i32,
            (point.y * 1000.0).round() as i32,
        )
    };
    let start = quantize(start);
    let end = quantize(end);
    let (start, end) = if start <= end {
        (start, end)
    } else {
        (end, start)
    };
    (
        start,
        end,
        (collider.z_min * 1000.0).round() as i32,
        (collider.height * 1000.0).round() as i32,
    )
}

pub fn move_and_slide(
    mut center: Vec2,
    displacement: Vec2,
    radius: f32,
    obstacles: &[Vec<Vec2>],
) -> Vec2 {
    if displacement.length_squared() <= f32::EPSILON || obstacles.is_empty() {
        return center + displacement;
    }

    let max_step = (radius * 0.5).max(1.0);
    let steps = (displacement.length() / max_step).ceil().max(1.0) as usize;
    let step = displacement / steps as f32;

    for _ in 0..steps {
        center += step;

        for _ in 0..4 {
            let mut pushed = false;
            for obstacle in obstacles {
                if let Some(push) = circle_polygon_penetration(center, radius, obstacle) {
                    center += push;
                    pushed = true;
                }
            }

            if !pushed {
                break;
            }
        }
    }

    center
}

pub fn circle_polygon_penetration(center: Vec2, radius: f32, vertices: &[Vec2]) -> Option<Vec2> {
    if vertices.len() < 3 {
        return None;
    }

    let inside = point_in_polygon(center, vertices);
    let mut closest = vertices[0];
    let mut closest_distance_squared = f32::INFINITY;
    let mut closest_outward_normal = Vec2::ZERO;

    for index in 0..vertices.len() {
        let start = vertices[index];
        let end = vertices[(index + 1) % vertices.len()];
        let edge = end - start;
        let edge_length_squared = edge.length_squared();
        if edge_length_squared <= f32::EPSILON {
            continue;
        }

        let relative = center - start;
        let t = (relative.dot(edge) / edge_length_squared).clamp(0.0, 1.0);
        let point = start + edge * t;
        let distance_squared = center.distance_squared(point);

        if distance_squared < closest_distance_squared {
            closest_distance_squared = distance_squared;
            closest = point;
            closest_outward_normal = Vec2::new(edge.y, -edge.x).normalize_or_zero();
        }
    }

    let closest_distance = closest_distance_squared.sqrt();
    let radius_squared = radius * radius;

    if inside {
        if closest_distance >= radius {
            return None;
        }

        let direction = if closest_distance > f32::EPSILON {
            (closest - center) / closest_distance
        } else {
            closest_outward_normal
        };

        return Some(direction * (closest_distance + radius));
    }

    if closest_distance_squared >= radius_squared {
        return None;
    }

    let delta = center - closest;
    if closest_distance > f32::EPSILON {
        Some(delta / closest_distance * (radius - closest_distance))
    } else {
        Some(closest_outward_normal * radius)
    }
}

pub fn polygon_signed_area(vertices: &[Vec2]) -> f32 {
    if vertices.len() < 3 {
        return 0.0;
    }

    let double_area: f32 = (0..vertices.len())
        .map(|index| {
            let a = vertices[index];
            let b = vertices[(index + 1) % vertices.len()];
            a.x * b.y - b.x * a.y
        })
        .sum();

    double_area * 0.5
}

pub fn point_in_polygon(point: Vec2, vertices: &[Vec2]) -> bool {
    if vertices.len() < 3 {
        return false;
    }

    let mut inside = false;
    let mut previous = vertices[vertices.len() - 1];

    for &current in vertices {
        let crosses_y = (current.y > point.y) != (previous.y > point.y);
        if crosses_y {
            let x = (previous.x - current.x) * (point.y - current.y) / (previous.y - current.y)
                + current.x;
            if point.x < x {
                inside = !inside;
            }
        }
        previous = current;
    }

    inside
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wall(center: Vec2, half_extents: Vec2) -> Vec<Vec2> {
        rect_corners(half_extents)
            .map(|corner| corner + center)
            .to_vec()
    }

    #[test]
    fn circle_stops_at_wall() {
        let obstacles = [wall(Vec2::ZERO, Vec2::new(10.0, 100.0))];

        let result = move_and_slide(
            Vec2::new(-50.0, 0.0),
            Vec2::new(100.0, 0.0),
            5.0,
            &obstacles,
        );

        assert!((result.x - -15.0).abs() < 0.01, "x was {}", result.x);
        assert!(result.y.abs() < 0.01, "y was {}", result.y);
    }

    #[test]
    fn circle_slides_along_wall() {
        let obstacles = [wall(Vec2::ZERO, Vec2::new(10.0, 100.0))];

        let result = move_and_slide(
            Vec2::new(-50.0, 50.0),
            Vec2::new(100.0, -50.0),
            5.0,
            &obstacles,
        );

        assert!((result.x - -15.0).abs() < 0.01, "x was {}", result.x);
        assert!((result.y - 0.0).abs() < 0.01, "y was {}", result.y);
    }

    #[test]
    fn circle_slides_along_rotated_wall() {
        let obstacle = rect_corners(Vec2::new(10.0, 100.0))
            .map(|corner| corner.rotate(Vec2::from_angle(std::f32::consts::FRAC_PI_8)))
            .to_vec();

        let result = move_and_slide(
            Vec2::new(-50.0, -20.0),
            Vec2::new(100.0, 0.0),
            5.0,
            std::slice::from_ref(&obstacle),
        );

        assert!(
            circle_polygon_penetration(result, 5.0, &obstacle).is_none(),
            "player stayed inside the wall: {:?}",
            result
        );
        assert!(result.y < -20.0, "player did not slide: {:?}", result);
    }

    fn concave_l_shape() -> Vec<Vec2> {
        order_ccw_vec(vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(2.0, 0.0),
            Vec2::new(2.0, 1.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(1.0, 2.0),
            Vec2::new(0.0, 2.0),
        ])
    }

    #[test]
    fn circle_slides_across_adjacent_wall_segments() {
        let obstacles = [
            wall(Vec2::new(0.0, 50.0), Vec2::new(10.0, 50.0)),
            wall(Vec2::new(0.0, -50.0), Vec2::new(10.0, 50.0)),
        ];

        let result = move_and_slide(
            Vec2::new(-15.0, -20.0),
            Vec2::new(0.0, 100.0),
            5.0,
            &obstacles,
        );

        assert!((result.x - -15.0).abs() < 0.01, "x was {}", result.x);
        assert!((result.y - 80.0).abs() < 0.01, "y was {}", result.y);
    }

    #[test]
    fn concave_notch_is_outside_the_polygon() {
        let obstacle = concave_l_shape();

        assert!(point_in_polygon(Vec2::new(0.5, 0.5), &obstacle));
        assert!(!point_in_polygon(Vec2::new(1.5, 1.5), &obstacle));
        assert!(
            circle_polygon_penetration(Vec2::new(1.5, 1.5), 0.2, &obstacle).is_none(),
            "concave notch should be empty space"
        );
    }

    #[test]
    fn circle_moves_through_concave_notch_and_stops_at_arm() {
        let obstacles = [concave_l_shape()];
        let mut center = Vec2::new(2.0, 1.5);

        for _ in 0..20 {
            center = move_and_slide(center, Vec2::new(-0.1, 0.0), 0.25, &obstacles);
        }

        assert!((center.x - 1.25).abs() < 0.05, "x was {}", center.x);
        assert!((center.y - 1.5).abs() < 0.05, "y was {}", center.y);
    }

    #[test]
    fn collider_edge_key_ignores_direction_and_separates_vertical_range() {
        let mut collider = ObstacleCollider::iso(Vec2::splat(10.0));
        let start = Vec2::new(2.0, 3.0);
        let end = Vec2::new(-1.0, 4.0);

        let forward = collider_edge_key(start, end, &collider);
        let backward = collider_edge_key(end, start, &collider);
        assert_eq!(forward, backward);

        collider.z_min = 1.0;
        assert_ne!(forward, collider_edge_key(start, end, &collider));
    }

    #[test]
    fn iso_footprint_is_diamond_shaped() {
        let corners = iso_corners(Vec2::splat(10.0));
        let width = corners
            .iter()
            .map(|corner| corner.x)
            .fold(f32::NEG_INFINITY, f32::max)
            - corners
                .iter()
                .map(|corner| corner.x)
                .fold(f32::INFINITY, f32::min);
        let height = corners
            .iter()
            .map(|corner| corner.y)
            .fold(f32::NEG_INFINITY, f32::max)
            - corners
                .iter()
                .map(|corner| corner.y)
                .fold(f32::INFINITY, f32::min);

        assert!(width > height * 1.9);
        assert!(width > 35.0);
    }
}
