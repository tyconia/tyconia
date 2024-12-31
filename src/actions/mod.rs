use bevy::input::mouse::MouseWheel;
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;

mod mappings;
use mappings::*;

use crate::actions::game_control::{get_movement, GameControl};
use crate::player::Player;
use crate::InGameState;

mod game_control;

pub const FOLLOW_EPSILON: f32 = 5.;

pub struct ActionsPlugin;

// This plugin listens for keyboard input and converts the input into Actions
// Actions can then be used as a resource in other systems to act on the player input.
impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorWorldPosition>()
            .add_event::<PlayerMovementAction>()
            .add_event::<UiAction>()
            .add_event::<PlayerAction>()
            .add_systems(
                Update,
                (
                    set_ui_actions,
                    set_movement_actions,
                    set_cursor.run_if(on_event::<CursorMoved>),
                )
                    .run_if(in_state(InGameState::Normal)),
            );
    }
}

pub enum Action {
    PlayerMovementAction(PlayerMovementAction),
    UiAction(UiAction),
    PlayerAction(PlayerAction),
}

#[derive(Default, Resource)]
pub struct CursorWorldPosition(pub Vec2);

#[derive(Default, Event)]
pub struct PlayerMovementAction(pub Vec2);

#[derive(Event)]
/// Actions for changing the HUD
pub enum UiAction {
    /// Change map zoom
    Zoom(f32),
    /// Hotbar select
    HotbarSlot(usize),
    /// Summon pause menu
    Menu,
}

#[derive(Event)]
/// Player  interactions with entities
pub enum PlayerAction {
    /// Grab copy of entity from inventory
    Pipette,
    /// Placing down entity
    Construct,
    /// Destroy entity
    Deconstruct,
    /// Provide item for entity
    Distribute,
    /// Copy entity attributes
    CopyConfiguration,
}

pub fn set_cursor(
    camera_q: Query<(&GlobalTransform, &Camera)>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut cursor_world_position: ResMut<CursorWorldPosition>,
) {
    for cursor_moved in cursor_moved_events.read() {
        // To get the mouse's world position, we have to transform its window position by
        // any transforms on the camera. This is done by projecting the cursor position into
        // camera space (world space).
        for (cam_t, cam) in camera_q.iter() {
            if let Ok(pos) = cam.viewport_to_world_2d(cam_t, cursor_moved.position) {
                *cursor_world_position = CursorWorldPosition(pos);
            }
        }
    }
}

/// Mouse wheel movement to Zoom actions
pub fn set_ui_actions(
    mut ui_actions: EventWriter<UiAction>,
    mut mouse_wheel: EventReader<MouseWheel>,
) {
    let mw_movement: f32 = mouse_wheel.read().map(|mw| mw.y).sum();
    if mw_movement > 0.1 || mw_movement < -0.1 {
        ui_actions.send(UiAction::Zoom(mw_movement));
    }
}

/// Keyboard movement to player movement actions
pub fn set_movement_actions(
    mut player_movement_action: EventWriter<PlayerMovementAction>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    touch_input: Res<Touches>,
    player: Query<&Transform, With<Player>>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    let mut player_movement = Vec2::new(
        get_movement(GameControl::Right, &keyboard_input)
            - get_movement(GameControl::Left, &keyboard_input),
        get_movement(GameControl::Up, &keyboard_input)
            - get_movement(GameControl::Down, &keyboard_input),
    );

    if let Some(touch_position) = touch_input.first_pressed_position() {
        let (camera, camera_transform) = camera.single();
        if let Ok(touch_position) = camera.viewport_to_world_2d(camera_transform, touch_position) {
            let diff = touch_position - player.single().translation.xy();
            if diff.length() > FOLLOW_EPSILON {
                player_movement = diff.normalize();
                player_movement_action.send(PlayerMovementAction(diff.normalize()));
            }
        }
    }

    if player_movement != Vec2::ZERO {
        player_movement_action.send(PlayerMovementAction(player_movement.normalize()));
    }
}
