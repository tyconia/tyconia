use crate::actions::{CursorWorldPosition, InterAction};
use crate::levels::{
    access_side_direction, elevation_screen_offset, iso_corners, level_z, point_in_polygon,
    polygon_signed_area, rank_from_elevation, stair_step_count, AccessDeclared, AccessKind,
    ColliderDeclared, CurrentLevel, FloorDeclared, GroundTransform, Inventory, InventoryActive,
    Level, LevelCollider, LevelInstance, LevelManager, LevelRequest, LevelRequests, LevelVertical,
    ObstacleCollider, OcclusionRole, RuntimeAccess, COLLIDER_ITEM_ID, FLOOR_SCREEN_RISE,
    PROP_Z_LAYER,
};
use crate::loading::UiAssets;
use crate::player::{Player, PlayerHeight};
use crate::ui::{
    Notification, NotificationChannel, NotificationLevel, NotificationPlacement, NotificationSize,
};
use crate::{DeveloperMode, GameState};
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use std::time::Duration;

const MIN_ACCESS_RUN_CELLS: f32 = 1.0;

mod authoring;
mod gizmos;
mod grid;
#[cfg(test)]
mod tests;

use authoring::*;
use gizmos::*;
pub use grid::IsoGrid;

fn editor_elevation_offset(elevation: f32) -> Vec2 {
    Vec2::new(0.0, elevation_screen_offset(elevation))
}

fn editor_cell_at_elevation(grid: &IsoGrid, cursor: Vec2, elevation: f32) -> IVec2 {
    grid.snap_cell(cursor - editor_elevation_offset(elevation))
}

pub struct LevelEditorPlugin;

impl Plugin for LevelEditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<IsoGrid>()
            .init_resource::<LevelEditorState>()
            .init_resource::<LevelFocus>()
            .init_resource::<VerticalGizmoCache>()
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
                notify_level_change
                    .after(follow_player_level)
                    .after(editor_hotkeys)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                (
                    follow_player_level,
                    editor_hotkeys,
                    select_editor_item.run_if(editor_active),
                    author_colliders.run_if(editor_active),
                    sync_level_vertical.run_if(editor_active),
                    cache_vertical_geometry,
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

#[derive(Resource, Default)]
struct LevelFocus {
    level: Option<i16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LevelChange {
    Up,
    Down,
}

fn level_change(previous: i16, current: i16) -> Option<LevelChange> {
    if current > previous {
        Some(LevelChange::Up)
    } else if current < previous {
        Some(LevelChange::Down)
    } else {
        None
    }
}

fn focused_level(
    state: &LevelEditorState,
    player: &Query<&PlayerHeight, With<Player>>,
) -> Option<i16> {
    if state.active {
        Some(state.level)
    } else {
        player.get_single().ok().map(|height| height.rank)
    }
}

fn notify_level_change(
    state: Res<LevelEditorState>,
    mut focus: ResMut<LevelFocus>,
    player: Query<&PlayerHeight, With<Player>>,
    ui: Res<UiAssets>,
    mut notifications: NotificationChannel,
) {
    let Some(level) = focused_level(&state, &player) else {
        return;
    };
    let Some(previous) = focus.level.replace(level) else {
        return;
    };
    let Some(change) = level_change(previous, level) else {
        return;
    };

    let icon = match change {
        LevelChange::Up => ui.level_up_ico.clone(),
        LevelChange::Down => ui.level_down_ico.clone(),
    };

    Notification {
        level: NotificationLevel::Info,
        title: format!("Y {level}"),
        description: String::new(),
        icon: Some(icon),
        size: NotificationSize::Small,
        placement: NotificationPlacement::TopRight,
    }
    .queue(Some(Duration::from_secs(2)), &mut notifications);
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

fn enable_editor(mut state: ResMut<LevelEditorState>, mut cache: ResMut<VerticalGizmoCache>) {
    cache.clear();
    state.active = true;
    state.grid_visible = true;
    info!(
        "Level editor active: L editor, G grid, 1 collider, 2 floor, 3 ramp, 4 stairs, PageUp/PageDown level, K save, R regenerate"
    );
}

fn disable_editor(
    mut state: ResMut<LevelEditorState>,
    mut cache: ResMut<VerticalGizmoCache>,
    mut inventories: Query<&mut InventoryActive>,
) {
    cache.clear();
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
    mut focus: ResMut<LevelFocus>,
) {
    state.begin_session(developer_mode.get().0);
    focus.level = None;
}

fn end_editor_session(mut state: ResMut<LevelEditorState>, mut focus: ResMut<LevelFocus>) {
    state.end_session();
    focus.level = None;
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
