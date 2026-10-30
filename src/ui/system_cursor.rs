use bevy::prelude::*;
use bevy::{window::SystemCursorIcon, winit::cursor::CursorIcon};

pub struct SystemCursorPlugin;

impl Plugin for SystemCursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CursorEvent>()
            .init_resource::<CurrentCursor>()
            .add_systems(Update, (handle_cursor_queue, reskin_system_cursor).chain());
    }
}

#[derive(Debug, Resource, Default)]
pub struct CurrentCursor(SystemCursorIcon);

#[derive(Debug, Event, Default)]
pub struct CursorEvent(SystemCursorIcon);

impl From<SystemCursorIcon> for CursorEvent {
    fn from(value: SystemCursorIcon) -> Self {
        Self(value)
    }
}

pub type CurrentCursorChannel<'a> = EventWriter<'a, CursorEvent>;

fn handle_cursor_queue(
    mut cursors: EventReader<CursorEvent>,
    mut current_cursor: ResMut<CurrentCursor>,
) {
    for cursor in cursors.read() {
        if current_cursor.0 != cursor.0 {
            current_cursor.0 = cursor.0;
        }
    }
}

fn reskin_system_cursor(
    mut cmd: Commands,
    windows: Query<Entity, With<Window>>,
    current_cursor: Res<CurrentCursor>,
) {
    if !current_cursor.is_changed() {
        return;
    }

    for window in windows.iter() {
        cmd.entity(window)
            .insert(CursorIcon::System(current_cursor.0));
    }
}
