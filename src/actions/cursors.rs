use crate::GameState;
use bevy::prelude::*;

/// Enables systems to update [`CursorWorldPosition`]
pub(crate) struct CursorsPlugin;

impl Plugin for CursorsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorWorldPosition>().add_systems(
            Update,
            (set_cursor.run_if(on_event::<CursorMoved>),).run_if(in_state(GameState::Playing)),
        );
    }
}

/// [`Resource`] position of the cursor projected to the world
#[derive(Default, Resource)]
pub struct CursorWorldPosition(pub Vec2);

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
