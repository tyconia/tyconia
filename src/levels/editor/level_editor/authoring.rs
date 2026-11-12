use super::grid::*;
use super::*;

pub(super) fn selected_item_is_collider(inventory: &Inventory, active: &InventoryActive) -> bool {
    active
        .0
        .and_then(|index| inventory.0.get(index))
        .and_then(|slot| slot.as_ref())
        .is_some_and(|entry| entry.item.0 == COLLIDER_ITEM_ID)
}

pub(super) fn author_colliders(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    mut inter_actions: EventReader<InterAction>,
    cursor: Res<CursorWorldPosition>,
    grid: Res<IsoGrid>,
    mut state: ResMut<LevelEditorState>,
    current: Res<CurrentLevel>,
    inventories: Query<(&Inventory, &InventoryActive)>,
    mut manager: Query<&mut LevelManager, With<Level>>,
    instances: Query<(Entity, &LevelInstance)>,
    level_colliders: Query<(Entity, &Parent), With<LevelCollider>>,
) {
    let Some(level_label) = current.0.clone() else {
        return;
    };

    let collider_selected = inventories
        .get_single()
        .is_ok_and(|(inventory, active)| selected_item_is_collider(inventory, active));
    let deconstruct_requested = inter_actions
        .read()
        .any(|action| matches!(action, InterAction::Deconstruct));
    let can_author = state.tool != EditorTool::Collider || collider_selected;

    if !can_author {
        return;
    }

    let elevation = state.level as f32;
    let cell = editor_cell_at_elevation(&grid, cursor.0, elevation);

    if deconstruct_requested {
        let Ok(mut manager) = manager.get_single_mut() else {
            return;
        };
        let Some(level) = manager.prefabs.get_mut(&level_label) else {
            return;
        };

        match state.tool {
            EditorTool::Collider => {
                let Some(fragment_count) =
                    remove_cell_from_colliders(&grid, &mut level.colliders, cell)
                else {
                    return;
                };
                let colliders = level.colliders.clone();

                if let Some((instance_entity, _)) = instances
                    .iter()
                    .find(|(_, instance)| instance.prefab == level_label)
                {
                    refresh_level_collider_entities(
                        &mut commands,
                        instance_entity,
                        level_colliders.iter().filter_map(|(entity, parent)| {
                            (parent.get() == instance_entity).then_some(entity)
                        }),
                        &colliders,
                    );
                }

                info!(
                    "Level editor: removed collider cell {:?} from '{}' ({} fragment(s))",
                    cell, level_label, fragment_count
                );
            }
            EditorTool::Floor => {
                let removed = remove_floor_cell(&mut level.floors, cell, elevation);
                info!(
                    "Level editor: removed floor cell {:?} at level {} from '{}' ({})",
                    cell, state.level, level_label, removed
                );
            }
            EditorTool::Ramp | EditorTool::Stairs => {
                let kind = access_kind(state.tool);
                let removed = remove_access_at(&mut level.accesses, &grid, cell, elevation, kind);
                info!(
                    "Level editor: removed {:?} at cell {:?} from '{}' ({})",
                    kind, cell, level_label, removed
                );
            }
        }

        state.dragging = false;
        state.drag_start = None;
        state.drag_end = None;
        return;
    }

    if mouse.just_pressed(MouseButton::Left) {
        state.dragging = true;
        state.drag_start = Some(cell);
        state.drag_end = Some(cell);
    }

    if mouse.pressed(MouseButton::Left) && state.dragging {
        state.drag_end = Some(editor_cell_at_elevation(&grid, cursor.0, elevation));
    }

    if !mouse.just_released(MouseButton::Left) || !state.dragging {
        return;
    }

    let (Some(start), Some(end)) = (state.drag_start, state.drag_end) else {
        state.dragging = false;
        return;
    };

    let Ok(mut manager) = manager.get_single_mut() else {
        state.dragging = false;
        return;
    };
    let Some(level) = manager.prefabs.get_mut(&level_label) else {
        state.dragging = false;
        return;
    };

    match state.tool {
        EditorTool::Collider => {
            let collider = grid.collider_from_cells(start, end);
            let collider = merge_new_collider(&grid, &mut level.colliders, collider);
            let colliders = level.colliders.clone();

            if let Some((instance_entity, _)) = instances
                .iter()
                .find(|(_, instance)| instance.prefab == level_label)
            {
                refresh_level_collider_entities(
                    &mut commands,
                    instance_entity,
                    level_colliders.iter().filter_map(|(entity, parent)| {
                        (parent.get() == instance_entity).then_some(entity)
                    }),
                    &colliders,
                );
            }

            info!(
                "Level editor: merged collider at {:?} into '{}'",
                collider.position, level_label
            );
        }
        EditorTool::Floor => {
            let added = add_floor_cells(&mut level.floors, start, end, elevation);
            info!(
                "Level editor: added {} floor cell(s) at level {} to '{}'",
                added, state.level, level_label
            );
        }
        EditorTool::Ramp | EditorTool::Stairs => {
            let kind = access_kind(state.tool);
            let added = add_access(&mut level.accesses, &grid, start, end, elevation, kind);
            info!(
                "Level editor: added {:?} from {:?} to {:?} at level {} to '{}' ({})",
                kind, start, end, state.level, level_label, added
            );
        }
    }

    state.dragging = false;
    state.drag_start = None;
    state.drag_end = None;
}

