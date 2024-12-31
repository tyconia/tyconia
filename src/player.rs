use crate::actions::{CursorWorldPosition, PlayerMovementAction, UiAction};
use crate::levels::chunks::OriginalColor;
use crate::loading::TextureAssets;
use crate::GameState;
use bevy::prelude::*;

pub struct PlayerPlugin;

#[derive(Component)]
pub struct Player;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), (spawn_player,))
            .add_systems(
                Update,
                (
                    move_player.run_if(on_event::<PlayerMovementAction>),
                    zoom_camera.run_if(on_event::<UiAction>),
                    highlight_tile.run_if(resource_changed::<CursorWorldPosition>),
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn spawn_player(mut commands: Commands, textures: Res<TextureAssets>) {
    commands
        .spawn((
            Sprite::from_image(textures.infinite_io.clone()),
            Transform::from_translation(Vec3::new(0., 0., 999.)),
            Player,
        ))
        .with_children(|children| {
            children.spawn((Camera2d, Msaa::Off, UiAntiAlias::Off));
        });
}

/// Zooms camera from Actions
fn zoom_camera(
    mut ui_action: EventReader<UiAction>,
    mut camera: Query<&mut OrthographicProjection, With<Camera>>,
    mut new_scale: Local<f32>,
) {
    let factor: f32 = ui_action
        .read()
        .filter_map(|ui_action| {
            if let UiAction::Zoom(factor) = ui_action {
                Some(factor)
            } else {
                None
            }
        })
        .sum();

    let mut camera = camera.single_mut();
    *new_scale = (camera.scale - factor / 12.).clamp(0.3, 1.8);
    camera.scale = camera.scale.lerp(*new_scale, 0.5);
}

fn move_player(
    time: Res<Time>,
    mut player_movement_action: EventReader<PlayerMovementAction>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    let player_movement = player_movement_action.read().last();
    if player_movement.is_none() {
        return;
    }
    let player_movement = player_movement.unwrap().0;
    let speed = 1900.;
    let movement = Vec3::new(
        player_movement.x * speed * time.delta_secs(),
        player_movement.y * speed * time.delta_secs(),
        0.,
    );
    for mut player_transform in &mut player_query {
        //player_transform.translation += movement;
        player_transform.translation = player_transform
            .translation
            .lerp(player_transform.translation + movement, 0.2)
    }
}

use bevy_ecs_tilemap::prelude::*;

#[derive(Component)]
struct HighlightedTile;

/// Highlights tile by Actions
fn highlight_tile(
    mut cmd: Commands,
    cursor_world_position: Res<CursorWorldPosition>,
    mut last_cursor_pos: Local<Vec2>,
    tilemap_q: Query<(
        &TilemapSize,
        &TilemapGridSize,
        &TilemapType,
        &TileStorage,
        &Transform,
    )>,
    mut highlighted_tiles_q: Query<
        (Entity, &mut TileColor, Option<&OriginalColor>),
        With<HighlightedTile>,
    >,
) {
    // Un-highlight any previously highlighted tiles
    for (highlighted_tile_entity, mut highlighted_tile_color, original_color) in
        highlighted_tiles_q.iter_mut()
    {
        if let Some(original_color) = original_color {
            *highlighted_tile_color = original_color.0;
        } else {
            *highlighted_tile_color = TileColor::default();
        }

        cmd.entity(highlighted_tile_entity)
            .remove::<HighlightedTile>();
    }

    for (map_size, grid_size, map_type, tile_storage, map_transform) in tilemap_q.iter() {
        // Grab the cursor position from the `Res<CursorPos>`
        let cursor_pos: Vec2 = cursor_world_position.0;
        // We need to make sure that the cursor's world position is correct relative to the map
        // due to any map transformation.
        let cursor_in_map_pos: Vec2 = {
            // Extend the cursor_pos vec3 by 0.0 and 1.0
            let cursor_pos = Vec4::from((cursor_pos, 0.0, 1.0));
            let cursor_in_map_pos = map_transform.compute_matrix().inverse() * cursor_pos;
            cursor_in_map_pos.xy()
        };
        // Once we have a world position we can transform it into a possible tile position.
        if let Some(tile_pos) =
            TilePos::from_world_pos(&cursor_in_map_pos, map_size, grid_size, map_type)
        {
            // Highlight the relevant tile's label
            if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                cmd.entity(tile_entity)
                    .insert(HighlightedTile)
                    .insert(TileColor(Color::srgb(0.6, 0.9, 0.6)));
            }
        }
    }
}
