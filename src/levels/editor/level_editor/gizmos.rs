use super::grid::*;
use super::*;

pub(super) fn draw_vertical_geometry(
    mut gizmos: Gizmos,
    grid: Res<IsoGrid>,
    state: Res<LevelEditorState>,
    instances: Query<(&LevelVertical, &GlobalTransform)>,
) {
    for (vertical, transform) in &instances {
        draw_vertical_gizmos(&mut gizmos, &grid, &state, vertical, transform);
    }
}

pub(super) fn editor_rank_alpha(rank: i16, focus: i16) -> f32 {
    match (rank - focus).abs() {
        0 => 1.0,
        1 => 0.45,
        _ => 0.12,
    }
}

pub(super) fn draw_vertical_gizmos(
    gizmos: &mut Gizmos,
    grid: &IsoGrid,
    state: &LevelEditorState,
    vertical: &LevelVertical,
    transform: &GlobalTransform,
) {
    let floor_color = Color::srgba(0.35, 0.9, 0.65, 0.55);
    let access_color = Color::srgba(0.95, 0.7, 0.25, 0.9);

    for floor in &vertical.floors {
        let offset = Vec2::new(0.0, elevation_screen_offset(floor.elevation));
        let factor = editor_rank_alpha(floor.rank, state.level);
        let color = if floor.rank < 0 {
            floor_color.with_alpha(0.2 * factor)
        } else {
            floor_color.with_alpha(0.55 * factor)
        };

        for boundary in grid.cell_boundary_loops(&floor.cells) {
            let points = boundary
                .iter()
                .map(|point| transform_point(*point + offset, transform))
                .collect::<Vec<_>>();
            gizmos.linestrip_2d(
                points.iter().copied().chain(std::iter::once(points[0])),
                color,
            );
        }
    }

    for access in &vertical.accesses {
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

pub(super) fn draw_iso_grid(
    mut gizmos: Gizmos,
    grid: Res<IsoGrid>,
    state: Res<LevelEditorState>,
    cursor: Res<CursorWorldPosition>,
) {
    if !state.grid_visible {
        return;
    }

    let minor = Color::srgba(0.65, 0.75, 0.85, 0.22);
    let major = Color::srgba(0.65, 0.75, 0.85, 0.42);
    let hover = Color::srgba(0.25, 0.95, 0.85, 0.95);
    let preview = Color::srgba(1.0, 0.75, 0.15, 0.95);
    let extent = grid.extent as f32;

    for boundary in -grid.extent..=grid.extent {
        let color = if boundary % grid.major_every == 0 {
            major
        } else {
            minor
        };
        let boundary = boundary as f32 + 0.5;

        gizmos.line_2d(
            grid.cell_to_world(Vec2::new(-extent - 0.5, boundary)),
            grid.cell_to_world(Vec2::new(extent + 0.5, boundary)),
            color,
        );
        gizmos.line_2d(
            grid.cell_to_world(Vec2::new(boundary, -extent - 0.5)),
            grid.cell_to_world(Vec2::new(boundary, extent + 0.5)),
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
            grid.footprint_center(min, max),
            grid.footprint_half_extents(max - min + IVec2::ONE),
            preview,
        )
    } else {
        (
            grid.cell_to_world(grid.snap_cell(cursor.0).as_vec2()),
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