pub(super) fn access_kind(tool: EditorTool) -> AccessKind {
    match tool {
        EditorTool::Stairs => AccessKind::Stairs,
        _ => AccessKind::Ramp,
    }
}

pub(super) fn add_floor_cells(
    floors: &mut Vec<FloorDeclared>,
    start: IVec2,
    end: IVec2,
    elevation: f32,
) -> usize {
    let floor_index = floors
        .iter()
        .position(|floor| (floor.elevation - elevation).abs() < 0.01)
        .unwrap_or_else(|| {
            floors.push(FloorDeclared {
                elevation,
                rank: elevation.round() as i16,
                cells: Vec::new(),
            });
            floors.len() - 1
        });
    let floor = &mut floors[floor_index];
    let min = start.min(end);
    let max = start.max(end);
    let mut added = 0;

    for y in min.y..=max.y {
        for x in min.x..=max.x {
            let cell = IVec2::new(x, y);
            if !floor.cells.contains(&cell) {
                floor.cells.push(cell);
                added += 1;
            }
        }
    }

    added
}

pub(super) fn remove_floor_cell(
    floors: &mut Vec<FloorDeclared>,
    cell: IVec2,
    elevation: f32,
) -> bool {
    let Some(index) = floors
        .iter()
        .position(|floor| (floor.elevation - elevation).abs() < 0.01)
    else {
        return false;
    };

    let before = floors[index].cells.len();
    floors[index].cells.retain(|candidate| *candidate != cell);
    let removed = floors[index].cells.len() != before;
    if floors[index].cells.is_empty() {
        floors.remove(index);
    }
    removed
}

pub(super) fn add_access(
    accesses: &mut Vec<AccessDeclared>,
    grid: &IsoGrid,
    start: IVec2,
    end: IVec2,
    elevation: f32,
    kind: AccessKind,
) -> bool {
    if start == end {
        return false;
    }

    let mut from = grid.cell_to_world(start.as_vec2());
    let to = grid.cell_to_world(end.as_vec2());
    let minimum_run = grid.step_u().length() * MIN_ACCESS_RUN_CELLS;
    let run = from - to;
    if run.length() < minimum_run {
        from = to + run.normalize_or_zero() * minimum_run;
    }

    let access = AccessDeclared {
        from,
        to,
        width: grid.step_u().length(),
        from_elevation: elevation,
        to_elevation: elevation + 1.0,
        kind,
    };
    if accesses.iter().any(|existing| {
        existing.kind == kind
            && (existing.from - access.from).length_squared() < 0.01
            && (existing.to - access.to).length_squared() < 0.01
            && (existing.from_elevation - access.from_elevation).abs() < 0.01
    }) {
        return false;
    }

    accesses.push(access);
    true
}

pub(super) fn remove_access_at(
    accesses: &mut Vec<AccessDeclared>,
    grid: &IsoGrid,
    cell: IVec2,
    elevation: f32,
    kind: AccessKind,
) -> usize {
    let center = grid.cell_to_world(cell.as_vec2());
    let mut removed = 0;

    accesses.retain(|access| {
        if access.kind != kind || (access.from_elevation - elevation).abs() >= 0.01 {
            return true;
        }
        let runtime = RuntimeAccess::from(access);
        let Some(_) = runtime.height_at(grid, center) else {
            return true;
        };
        removed += 1;
        false
    });

    removed
}

pub(super) fn sync_level_vertical(
    current: Res<CurrentLevel>,
    changed_manager: Query<&LevelManager, (With<Level>, Changed<LevelManager>)>,
    manager: Query<&LevelManager, With<Level>>,
    mut verticals: Query<(&LevelInstance, &mut LevelVertical)>,
) {
    if changed_manager.get_single().is_err() {
        return;
    }
    let Ok(manager) = manager.get_single() else {
        return;
    };
    let Some(label) = current.0.as_ref() else {
        return;
    };
    let Some(level) = manager.prefabs.get(label) else {
        return;
    };

    for (instance, mut vertical) in &mut verticals {
        if &instance.prefab == label {
            *vertical = LevelVertical::from_level(level);
        }
    }
}
