use super::authoring::*;
use super::grid::*;
use super::*;

fn grid() -> IsoGrid {
    IsoGrid::default()
}

#[test]
fn cell_world_roundtrip() {
    let grid = grid();

    for cell in [
        IVec2::new(0, 0),
        IVec2::new(1, 0),
        IVec2::new(0, 1),
        IVec2::new(-4, 7),
        IVec2::new(12, -3),
    ] {
        let world = grid.cell_to_world(cell.as_vec2());
        assert_eq!(grid.snap_cell(world), cell);
    }
}

#[test]
fn single_cell_is_diamond_sized_to_grid() {
    let grid = grid();
    let corners = iso_corners(Vec2::splat(grid.cell_half_extent()));
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

    assert!((width - grid.cell_size.x).abs() < 0.01);
    assert!((height - grid.cell_size.y).abs() < 0.01);
}

#[test]
fn editor_session_clears_stale_inventory_and_drag_state() {
    let mut state = LevelEditorState {
        active: false,
        grid_visible: false,
        dragging: true,
        drag_start: Some(IVec2::new(1, 2)),
        drag_end: Some(IVec2::new(3, 4)),
        previous_inventory_active: Some(Some(0)),
        tool: EditorTool::Floor,
        level: 2,
    };

    state.begin_session(true);
    assert!(state.active);
    assert!(state.grid_visible);
    assert!(!state.dragging);
    assert_eq!(state.drag_start, None);
    assert_eq!(state.drag_end, None);
    assert_eq!(state.previous_inventory_active, None);
    assert_eq!(state.tool, EditorTool::Floor);
    assert_eq!(state.level, 2);

    state.previous_inventory_active = Some(None);
    state.end_session();
    assert!(!state.active);
    assert_eq!(state.previous_inventory_active, None);
}

#[test]
fn collider_authoring_requires_the_collider_item() {
    let inventory = Inventory(vec![
        None,
        Some(crate::levels::ItemEntry {
            item: COLLIDER_ITEM_ID.into(),
            quantity: crate::levels::INFINITE_QUANTITY,
        }),
    ]);

    assert!(selected_item_is_collider(
        &inventory,
        &InventoryActive(Some(1))
    ));
    assert!(!selected_item_is_collider(
        &inventory,
        &InventoryActive(Some(0))
    ));
    assert!(!selected_item_is_collider(
        &inventory,
        &InventoryActive(None)
    ));
}

#[test]
fn collider_cells_roundtrip_and_reject_non_grid_colliders() {
    let grid = grid();
    let collider = grid.collider_from_cells(IVec2::new(-2, 3), IVec2::new(1, 5));

    assert_eq!(
        grid.rect_cells_from_collider(&collider),
        Some((IVec2::new(-2, 3), IVec2::new(1, 5)))
    );

    let mut rotated = collider.clone();
    rotated.rotation_degrees = 15.0;
    assert_eq!(grid.cells_from_collider(&rotated), None);

    let mut custom = collider;
    custom.half_extents.x += 3.0;
    assert_eq!(grid.cells_from_collider(&custom), None);
}

#[test]
fn adjacent_aligned_colliders_merge_into_one_rectangle() {
    let grid = grid();
    let mut colliders = vec![
        grid.collider_from_cells(IVec2::new(0, 0), IVec2::new(0, 0)),
        grid.collider_from_cells(IVec2::new(1, 0), IVec2::new(1, 0)),
    ];

    let merged = merge_new_collider(
        &grid,
        &mut colliders,
        grid.collider_from_cells(IVec2::new(2, 0), IVec2::new(2, 0)),
    );

    assert_eq!(colliders.len(), 1);
    assert_eq!(colliders[0].position, merged.position);
    assert_eq!(
        grid.rect_cells_from_collider(&colliders[0]),
        Some((IVec2::new(0, 0), IVec2::new(2, 0)))
    );
}

