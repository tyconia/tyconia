use super::MovementAction;
use crate::GameState;
use bevy::prelude::*;

/// Enables systems to update [`CursorWorldPosition`]
pub(crate) struct CursorsPlugin;

impl Plugin for CursorsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorWorldPosition>().add_systems(
            Update,
            (set_cursor.run_if(
                // runs when either mouse or player movement is detected
                on_event::<CursorMoved>.or(on_event::<MovementAction>),
            ),)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

/// [`Resource`] position of the cursor projected to the world
#[derive(Default, Resource)]
pub struct CursorWorldPosition(pub Vec2);

pub fn set_cursor(
    camera: Query<(&GlobalTransform, &Camera)>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut cursor_world_position: ResMut<CursorWorldPosition>,
    mut last_moved: Local<Vec2>,
) {
    let cursor_moved_events = cursor_moved_events.read();
    if cursor_moved_events.len() == 0 {
        let (transform, cam) = camera.single();
        if let Ok(pos) = cam.viewport_to_world_2d(transform, *last_moved) {
            *cursor_world_position = CursorWorldPosition(pos);
        }
    } else {
        for cursor_moved in cursor_moved_events {
            let (transform, cam) = camera.single();
            if let Ok(pos) = cam.viewport_to_world_2d(transform, cursor_moved.position) {
                *last_moved = cursor_moved.position;
                *cursor_world_position = CursorWorldPosition(pos);
            }
        }
    }
}
