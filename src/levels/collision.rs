use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy)]
pub struct ObstacleCollider {
    pub corners: [Vec2; 4],
    pub screen_space: bool,
}

impl ObstacleCollider {
    pub fn iso(half_extents: Vec2) -> Self {
        Self {
            corners: iso_corners(half_extents),
            screen_space: false,
        }
    }

    pub fn rect(half_extents: Vec2) -> Self {
        Self {
            corners: rect_corners(half_extents),
            screen_space: true,
        }
    }

    pub fn world_corners(&self, transform: &GlobalTransform) -> [Vec2; 4] {
        self.corners
            .map(|corner| transform.transform_point(corner.extend(0.0)).truncate())
    }
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
) {
    for (transform, collider) in &colliders {
        let color = if collider.screen_space {
            Color::srgba(0.2, 0.7, 1.0, 0.9)
        } else {
            Color::srgba(1.0, 0.35, 0.1, 0.9)
        };
        let corners = collider.world_corners(transform);
        gizmos.linestrip_2d(
            corners.into_iter().chain(std::iter::once(corners[0])),
            color,
        );
    }
}

pub fn move_and_slide(
    mut center: Vec2,
    displacement: Vec2,
    radius: f32,
    obstacles: &[[Vec2; 4]],
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

pub fn circle_polygon_penetration(
    center: Vec2,
    radius: f32,
    corners: &[Vec2; 4],
) -> Option<Vec2> {
    let mut inside = true;
    let mut closest = corners[0];
    let mut closest_distance_squared = f32::INFINITY;
    let mut edge_distance = f32::INFINITY;
    let mut outward_normal = Vec2::ZERO;

    for index in 0..4 {
        let start = corners[index];
        let end = corners[(index + 1) % 4];
        let edge = end - start;
        let edge_length_squared = edge.length_squared();
        if edge_length_squared <= f32::EPSILON {
            continue;
        }

        let relative = center - start;
        let cross = edge.perp_dot(relative);
        if cross < 0.0 {
            inside = false;
        }

        let edge_length = edge_length_squared.sqrt();
        let distance_to_edge = cross / edge_length;
        if distance_to_edge < edge_distance {
            edge_distance = distance_to_edge;
            outward_normal = Vec2::new(edge.y, -edge.x) / edge_length;
        }

        let t = (relative.dot(edge) / edge_length_squared).clamp(0.0, 1.0);
        let point = start + edge * t;
        let distance_squared = center.distance_squared(point);
        if distance_squared < closest_distance_squared {
            closest_distance_squared = distance_squared;
            closest = point;
        }
    }

    let radius_squared = radius * radius;

    if inside {
        if edge_distance >= radius {
            return None;
        }

        return Some(outward_normal * (edge_distance + radius));
    }

    if closest_distance_squared >= radius_squared {
        return None;
    }

    let delta = center - closest;
    let distance = closest_distance_squared.sqrt();
    if distance > f32::EPSILON {
        Some(delta / distance * (radius - distance))
    } else {
        Some(outward_normal * radius)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wall(center: Vec2, half_extents: Vec2) -> [Vec2; 4] {
        rect_corners(half_extents).map(|corner| corner + center)
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
            .map(|corner| corner.rotate(Vec2::from_angle(std::f32::consts::FRAC_PI_8)));

        let result = move_and_slide(
            Vec2::new(-50.0, -20.0),
            Vec2::new(100.0, 0.0),
            5.0,
            &[obstacle],
        );

        assert!(
            circle_polygon_penetration(result, 5.0, &obstacle).is_none(),
            "player stayed inside the wall: {:?}",
            result
        );
        assert!(result.y < -20.0, "player did not slide: {:?}", result);
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
