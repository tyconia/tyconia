use crate::actions::CursorWorldPosition;
use crate::levels::{
    iso_corners, ColliderDeclared, CurrentLevel, Inventory, InventoryActive, Level, LevelInstance,
    LevelManager, LevelRequest, LevelRequests, ObstacleCollider, COLLIDER_ITEM_ID,
};
use crate::{DeveloperMode, GameState};
use bevy::prelude::*;

pub struct LevelEditorPlugin;

impl Plugin for LevelEditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<IsoGrid>()
            .init_resource::<LevelEditorState>()
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
                    editor_hotkeys,
                    select_editor_item.run_if(editor_active),
                    author_colliders.run_if(editor_active),
                    draw_iso_grid.run_if(editor_active),
                )
                    .chain()
                    .run_if(in_state(GameState::Playing).and(in_state(DeveloperMode(true)))),
            );
    }
}

fn editor_active(state: Res<LevelEditorState>) -> bool {
    state.active
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
    info!("Level editor active: L editor, G grid, drag LMB collider, K save, R regenerate");
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
        }
    }
}

#[derive(Resource, Debug, Default)]
pub struct LevelEditorState {
    pub active: bool,
    pub grid_visible: bool,
    pub dragging: bool,
    pub drag_start: Option<IVec2>,
    pub drag_end: Option<IVec2>,
    pub previous_inventory_active: Option<Option<usize>>,
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
    cursor: Res<CursorWorldPosition>,
    grid: Res<IsoGrid>,
    mut state: ResMut<LevelEditorState>,
    current: Res<CurrentLevel>,
    inventories: Query<(&Inventory, &InventoryActive)>,
    mut manager: Query<&mut LevelManager, With<Level>>,
    instances: Query<(Entity, &LevelInstance)>,
) {
    let Some(level_label) = current.0.clone() else {
        return;
    };

    let collider_selected = inventories
        .get_single()
        .is_ok_and(|(inventory, active)| selected_item_is_collider(inventory, active));

    if collider_selected && mouse.just_pressed(MouseButton::Left) {
        let cell = grid.snap_cell(cursor.0);
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

    let collider = grid.collider_from_cells(start, end);
    let Ok(mut manager) = manager.get_single_mut() else {
        state.dragging = false;
        return;
    };
    let Some(level) = manager.prefabs.get_mut(&level_label) else {
        state.dragging = false;
        return;
    };
    level.colliders.push(collider.clone());

    if let Some((instance_entity, _)) = instances
        .iter()
        .find(|(_, instance)| instance.prefab == level_label)
    {
        let obstacle = ObstacleCollider::iso(collider.half_extents);
        commands.entity(instance_entity).with_children(|parent| {
            parent.spawn((
                obstacle,
                Transform::from_xyz(collider.position.x, collider.position.y, 0.0),
            ));
        });
    }

    info!(
        "Level editor: added collider at {:?} to '{}'",
        collider.position, level_label
    );

    state.dragging = false;
    state.drag_start = None;
    state.drag_end = None;
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
    fn collider_from_cells_is_snapped_and_extends_with_cells() {
        let grid = grid();
        let single = grid.collider_from_cells(IVec2::ZERO, IVec2::ZERO);
        let double = grid.collider_from_cells(IVec2::ZERO, IVec2::new(1, 0));

        assert_eq!(single.position, Vec2::ZERO);
        assert!((single.half_extents.x - grid.cell_half_extent()).abs() < 0.01);
        assert_eq!(double.position, grid.cell_to_world(Vec2::new(0.5, 0.0)));
        assert!((double.half_extents.x - grid.cell_half_extent() * 2.0).abs() < 0.01);
    }
}