#[test]
fn overlapping_colliders_collapse_into_larger_footprint() {
    let grid = grid();
    let mut colliders = vec![
        grid.collider_from_cells(IVec2::new(-2, -1), IVec2::new(1, 1)),
        grid.collider_from_cells(IVec2::new(2, -1), IVec2::new(3, 1)),
    ];

    merge_new_collider(
        &grid,
        &mut colliders,
        grid.collider_from_cells(IVec2::new(-2, -1), IVec2::new(3, 1)),
    );

    assert_eq!(colliders.len(), 1);
    assert_eq!(
        grid.rect_cells_from_collider(&colliders[0]),
        Some((IVec2::new(-2, -1), IVec2::new(3, 1)))
    );
}

#[test]
fn diagonal_or_offset_colliders_do_not_merge() {
    let grid = grid();
    let mut colliders = vec![grid.collider_from_cells(IVec2::ZERO, IVec2::ZERO)];

    merge_new_collider(
        &grid,
        &mut colliders,
        grid.collider_from_cells(IVec2::new(1, 1), IVec2::new(1, 1)),
    );

    assert_eq!(colliders.len(), 2);
}

#[test]
fn adjacent_colliders_merge_into_a_concave_polygon() {
    let grid = grid();
    let mut colliders = vec![grid.collider_from_cells(IVec2::ZERO, IVec2::ZERO)];

    merge_new_collider(
        &grid,
        &mut colliders,
        grid.collider_from_cells(IVec2::new(1, 0), IVec2::new(1, 0)),
    );
    let merged = merge_new_collider(
        &grid,
        &mut colliders,
        grid.collider_from_cells(IVec2::new(0, 1), IVec2::new(0, 1)),
    );

    assert_eq!(colliders.len(), 1);
    assert!(!merged.vertices.is_empty());

    let mut cells = grid
        .cells_from_collider(&merged)
        .expect("concave collider should roundtrip through cells");
    cells.sort_by_key(|cell| (cell.x, cell.y));
    assert_eq!(
        cells,
        vec![IVec2::new(0, 0), IVec2::new(0, 1), IVec2::new(1, 0)]
    );
}

#[test]
fn deconstruct_removes_one_cell_and_splits_disconnected_remnants() {
    let grid = grid();
    let mut colliders = vec![grid.collider_from_cells(IVec2::ZERO, IVec2::new(2, 0))];

    let fragment_count = remove_cell_from_colliders(&grid, &mut colliders, IVec2::new(1, 0))
        .expect("cell should be removed");
    let remaining: HashSet<IVec2> = colliders
        .iter()
        .flat_map(|collider| grid.cells_from_collider(collider).unwrap())
        .collect();

    assert_eq!(fragment_count, 2);
    assert_eq!(colliders.len(), 2);
    assert_eq!(
        remaining,
        HashSet::from([IVec2::new(0, 0), IVec2::new(2, 0)])
    );
}

#[test]
fn deconstruct_keeps_connected_remnant_as_one_collider() {
    let grid = grid();
    let mut colliders = vec![grid.collider_from_cells(IVec2::ZERO, IVec2::new(1, 1))];

    let fragment_count = remove_cell_from_colliders(&grid, &mut colliders, IVec2::ZERO)
        .expect("cell should be removed");
    let remaining = grid
        .cells_from_collider(&colliders[0])
        .expect("remaining L should be one collider");

    assert_eq!(fragment_count, 1);
    assert_eq!(colliders.len(), 1);
    assert_eq!(remaining.len(), 3);
    assert!(!remaining.contains(&IVec2::ZERO));
}

#[test]
fn deconstruct_center_of_solid_rect_preserves_the_hole() {
    let grid = grid();
    let mut colliders = vec![grid.collider_from_cells(IVec2::new(-1, -1), IVec2::new(1, 1))];

    remove_cell_from_colliders(&grid, &mut colliders, IVec2::ZERO).expect("cell should be removed");
    let remaining: HashSet<IVec2> = colliders
        .iter()
        .flat_map(|collider| grid.cells_from_collider(collider).unwrap())
        .collect();

    assert_eq!(remaining.len(), 8);
    assert!(!remaining.contains(&IVec2::ZERO));
}

