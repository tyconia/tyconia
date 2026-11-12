use super::grid::*;
use super::*;
use std::collections::HashMap;

pub(super) const LEVEL_ACCENT_COUNT: i16 = 6;
const FOCUSED_FLOOR_LIP: f32 = 5.0;

/// Cycles through a small palette so stacked floors stay distinguishable even
/// when their checker patterns and screen offsets happen to line up.
pub(super) fn level_accent(rank: i16) -> Color {
    match rank.rem_euclid(LEVEL_ACCENT_COUNT) {
        0 => Color::srgb(0.26, 0.85, 0.72),
        1 => Color::srgb(0.98, 0.74, 0.28),
        2 => Color::srgb(0.96, 0.42, 0.34),
        3 => Color::srgb(0.42, 0.68, 0.98),
        4 => Color::srgb(0.74, 0.55, 0.98),
        _ => Color::srgb(0.62, 0.9, 0.4),
    }
}

#[derive(Resource, Default)]
pub(super) struct VerticalGizmoCache {
    floors: HashMap<Entity, Vec<CachedFloorGizmos>>,
}

impl VerticalGizmoCache {
    pub(super) fn clear(&mut self) {
        self.floors.clear();
    }
}

#[derive(Clone)]
struct CachedFloorGizmos {
    rank: i16,
    elevation: f32,
    boundaries: Vec<Vec<Vec2>>,
}

pub(super) fn cache_vertical_geometry(
    grid: Res<IsoGrid>,
    state: Res<LevelEditorState>,
    mut cache: ResMut<VerticalGizmoCache>,
    instances: Query<(Entity, &LevelVertical)>,
    changed: Query<(Entity, &LevelVertical), Changed<LevelVertical>>,
    mut removed: RemovedComponents<LevelVertical>,
    mut was_active: Local<bool>,
) {
    for entity in removed.read() {
        cache.floors.remove(&entity);
    }

    if !state.active {
        *was_active = false;
        return;
    }

    let rebuild = grid.is_changed() || !*was_active || cache.floors.is_empty();
    *was_active = true;

    if rebuild {
        cache.floors.clear();
        for (entity, vertical) in &instances {
            cache
                .floors
                .insert(entity, cached_floor_gizmos(&grid, vertical));
        }
    } else {
        for (entity, vertical) in &changed {
            cache
                .floors
                .insert(entity, cached_floor_gizmos(&grid, vertical));
        }
    }
}

fn cached_floor_gizmos(grid: &IsoGrid, vertical: &LevelVertical) -> Vec<CachedFloorGizmos> {
    vertical
        .floors
        .iter()
        .map(|floor| CachedFloorGizmos {
            rank: floor.rank,
            elevation: floor.elevation,
            boundaries: grid.cell_boundary_loops(&floor.cells),
        })
        .collect()
}

pub(super) fn draw_vertical_geometry(
    mut gizmos: Gizmos,
    grid: Res<IsoGrid>,
    state: Res<LevelEditorState>,
    cache: Res<VerticalGizmoCache>,
    instances: Query<(Entity, &LevelVertical, &GlobalTransform)>,
) {
    for (entity, vertical, transform) in &instances {
        let Some(floors) = cache.floors.get(&entity) else {
            continue;
        };
        draw_vertical_gizmos(
            &mut gizmos,
            &grid,
            &state,
            floors,
            &vertical.accesses,
            transform,
        );
    }
}

pub(super) fn editor_rank_alpha(rank: i16, focus: i16) -> f32 {
    match (rank - focus).abs() {
        0 => 1.0,
        1 => 0.45,
        _ => 0.12,
    }
}

