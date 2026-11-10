use crate::actions::{CursorWorldPosition, InterAction};
use crate::levels::{
    access_side_direction, elevation_screen_offset, iso_corners, level_z, point_in_polygon,
    polygon_signed_area, rank_from_elevation, stair_step_count, AccessDeclared, AccessKind,
    ColliderDeclared, CurrentLevel, FloorDeclared, GroundTransform, Inventory, InventoryActive,
    Level, LevelCollider, LevelInstance, LevelManager, LevelRequest, LevelRequests, LevelVertical,
    ObstacleCollider, RuntimeAccess, COLLIDER_ITEM_ID, FLOOR_SCREEN_RISE, PROP_Z_LAYER,
};
use crate::{DeveloperMode, GameState};
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

const MIN_ACCESS_RUN_CELLS: f32 = 1.0;

pub struct LevelEditorPlugin;

impl Plugin for LevelEditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<IsoGrid>()
            .init_resource::<LevelEditorState>()
            .add_systems(OnEnter(GameState::Playing), begin_editor_session)
            .add_systems(OnExit(GameState::Playing), end_editor_session)
            .add_systems(OnEnter(DeveloperMode(true)), enable_editor)
            .add_systems(OnExit(DeveloperMode(true)), disable_editor)
            .add_systems(
                Update,
                enable_editor_from_shortcut
                    .run_if(in_state(GameState::Playing).and(in_state(DeveloperMode(false)))),
            )
            .add_systems(
                Update,
                (
                    follow_player_level,
                    editor_hotkeys,
                    select_editor_item.run_if(editor_active),
                    author_colliders.run_if(editor_active),
                    sync_level_vertical.run_if(editor_active),
                    draw_iso_grid.run_if(editor_active),
                    draw_vertical_geometry.run_if(editor_active),
                )
                    .chain()
                    .run_if(in_state(GameState::Playing).and(in_state(DeveloperMode(true)))),
            );
    }
}

fn editor_active(state: Res<LevelEditorState>) -> bool {
    state.active
}

fn follow_player_level(
    player: Query<&crate::player::PlayerHeight, With<crate::player::Player>>,
    mut state: ResMut<LevelEditorState>,
    mut last_level: Local<Option<i16>>,
) {
    let Ok(height) = player.get_single() else {
        return;
    };

    let level = height.elevation.round() as i16;
    if last_level.as_ref() == Some(&level) {
        return;
    }
    *last_level = Some(level);

    if state.level != level {
        state.level = level;
        info!("Level editor: following player to level {}", level);
    }
}

fn enable_editor_from_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    mut developer_mode: ResMut<NextState<DeveloperMode>>,
) {
    if keys.just_pressed(KeyCode::KeyL) {
        info!("Level editor: enabling developer mode from L shortcut");
        developer_mode.set(DeveloperMode(true));
    }
}

fn enable_editor(mut state: ResMut<LevelEditorState>) {
    state.active = true;
    state.grid_visible = true;
    info!(
        "Level editor active: L editor, G grid, 1 collider, 2 floor, 3 ramp, 4 stairs, PageUp/PageDown level, K save, R regenerate"
    );
}