#[test]
fn deconstruct_ignores_cells_without_a_collider() {
    let grid = grid();
    let mut colliders = vec![grid.collider_from_cells(IVec2::ZERO, IVec2::ZERO)];

    assert!(
        remove_cell_from_colliders(&grid, &mut colliders, IVec2::new(4, 4)).is_none(),
        "empty cells should not remove anything"
    );
    assert_eq!(colliders.len(), 1);
}

#[test]
fn completing_a_ring_does_not_fill_the_hole() {
    let grid = grid();
    let mut colliders = Vec::new();

    for cell in [
        IVec2::new(-1, -1),
        IVec2::new(0, -1),
        IVec2::new(1, -1),
        IVec2::new(1, 0),
        IVec2::new(1, 1),
        IVec2::new(0, 1),
        IVec2::new(-1, 1),
    ] {
        merge_new_collider(&grid, &mut colliders, grid.collider_from_cells(cell, cell));
    }

    assert_eq!(colliders.len(), 1);
    merge_new_collider(
        &grid,
        &mut colliders,
        grid.collider_from_cells(IVec2::new(-1, 0), IVec2::new(-1, 0)),
    );

    let occupied: HashSet<IVec2> = colliders
        .iter()
        .flat_map(|collider| {
            grid.cells_from_collider(collider)
                .expect("ring colliders should remain grid-based")
        })
        .collect();
    assert_eq!(occupied.len(), 8);
    assert!(!occupied.contains(&IVec2::ZERO));

    merge_new_collider(
        &grid,
        &mut colliders,
        grid.collider_from_cells(IVec2::new(-1, -1), IVec2::new(-1, -1)),
    );

    let all_cells = colliders
        .iter()
        .flat_map(|collider| {
            grid.cells_from_collider(collider)
                .expect("ring colliders should remain grid-based")
        })
        .collect::<Vec<_>>();
    let occupied: HashSet<IVec2> = all_cells.iter().copied().collect();
    assert_eq!(all_cells.len(), occupied.len());
    assert_eq!(occupied.len(), 8);
    assert!(!occupied.contains(&IVec2::ZERO));
}

#[test]
fn collider_from_cells_is_snapped_and_extends_with_cells() {
    let grid = grid();
    let single = grid.collider_from_cells(IVec2::ZERO, IVec2::ZERO);
    let double = grid.collider_from_cells(IVec2::ZERO, IVec2::new(1, 0));

    assert_eq!(single.position, Vec2::ZERO);
    assert!((single.half_extents.x - grid.cell_half_extent()).abs() < 0.01);
    assert_eq!(double.position, grid.cell_to_world(Vec2::new(0.5, 0.0)));
    assert!((double.half_extents.x - grid.cell_half_extent() * 2.0).abs() < 0.01);
}

#[test]
fn floor_tool_paints_and_removes_cells_per_level() {
    let mut floors = Vec::new();

    assert_eq!(
        add_floor_cells(&mut floors, IVec2::ZERO, IVec2::new(1, 1), 1.0),
        4
    );
    assert_eq!(
        add_floor_cells(&mut floors, IVec2::ZERO, IVec2::new(0, 0), 1.0),
        0
    );
    assert_eq!(floors.len(), 1);
    assert_eq!(floors[0].rank, 1);

    assert!(remove_floor_cell(&mut floors, IVec2::new(1, 1), 1.0));
    assert!(!remove_floor_cell(&mut floors, IVec2::new(1, 1), 1.0));
    assert_eq!(floors[0].cells.len(), 3);

    assert!(!remove_floor_cell(&mut floors, IVec2::new(1, 1), 2.0));
    for cell in [IVec2::new(0, 0), IVec2::new(1, 0), IVec2::new(0, 1)] {
        assert!(remove_floor_cell(&mut floors, cell, 1.0));
    }
    assert!(floors.is_empty());
}