fn draw_vertical_gizmos(
    gizmos: &mut Gizmos,
    grid: &IsoGrid,
    state: &LevelEditorState,
    floors: &[CachedFloorGizmos],
    accesses: &[RuntimeAccess],
    transform: &GlobalTransform,
) {
    let access_color = Color::srgba(0.95, 0.7, 0.25, 0.9);

    // Draw the focused floor last so its brighter perimeter and lip sit on top
    // of neighbouring levels.
    for focused_pass in [false, true] {
        for floor in floors
            .iter()
            .filter(|floor| (floor.rank == state.level) == focused_pass)
        {
            let offset = Vec2::new(0.0, elevation_screen_offset(floor.elevation));
            let factor = editor_rank_alpha(floor.rank, state.level);
            let focused = floor.rank == state.level;
            let accent = level_accent(floor.rank);
            let alpha = if focused {
                1.0
            } else if floor.rank < 0 {
                0.2 * factor
            } else {
                0.55 * factor
            };
            let color = accent.with_alpha(alpha);
            let lip_color = accent.with_alpha(alpha * 0.35);

            for boundary in &floor.boundaries {
                let points = boundary
                    .iter()
                    .map(|point| transform_point(*point + offset, transform))
                    .collect::<Vec<_>>();
                if points.is_empty() {
                    continue;
                }

                gizmos.linestrip_2d(
                    points.iter().copied().chain(std::iter::once(points[0])),
                    color,
                );

                if focused {
                    let lip = points
                        .iter()
                        .map(|point| *point - Vec2::Y * FOCUSED_FLOOR_LIP)
                        .collect::<Vec<_>>();
                    gizmos.linestrip_2d(
                        lip.iter().copied().chain(std::iter::once(lip[0])),
                        lip_color,
                    );
                }
            }
        }
    }

    for access in accesses {
        let start = access.from + Vec2::new(0.0, elevation_screen_offset(access.from_elevation));
        let end = access.to + Vec2::new(0.0, elevation_screen_offset(access.to_elevation));
        let side = access_side_direction(grid, access.from, access.to) * access.width * 0.5;
        let rank = access.visual_rank();
        let factor = editor_rank_alpha(rank, state.level);
        let base_alpha = if access.kind == AccessKind::Stairs {
            0.7
        } else {
            0.9
        };
        let base = access_color.with_alpha(base_alpha * factor);
        let is_underground = access.from_elevation < 0.0 || access.to_elevation < 0.0;
        let (color, side_color) = if is_underground {
            (base.with_alpha(0.2 * factor), base.with_alpha(0.1 * factor))
        } else {
            (base, base.with_alpha(0.5 * factor))
        };

        if access.kind == AccessKind::Stairs {
            draw_stair_access_gizmo(gizmos, access, side, color, side_color, transform);
            continue;
        }

        draw_transformed_line(gizmos, transform, start, end, color);
        draw_transformed_line(gizmos, transform, start + side, end + side, side_color);
        draw_transformed_line(gizmos, transform, start - side, end - side, side_color);
        draw_transformed_line(gizmos, transform, start + side, start - side, side_color);
        draw_transformed_line(gizmos, transform, end + side, end - side, side_color);
    }
}

pub(super) fn transform_point(point: Vec2, transform: &GlobalTransform) -> Vec2 {
    transform.transform_point(point.extend(0.0)).truncate()
}

pub(super) fn draw_transformed_line(
    gizmos: &mut Gizmos,
    transform: &GlobalTransform,
    start: Vec2,
    end: Vec2,
    color: Color,
) {
    gizmos.line_2d(
        transform_point(start, transform),
        transform_point(end, transform),
        color,
    );
}

pub(super) fn draw_stair_access_gizmo(
    gizmos: &mut Gizmos,
    access: &RuntimeAccess,
    side: Vec2,
    color: Color,
    side_color: Color,
    transform: &GlobalTransform,
) {
    let (low_ground, high_ground, low_elevation, high_elevation) =
        if access.from_elevation <= access.to_elevation {
            (
                access.from,
                access.to,
                access.from_elevation,
                access.to_elevation,
            )
        } else {
            (
                access.to,
                access.from,
                access.to_elevation,
                access.from_elevation,
            )
        };
    let run = high_ground - low_ground;
    let rise = high_elevation - low_elevation;
    if run.length_squared() <= f32::EPSILON {
        return;
    }

    let steps = stair_step_count(rise.abs() * FLOOR_SCREEN_RISE);
    for index in 0..steps {
        let t0 = index as f32 / steps as f32;
        let t1 = (index + 1) as f32 / steps as f32;
        let ground0 = low_ground + run * t0;
        let ground1 = low_ground + run * t1;
        let elevation0 = low_elevation + rise * t0;
        let elevation1 = low_elevation + rise * t1;
        let top0 = ground0 + Vec2::new(0.0, elevation_screen_offset(elevation1));
        let top1 = ground1 + Vec2::new(0.0, elevation_screen_offset(elevation1));
        let riser0 = ground0 + Vec2::new(0.0, elevation_screen_offset(elevation0));

        draw_transformed_line(gizmos, transform, top0, top1, color);
        draw_transformed_line(gizmos, transform, riser0, top0, color);
        draw_transformed_line(gizmos, transform, top0 + side, top1 + side, side_color);
        draw_transformed_line(gizmos, transform, top0 - side, top1 - side, side_color);
        draw_transformed_line(gizmos, transform, riser0 + side, top0 + side, side_color);
        draw_transformed_line(gizmos, transform, riser0 - side, top0 - side, side_color);
        draw_transformed_line(gizmos, transform, top0 + side, top0 - side, color);
    }

    let low = low_ground + Vec2::new(0.0, elevation_screen_offset(low_elevation));
    let high = high_ground + Vec2::new(0.0, elevation_screen_offset(high_elevation));
    draw_transformed_line(gizmos, transform, low + side, low - side, side_color);
    draw_transformed_line(gizmos, transform, high + side, high - side, color);
}