fn disable_editor(
    mut state: ResMut<LevelEditorState>,
    mut inventories: Query<&mut InventoryActive>,
) {
    state.active = false;
    state.dragging = false;
    state.drag_start = None;
    state.drag_end = None;

    if let Some(previous) = state.previous_inventory_active.take() {
        if let Ok(mut active) = inventories.get_single_mut() {
            active.0 = previous;
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct IsoGrid {
    pub origin: Vec2,
    pub cell_size: Vec2,
    pub extent: i32,
    pub major_every: i32,
}

impl Default for IsoGrid {
    fn default() -> Self {
        Self {
            origin: Vec2::ZERO,
            cell_size: Vec2::new(64.0, 32.0),
            extent: 32,
            major_every: 4,
        }
    }
}

impl IsoGrid {
    pub fn half_axis(&self) -> Vec2 {
        self.cell_size * 0.25
    }

    pub fn step_u(&self) -> Vec2 {
        self.half_axis() * 2.0
    }

    pub fn step_v(&self) -> Vec2 {
        Vec2::new(self.step_u().x, -self.step_u().y)
    }

    pub fn cell_to_world(&self, cell: Vec2) -> Vec2 {
        self.origin + self.step_u() * cell.x + self.step_v() * cell.y
    }

    pub fn world_to_cell(&self, world: Vec2) -> Vec2 {
        let u = self.step_u();
        let v = self.step_v();
        let delta = world - self.origin;
        let determinant = u.x * v.y - v.x * u.y;

        if determinant.abs() <= f32::EPSILON {
            return Vec2::ZERO;
        }

        Vec2::new(
            (delta.x * v.y - v.x * delta.y) / determinant,
            (u.x * delta.y - delta.x * u.y) / determinant,
        )
    }

    pub fn snap_cell(&self, world: Vec2) -> IVec2 {
        self.world_to_cell(world).round().as_ivec2()
    }

    pub fn cell_half_extent(&self) -> f32 {
        self.half_axis().length()
    }

    pub fn footprint_half_extents(&self, cells: IVec2) -> Vec2 {
        Vec2::new(
            self.step_u().length() * cells.x as f32 * 0.5,
            self.step_v().length() * cells.y as f32 * 0.5,
        )
    }

    pub fn footprint_center(&self, min: IVec2, max: IVec2) -> Vec2 {
        self.cell_to_world((min.as_vec2() + max.as_vec2()) * 0.5)
    }

    pub fn collider_from_cells(&self, min: IVec2, max: IVec2) -> ColliderDeclared {
        let (min, max) = (min.min(max), min.max(max));
        let cells = max - min + IVec2::ONE;

        ColliderDeclared {
            position: self.footprint_center(min, max),
            half_extents: self.footprint_half_extents(cells),
            rotation_degrees: 0.0,
            screen_space: false,
            vertices: Vec::new(),
            z_min: 0.0,
            height: crate::levels::default_collider_height(),
        }
    }

    pub fn collider_from_cell_set(&self, cells: &HashSet<IVec2>) -> Option<ColliderDeclared> {
        let min = cells.iter().copied().reduce(IVec2::min)?;
        let max = cells.iter().copied().reduce(IVec2::max)?;
        let width = max.x - min.x + 1;
        let height = max.y - min.y + 1;

        if cells.len() == (width * height) as usize {
            return Some(self.collider_from_cells(min, max));
        }

        let mut loops = self.cell_boundary_loops(cells);
        if loops.len() != 1 {
            return None;
        }

        let mut points = loops.remove(0);
        simplify_polygon(&mut points);
        if points.len() < 3 {
            return None;
        }
        if polygon_signed_area(&points) < 0.0 {
            points.reverse();
        }

        let position = points.iter().copied().sum::<Vec2>() / points.len() as f32;
        let min = points
            .iter()
            .copied()
            .fold(Vec2::splat(f32::INFINITY), Vec2::min);
        let max = points
            .iter()
            .copied()
            .fold(Vec2::splat(f32::NEG_INFINITY), Vec2::max);

        Some(ColliderDeclared {
            position,
            half_extents: (max - min) * 0.5,
            rotation_degrees: 0.0,
            screen_space: false,
            vertices: points.iter().map(|point| *point - position).collect(),
            z_min: 0.0,
            height: crate::levels::default_collider_height(),
        })
    }

    pub fn cell_boundary_loops(&self, cells: &HashSet<IVec2>) -> Vec<Vec<Vec2>> {
        let mut outgoing: HashMap<IVec2, Vec<IVec2>> = HashMap::new();

        for &cell in cells {
            let corners = [
                IVec2::new(cell.x * 2 - 1, cell.y * 2 - 1),
                IVec2::new(cell.x * 2 + 1, cell.y * 2 - 1),
                IVec2::new(cell.x * 2 + 1, cell.y * 2 + 1),
                IVec2::new(cell.x * 2 - 1, cell.y * 2 + 1),
            ];

            for (direction, edge) in [
                (IVec2::X, (1, 2)),
                (IVec2::Y, (2, 3)),
                (-IVec2::X, (3, 0)),
                (-IVec2::Y, (0, 1)),
            ] {
                if !cells.contains(&(cell + direction)) {
                    outgoing
                        .entry(corners[edge.0])
                        .or_default()
                        .push(corners[edge.1]);
                }
            }
        }

        let mut loops = Vec::new();

        while let Some((&start, _)) = outgoing.iter().next() {
            let mut loop_keys = Vec::new();
            let mut previous = None;
            let mut current = start;

            loop {
                let Some(candidates) = outgoing.get_mut(&current) else {
                    break;
                };
                if candidates.is_empty() {
                    break;
                }

                loop_keys.push(current);
                let next = if candidates.len() == 1 {
                    candidates.remove(0)
                } else {
                    let index = match previous {
                        Some(previous) => choose_next_boundary_edge(previous, current, candidates),
                        None => 0,
                    };
                    candidates.remove(index)
                };

                previous = Some(current);
                current = next;
                if current == start {
                    break;
                }
                if loop_keys.len() > cells.len() * 8 + 8 {
                    break;
                }
            }

            if current == start && loop_keys.len() >= 3 {
                loops.push(
                    loop_keys
                        .into_iter()
                        .map(|key| self.cell_to_world(key.as_vec2() * 0.5))
                        .collect(),
                );
            }

            outgoing.retain(|_, edges| !edges.is_empty());
        }

        loops
    }

    pub fn rect_cells_from_collider(&self, collider: &ColliderDeclared) -> Option<(IVec2, IVec2)> {
        if collider.screen_space
            || collider.rotation_degrees.abs() > 0.01
            || !collider.vertices.is_empty()
        {
            return None;
        }

        let step_length = self.step_u().length();
        if step_length <= f32::EPSILON {
            return None;
        }

        let cells = IVec2::new(
            (collider.half_extents.x * 2.0 / step_length).round() as i32,
            (collider.half_extents.y * 2.0 / step_length).round() as i32,
        );
        if cells.x < 1 || cells.y < 1 {
            return None;
        }

        let expected_half_extents = self.footprint_half_extents(cells);
        if (expected_half_extents - collider.half_extents).length_squared() > 0.001 {
            return None;
        }

        let center = self.world_to_cell(collider.position);
        let min = IVec2::new(
            (center.x - (cells.x as f32 - 1.0) * 0.5).round() as i32,
            (center.y - (cells.y as f32 - 1.0) * 0.5).round() as i32,
        );
        let max = min + cells - IVec2::ONE;
        let rebuilt = self.collider_from_cells(min, max);

        if (rebuilt.position - collider.position).length_squared() > 0.001 {
            return None;
        }

        Some((min, max))
    }

    pub fn cells_from_collider(&self, collider: &ColliderDeclared) -> Option<Vec<IVec2>> {
        if collider.screen_space || collider.rotation_degrees.abs() > 0.01 {
            return None;
        }

        if !collider.vertices.is_empty() {
            return self.cells_from_polygon(collider);
        }

        let (min, max) = self.rect_cells_from_collider(collider)?;
        Some(
            (min.y..=max.y)
                .flat_map(|y| (min.x..=max.x).map(move |x| IVec2::new(x, y)))
                .collect(),
        )
    }

    fn cells_from_polygon(&self, collider: &ColliderDeclared) -> Option<Vec<IVec2>> {
        let points: Vec<Vec2> = collider
            .vertices
            .iter()
            .map(|vertex| *vertex + collider.position)
            .collect();
        if points.len() < 3 {
            return None;
        }

        let mut cell_min = Vec2::splat(f32::INFINITY);
        let mut cell_max = Vec2::splat(f32::NEG_INFINITY);
        for point in &points {
            let doubled_cell = self.world_to_cell(*point) * 2.0;
            if (doubled_cell - doubled_cell.round()).length_squared() > 0.01 {
                return None;
            }
            let cell = self.world_to_cell(*point);
            cell_min = cell_min.min(cell);
            cell_max = cell_max.max(cell);
        }

        let min = IVec2::new(cell_min.x.floor() as i32, cell_min.y.floor() as i32);
        let max = IVec2::new(cell_max.x.ceil() as i32, cell_max.y.ceil() as i32);

        let mut cells = Vec::new();
        for y in min.y..=max.y {
            for x in min.x..=max.x {
                let center = self.cell_to_world(Vec2::new(x as f32, y as f32));
                if point_in_polygon(center, &points) {
                    cells.push(IVec2::new(x, y));
                }
            }
        }
        if cells.is_empty() {
            return None;
        }

        let cell_area = {
            let u = self.step_u();
            let v = self.step_v();
            (u.x * v.y - v.x * u.y).abs()
        };
        let polygon_area = polygon_signed_area(&points).abs();
        if (polygon_area - cells.len() as f32 * cell_area).abs() > cell_area * 0.01 {
            return None;
        }

        Some(cells)
    }
}

fn choose_next_boundary_edge(previous: IVec2, current: IVec2, candidates: &[IVec2]) -> usize {
    let incoming = (current - previous).as_vec2();
    let mut best = 0;
    let mut best_angle = f32::INFINITY;

    for (index, candidate) in candidates.iter().enumerate() {
        let outgoing = (*candidate - current).as_vec2();
        let angle = incoming.perp_dot(outgoing).atan2(incoming.dot(outgoing));
        if angle < best_angle {
            best_angle = angle;
            best = index;
        }
    }

    best
}

fn simplify_polygon(points: &mut Vec<Vec2>) {
    while points.len() > 3 {
        let mut removed = false;

        for index in 0..points.len() {
            let previous = points[(index + points.len() - 1) % points.len()];
            let current = points[index];
            let next = points[(index + 1) % points.len()];
            let cross = (current - previous).perp_dot(next - current);
            let continues = (current - previous).dot(next - current) > 0.0;

            if cross.abs() <= 0.001 && continues {
                points.remove(index);
                removed = true;
                break;
            }
        }

        if !removed {
            break;
        }
    }
}

fn cell_touches_set(cell: IVec2, cells: &HashSet<IVec2>) -> bool {
    cells.contains(&cell)
        || [IVec2::X, -IVec2::X, IVec2::Y, -IVec2::Y]
            .into_iter()
            .any(|direction| cells.contains(&(cell + direction)))
}

fn merge_new_collider(
    grid: &IsoGrid,
    colliders: &mut Vec<ColliderDeclared>,
    collider: ColliderDeclared,
) -> ColliderDeclared {
    let Some(new_cells) = grid.cells_from_collider(&collider) else {
        colliders.push(collider.clone());
        return collider;
    };

    let mut merged_cells: HashSet<IVec2> = new_cells.into_iter().collect();
    let mut consumed = Vec::new();

    loop {
        let mut found = None;

        for (index, existing) in colliders.iter().enumerate() {
            if consumed.contains(&index) || !existing.same_vertical_range(&collider) {
                continue;
            }

            let Some(existing_cells) = grid.cells_from_collider(existing) else {
                continue;
            };

            if existing_cells
                .iter()
                .any(|cell| cell_touches_set(*cell, &merged_cells))
            {
                merged_cells.extend(existing_cells);
                found = Some(index);
                break;
            }
        }

        let Some(index) = found else {
            break;
        };
        consumed.push(index);
    }

    if consumed.is_empty() {
        colliders.push(collider.clone());
        return collider;
    }

    let mut merged = if let Some(merged_collider) = grid.collider_from_cell_set(&merged_cells) {
        vec![merged_collider]
    } else {
        colliders_from_cell_set(grid, &merged_cells)
    };
    if merged.is_empty() {
        colliders.push(collider.clone());
        return collider;
    }
    for fragment in &mut merged {
        fragment.copy_vertical_from(&collider);
    }
    let result = merged[0].clone();

    consumed.sort_unstable_by(|a, b| b.cmp(a));
    for index in consumed {
        colliders.remove(index);
    }

    colliders.extend(merged);
    result
}

fn remove_cell_from_colliders(
    grid: &IsoGrid,
    colliders: &mut Vec<ColliderDeclared>,
    cell: IVec2,
) -> Option<usize> {
    let index = colliders.iter().rposition(|collider| {
        grid.cells_from_collider(collider)
            .is_some_and(|cells| cells.contains(&cell))
    })?;
    let mut cells: HashSet<IVec2> = grid
        .cells_from_collider(&colliders[index])
        .expect("matched collider should convert to cells")
        .into_iter()
        .collect();
    let removed = colliders.remove(index);

    cells.remove(&cell);
    let mut fragments = colliders_from_cell_set(grid, &cells);
    for fragment in &mut fragments {
        fragment.copy_vertical_from(&removed);
    }
    let fragment_count = fragments.len();
    colliders.extend(fragments);
    Some(fragment_count)
}

fn colliders_from_cell_set(grid: &IsoGrid, cells: &HashSet<IVec2>) -> Vec<ColliderDeclared> {
    let mut colliders = Vec::new();

    for component in connected_cell_components(cells) {
        if let Some(collider) = grid.collider_from_cell_set(&component) {
            colliders.push(collider);
        } else {
            colliders.extend(decompose_cell_set_into_rects(grid, component));
        }
    }

    colliders
}

fn connected_cell_components(cells: &HashSet<IVec2>) -> Vec<HashSet<IVec2>> {
    let mut remaining = cells.clone();
    let mut components = Vec::new();

    while let Some(&start) = remaining.iter().next() {
        remaining.remove(&start);
        let mut component = HashSet::from([start]);
        let mut stack = vec![start];

        while let Some(cell) = stack.pop() {
            for direction in [IVec2::X, -IVec2::X, IVec2::Y, -IVec2::Y] {
                let neighbor = cell + direction;
                if remaining.remove(&neighbor) {
                    component.insert(neighbor);
                    stack.push(neighbor);
                }
            }
        }

        components.push(component);
    }

    components
}

fn decompose_cell_set_into_rects(
    grid: &IsoGrid,
    mut cells: HashSet<IVec2>,
) -> Vec<ColliderDeclared> {
    let mut colliders = Vec::new();

    while let Some(&start) = cells.iter().min_by_key(|cell| (cell.y, cell.x)) {
        let mut width = 1;
        while cells.contains(&IVec2::new(start.x + width, start.y)) {
            width += 1;
        }

        let mut height = 1;
        'height: loop {
            for offset_x in 0..width {
                if !cells.contains(&IVec2::new(start.x + offset_x, start.y + height)) {
                    break 'height;
                }
            }
            height += 1;
        }

        for offset_y in 0..height {
            for offset_x in 0..width {
                cells.remove(&IVec2::new(start.x + offset_x, start.y + offset_y));
            }
        }

        colliders.push(
            grid.collider_from_cells(start, IVec2::new(start.x + width - 1, start.y + height - 1)),
        );
    }

    colliders
}

fn refresh_level_collider_entities(
    commands: &mut Commands,
    instance_entity: Entity,
    old_colliders: impl IntoIterator<Item = Entity>,
    colliders: &[ColliderDeclared],
) {
    for entity in old_colliders {
        commands.entity(entity).despawn_recursive();
    }

    commands.entity(instance_entity).with_children(|parent| {
        for collider in colliders {
            let ground = GroundTransform::from_declared(collider);
            let rank = rank_from_elevation(collider.z_min);
            parent.spawn((
                LevelCollider,
                ObstacleCollider::from_declared(collider),
                ground,
                Transform::from_xyz(
                    collider.position.x,
                    collider.position.y + elevation_screen_offset(collider.z_min),
                    level_z(rank, PROP_Z_LAYER, collider.position.y),
                )
                .with_rotation(Quat::from_rotation_z(ground.rotation)),
            ));
        }
    });
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditorTool {
    #[default]
    Collider,
    Floor,
    Ramp,
    Stairs,
}

#[derive(Resource, Debug, Default)]
pub struct LevelEditorState {
    pub active: bool,
    pub grid_visible: bool,
    pub dragging: bool,
    pub drag_start: Option<IVec2>,
    pub drag_end: Option<IVec2>,
    pub previous_inventory_active: Option<Option<usize>>,
    pub tool: EditorTool,
    pub level: i16,
}

impl LevelEditorState {
    fn begin_session(&mut self, active: bool) {
        self.active = active;
        self.grid_visible = true;
        self.clear_transient_state();
    }

    fn end_session(&mut self) {
        self.active = false;
        self.clear_transient_state();
    }

    fn clear_transient_state(&mut self) {
        self.dragging = false;
        self.drag_start = None;
        self.drag_end = None;
        self.previous_inventory_active = None;
    }
}

fn begin_editor_session(
    mut state: ResMut<LevelEditorState>,
    developer_mode: Res<State<DeveloperMode>>,
) {
    state.begin_session(developer_mode.get().0);
}

fn end_editor_session(mut state: ResMut<LevelEditorState>) {
    state.end_session();
}

fn editor_hotkeys(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<LevelEditorState>,
    current: Res<CurrentLevel>,
    mut requests: ResMut<LevelRequests>,
) {
    if keys.just_pressed(KeyCode::KeyL) {
        state.active = !state.active;
        state.dragging = false;
        state.drag_start = None;
        state.drag_end = None;
        info!(
            "Level editor: {}",
            if state.active { "active" } else { "inactive" }
        );
    }

    if keys.just_pressed(KeyCode::KeyG) {
        state.grid_visible = !state.grid_visible;
    }

    let tool = if keys.just_pressed(KeyCode::Digit1) {
        Some(EditorTool::Collider)
    } else if keys.just_pressed(KeyCode::Digit2) {
        Some(EditorTool::Floor)
    } else if keys.just_pressed(KeyCode::Digit3) {
        Some(EditorTool::Ramp)
    } else if keys.just_pressed(KeyCode::Digit4) {
        Some(EditorTool::Stairs)
    } else {
        None
    };
    if let Some(tool) = tool {
        state.tool = tool;
        info!("Level editor: tool {:?}", tool);
    }

    if keys.just_pressed(KeyCode::PageUp) {
        state.level += 1;
        info!("Level editor: level {}", state.level);
    }
    if keys.just_pressed(KeyCode::PageDown) {
        state.level -= 1;
        info!("Level editor: level {}", state.level);
    }

    let Some(level_label) = current.0.clone() else {
        return;
    };

    if keys.just_pressed(KeyCode::KeyK) {
        requests.0.push(LevelRequest::Save {
            level_label: level_label.clone(),
        });
    }

    if keys.just_pressed(KeyCode::KeyR) {
        requests.0.push(LevelRequest::Regenerate { level_label });
    }
}

fn select_editor_item(
    mut state: ResMut<LevelEditorState>,
    mut inventories: Query<(&Inventory, &mut InventoryActive)>,
) {
    if state.previous_inventory_active.is_some() {
        return;
    }

    let Ok((inventory, mut active)) = inventories.get_single_mut() else {
        return;
    };
    state.previous_inventory_active = Some(active.0);

    let Some(index) = inventory.0.iter().position(|slot| {
        slot.as_ref()
            .is_some_and(|entry| entry.item.0 == COLLIDER_ITEM_ID)
    }) else {
        return;
    };

    active.0 = Some(index);
}

fn selected_item_is_collider(inventory: &Inventory, active: &InventoryActive) -> bool {
    active
        .0
        .and_then(|index| inventory.0.get(index))
        .and_then(|slot| slot.as_ref())
        .is_some_and(|entry| entry.item.0 == COLLIDER_ITEM_ID)
}

fn author_colliders(
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

    let cell = grid.snap_cell(cursor.0);
    let elevation = state.level as f32;

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
        state.drag_end = Some(grid.snap_cell(cursor.0));
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

fn access_kind(tool: EditorTool) -> AccessKind {
    match tool {
        EditorTool::Stairs => AccessKind::Stairs,
        _ => AccessKind::Ramp,
    }
}

fn add_floor_cells(
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

fn remove_floor_cell(floors: &mut Vec<FloorDeclared>, cell: IVec2, elevation: f32) -> bool {
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

fn add_access(
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

fn remove_access_at(
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

fn sync_level_vertical(
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

fn draw_vertical_geometry(
    mut gizmos: Gizmos,
    grid: Res<IsoGrid>,
    state: Res<LevelEditorState>,
    instances: Query<(&LevelVertical, &GlobalTransform)>,
) {
    for (vertical, transform) in &instances {
        draw_vertical_gizmos(&mut gizmos, &grid, &state, vertical, transform);
    }
}

fn editor_rank_alpha(rank: i16, focus: i16) -> f32 {
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

fn transform_point(point: Vec2, transform: &GlobalTransform) -> Vec2 {
    transform.transform_point(point.extend(0.0)).truncate()
}

fn draw_transformed_line(
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

fn draw_stair_access_gizmo(
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

fn draw_iso_grid(
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

#[cfg(test)]
mod tests {
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

        remove_cell_from_colliders(&grid, &mut colliders, IVec2::ZERO)
            .expect("cell should be removed");
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
        assert!(
            ((accesses[0].to - accesses[0].from).length() - grid.step_u().length()).abs() < 0.01
        );

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
}