#[test]
fn access_tool_connects_active_level_to_the_next_and_removes_by_height() {
    let grid = grid();
    let mut accesses = Vec::new();

    assert!(add_access(
        &mut accesses,
        &grid,
        IVec2::ZERO,
        IVec2::new(1, 0),
        0.0,
        AccessKind::Ramp,
    ));
    assert!(!add_access(
        &mut accesses,
        &grid,
        IVec2::ZERO,
        IVec2::new(1, 0),
        0.0,
        AccessKind::Ramp,
    ));
    assert_eq!(accesses.len(), 1);
    assert_eq!(accesses[0].from_elevation, 0.0);
    assert_eq!(accesses[0].to_elevation, 1.0);
    assert!(((accesses[0].to - accesses[0].from).length() - grid.step_u().length()).abs() < 0.01);

    assert_eq!(
        remove_access_at(
            &mut accesses,
            &grid,
            IVec2::new(1, 0),
            1.0,
            AccessKind::Ramp
        ),
        0
    );
    assert_eq!(
        remove_access_at(
            &mut accesses,
            &grid,
            IVec2::new(1, 0),
            0.0,
            AccessKind::Stairs
        ),
        0
    );
    assert_eq!(
        remove_access_at(
            &mut accesses,
            &grid,
            IVec2::new(1, 0),
            0.0,
            AccessKind::Ramp
        ),
        1
    );
    assert!(accesses.is_empty());
}

#[test]
fn elevated_cursor_targets_the_cell_under_the_visual_position() {
    let grid = grid();
    let elevation = 3.0;
    let cell = IVec2::new(3, -2);
    let visual = grid.cell_to_world(cell.as_vec2()) + editor_elevation_offset(elevation);

    assert_eq!(editor_cell_at_elevation(&grid, visual, elevation), cell);
}

#[test]
fn visible_grid_bounds_cover_the_viewport() {
    let grid = grid();
    let center = Vec2::new(480.0, -320.0);
    let area = Rect::new(-640.0, -480.0, 640.0, 480.0);
    let (min, max) = visible_grid_bounds(&grid, center, area);

    for corner in [
        Vec2::new(area.min.x, area.min.y),
        Vec2::new(area.min.x, area.max.y),
        Vec2::new(area.max.x, area.min.y),
        Vec2::new(area.max.x, area.max.y),
    ] {
        let cell = grid.world_to_cell(center + corner);
        assert!(min.x as f32 + 0.5 <= cell.x);
        assert!(max.x as f32 + 0.5 >= cell.x);
        assert!(min.y as f32 + 0.5 <= cell.y);
        assert!(max.y as f32 + 0.5 >= cell.y);
    }
}

#[test]
fn grid_level_offset_matches_the_floor_rise() {
    for level in 0..=3 {
        let elevation = level as f32;
        assert_eq!(
            editor_elevation_offset(elevation).y,
            elevation_screen_offset(elevation)
        );
    }
    assert_eq!(editor_elevation_offset(1.0).y, FLOOR_SCREEN_RISE);
}

#[test]
fn level_accents_cycle_without_colliding_between_neighbours() {
    for rank in 0..LEVEL_ACCENT_COUNT {
        assert_ne!(level_accent(rank), level_accent(rank + 1));
    }

    assert_eq!(level_accent(LEVEL_ACCENT_COUNT), level_accent(0));
    assert_eq!(level_accent(-1), level_accent(LEVEL_ACCENT_COUNT - 1));
}

#[test]
fn level_change_reports_direction_and_ignores_equal_levels() {
    assert_eq!(level_change(0, 1), Some(LevelChange::Up));
    assert_eq!(level_change(1, 0), Some(LevelChange::Down));
    assert_eq!(level_change(2, 2), None);
}