pub(super) fn visible_grid_bounds(
    grid: &IsoGrid,
    camera_center: Vec2,
    area: Rect,
) -> (IVec2, IVec2) {
    let corners = [
        Vec2::new(area.min.x, area.min.y),
        Vec2::new(area.min.x, area.max.y),
        Vec2::new(area.max.x, area.min.y),
        Vec2::new(area.max.x, area.max.y),
    ];
    let mut min = Vec2::splat(f32::INFINITY);
    let mut max = Vec2::splat(f32::NEG_INFINITY);

    for corner in corners {
        let cell = grid.world_to_cell(camera_center + corner);
        min = min.min(cell);
        max = max.max(cell);
    }

    (
        min.floor().as_ivec2() - IVec2::ONE,
        max.ceil().as_ivec2() + IVec2::ONE,
    )
}

pub(super) fn draw_iso_grid(
    mut gizmos: Gizmos,
    grid: Res<IsoGrid>,
    state: Res<LevelEditorState>,
    cursor: Res<CursorWorldPosition>,
    cameras: Query<(&GlobalTransform, &OrthographicProjection), With<Camera2d>>,
) {
    if !state.grid_visible {
        return;
    }

    let accent = level_accent(state.level);
    let minor = accent.with_alpha(0.2);
    let major = accent.with_alpha(0.4);
    let hover = Color::srgba(0.25, 0.95, 0.85, 0.95);
    let preview = Color::srgba(1.0, 0.75, 0.15, 0.95);
    let elevation = state.level as f32;
    let offset = editor_elevation_offset(elevation);
    let (camera_center, area) = cameras
        .get_single()
        .map(|(transform, projection)| (transform.translation().truncate(), projection.area))
        .unwrap_or_else(|_| {
            (
                cursor.0 - offset,
                Rect::from_center_size(Vec2::ZERO, Vec2::splat(grid.extent as f32 * 32.0)),
            )
        });
    let (min, max) = visible_grid_bounds(&grid, camera_center, area);

    for boundary in min.y..=max.y {
        let color = if boundary % grid.major_every == 0 {
            major
        } else {
            minor
        };
        let boundary = boundary as f32 + 0.5;

        gizmos.line_2d(
            grid.cell_to_world(Vec2::new(min.x as f32 - 0.5, boundary)) + offset,
            grid.cell_to_world(Vec2::new(max.x as f32 + 0.5, boundary)) + offset,
            color,
        );
    }
    for boundary in min.x..=max.x {
        let color = if boundary % grid.major_every == 0 {
            major
        } else {
            minor
        };
        let boundary = boundary as f32 + 0.5;

        gizmos.line_2d(
            grid.cell_to_world(Vec2::new(boundary, min.y as f32 - 0.5)) + offset,
            grid.cell_to_world(Vec2::new(boundary, max.y as f32 + 0.5)) + offset,
            color,
        );
    }

    let (center, half_extents, color) = if state.dragging {
        let (Some(start), Some(end)) = (state.drag_start, state.drag_end) else {
            return;
        };
        let min = start.min(end);
        let max = start.max(end);
        (
            grid.footprint_center(min, max) + offset,
            grid.footprint_half_extents(max - min + IVec2::ONE),
            preview,
        )
    } else {
        (
            grid.cell_to_world(editor_cell_at_elevation(&grid, cursor.0, elevation).as_vec2())
                + offset,
            Vec2::splat(grid.cell_half_extent()),
            hover,
        )
    };

    let corners = iso_corners(half_extents).map(|corner| corner + center);
    gizmos.linestrip_2d(
        corners.into_iter().chain(std::iter::once(corners[0])),
        color,
    );
}
