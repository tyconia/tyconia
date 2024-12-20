use bevy::prelude::{ButtonInput, KeyCode, Res};

pub enum GameControl {
    Up,
    Down,
    Left,
    Right,
}

impl GameControl {
    pub fn pressed(&self, keyboard_input: &Res<ButtonInput<KeyCode>>) -> bool {
        match self {
            GameControl::Up => {
                keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp)
            }
            GameControl::Down => {
                keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown)
            }
            GameControl::Left => {
                keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft)
            }
            GameControl::Right => {
                keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight)
            }
        }
    }
}

pub fn get_movement(control: GameControl, input: &Res<ButtonInput<KeyCode>>) -> f32 {
    if control.pressed(input) {
        1.0
    } else {
        0.0
    }
}

//fn control_crosshair(
//    windows_query: Query<&Window, (With<PrimaryWindow>, Without<PlayerTurret>)>,
//    camera_query: Query<(&Camera, &GlobalTransform), (With<Camera2d>, Without<PlayerTurret>)>,
//
//    // body_query: Query<&Transform, (With<Player>, Without<PlayerTurret>)>,
//
//    // mut turret_query: Query<(&mut Transform, &mut PlayerTurret)>,
//    mut last_mouse_location: Local<Vec2>,
//    mut crosshair_query: Query<&mut Transform, With<PlayerCrosshair>>,
//) {
//    let window = windows_query.single();
//    let mut crosshair = crosshair_query.single_mut();
//    let (camera, camera_transform) = camera_query.single();
//
//    let cursor_screen_position = window.cursor_position().unwrap_or(*last_mouse_location);
//    // Save mouse location
//    *last_mouse_location = cursor_screen_position;
//
//    let cursor_position_in_world = camera
//        .viewport_to_world(camera_transform, cursor_screen_position)
//        .map(|ray| ray.origin.truncate())
//        .unwrap();
//
//    crosshair.translation = Vec3::new(cursor_position_in_world.x, cursor_position_in_world.y, 0.);
//    crosshair.rotation = Quat::IDENTITY;
//}
