use crate::actions::{cursors::CursorWorldPosition, movement::MovementAction, ui::UiAction};
use crate::levels::*;
use crate::ui::*;
use crate::GameState;
use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerSpawnRequest>()
            .add_systems(OnEnter(GameState::Playing), (spawn_player,))
            .add_systems(
                Update,
                (
                    apply_player_spawn,
                    move_player.run_if(on_event::<MovementAction>),
                    zoom_camera,
                    //(un_highlight, (hightlight_building /* highlight_tile */,))
                    //    .chain()
                    //    .run_if(resource_changed::<CursorWorldPosition>),
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerCollider {
    pub radius: f32,
}

fn spawn_player(mut commands: Commands) {
    let mut inventory = crate::levels::Inventory::with_capacity(22);

    inventory.dump(vec![
        ItemEntry {
            item: COLLIDER_ITEM_ID.into(),
            quantity: INFINITE_QUANTITY,
        },
        ItemEntry {
            item: "auto_arm".into(),
            quantity: 20,
        },
        ItemEntry {
            item: "auto_arm".into(),
            quantity: 20,
        },
        ItemEntry {
            item: "mover_belt".into(),
            quantity: 80,
        },
        ItemEntry {
            item: "infinite_io".into(),
            quantity: 20,
        },
    ]);

    commands
        .spawn((
            Sprite::from_color(Color::srgb(0.95, 0.72, 0.2), Vec2::splat(22.0)),
            Transform::from_xyz(0.0, 0.0, 10.0),
            StateScoped(GameState::Playing),
            Player,
            PlayerCollider { radius: 10.0 },
            inventory,
            crate::hud::InventoryUISource {
                display_width: 6,
                display_height: 5,
            },
            crate::levels::InventoryActive::default(),
        ))
        .with_children(|children| {
            children.spawn((
                Camera2d,
                Msaa::Off,
                UiAntiAlias::Off,
                Transform::from_xyz(0.0, 0.0, 500.0),
            ));
        });
}

fn apply_player_spawn(
    mut spawn_request: ResMut<PlayerSpawnRequest>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    let Some(position) = spawn_request.0.take() else {
        return;
    };
    let Ok(mut transform) = player_query.get_single_mut() else {
        return;
    };

    transform.translation.x = position.x;
    transform.translation.y = position.y;
}

/// Zooms camera from Actions with linear interpolation
fn zoom_camera(
    mut ui_action: EventReader<UiAction>,
    mut camera: Query<&mut OrthographicProjection, With<Camera>>,
    mut new_scale: Local<f32>,
    time: Res<Time>,
) {
    let factor: isize = ui_action
        .read()
        .filter_map(|ui_action| ui_action.zoom())
        .sum();

    let Ok(mut camera) = camera.get_single_mut() else {
        return;
    };

    fn approx_inequal(a: f32, b: f32, tolerance: f32) -> bool {
        !((a - b).abs() <= tolerance)
    }

    if approx_inequal(camera.scale, *new_scale, 0.01) || factor != 0 {
        *new_scale = (*new_scale - factor as f32 * time.delta_secs()).clamp(0.25, 1.8);
    }

    camera.scale = camera.scale.lerp(*new_scale, 0.11);
}

/// TODO: Z indexing for obstacles
fn move_player(
    time: Res<Time>,
    mut player_movement_action: EventReader<MovementAction>,
    mut player_query: Query<(&mut Transform, &PlayerCollider), With<Player>>,
    obstacle_query: Query<(&GlobalTransform, &ObstacleCollider), Without<Player>>,
) {
    const SPEED: f32 = 320.0;

    let movement: Vec2 = player_movement_action
        .read()
        .map(|action| Vec2::from(action))
        .fold(Vec2::ZERO, |x, y| x + y)
        .normalize_or_zero();

    if movement == Vec2::ZERO {
        return;
    }

    let obstacles: Vec<[Vec2; 4]> = obstacle_query
        .iter()
        .map(|(transform, collider)| collider.world_corners(transform))
        .collect();

    let displacement = movement * SPEED * time.delta_secs().min(0.05);

    for (mut player_transform, collider) in &mut player_query {
        let position = move_and_slide(
            player_transform.translation.xy(),
            displacement,
            collider.radius,
            &obstacles,
        );
        player_transform.translation.x = position.x;
        player_transform.translation.y = position.y;
    }
}

use bevy_ecs_tilemap::prelude::*;

fn un_highlight(mut cmd: Commands, mut highlighted_tiles: HighlightedtilesQuery) {
    // Un-highlight any previously highlighted tiles
    for (highlighted_tile_entity, mut highlighted_tile_color, original_color) in
        highlighted_tiles.iter_mut()
    {
        if let Some(original_color) = original_color {
            *highlighted_tile_color = original_color.0;
        } else {
            *highlighted_tile_color = TileColor::default();
        }

        cmd.entity(highlighted_tile_entity)
            .remove::<HighlightedTile>();
    }
}

pub const HIGHLIGHT_CLR: Color = Color::srgba(
    168.0 / 255.0, // red: ~0.6588235
    255.0 / 255.0, // green: 1.0
    229.0 / 255.0, // blue: ~0.8980392
    250.0 / 255.0, // alpha: ~0.3921569
);

pub const NEGATIVE_HIGHLIGHT_CLR: Color = Color::srgba(
    255.0 / 255.0, // red: 1.0
    168.0 / 255.0, // green: ~0.6588235
    229.0 / 255.0, // blue: ~0.8980392
    240.0 / 255.0, // alpha: ~0.3921569
);

fn hightlight_building(
    mut cmd: Commands,
    cursor_world_position: Res<CursorWorldPosition>,
    building_tilemap: TilemapQuery<(Without<FloorTilemap>, With<BuildingTilemap>)>,
) {
    let Ok((_, map_size, grid_size, map_type, tile_storage, map_transform)) =
        building_tilemap.get_single()
    else {
        return;
    };

    cursor_tile_position(
        &*cursor_world_position,
        &*map_size,
        &*grid_size,
        &*map_type,
        &*map_transform,
    )
    .map(|tile_pos| {
        tile_storage.get(&tile_pos).map(|tile_entity| {
            cmd.entity(tile_entity)
                .insert(HighlightedTile)
                .insert(TileColor(HIGHLIGHT_CLR));
        });
    });
}

/// Highlights tile by Actions
fn highlight_tile(
    mut cmd: Commands,
    cursor_world_position: Res<CursorWorldPosition>,

    floor_tilemap: TilemapQuery<(With<FloorTilemap>, Without<BuildingTilemap>)>,
    building_tilemap: TilemapQuery<(Without<FloorTilemap>, With<BuildingTilemap>)>,
) {
    let Ok((_, map_size, grid_size, map_type, tile_storage, map_transform)) =
        floor_tilemap.get_single()
    else {
        return;
    };

    let occupied_tile = {
        let Ok((_, map_size, grid_size, map_type, tile_storage, map_transform)) =
            building_tilemap.get_single()
        else {
            return;
        };

        cursor_tile_position(
            &*cursor_world_position,
            &*map_size,
            &*grid_size,
            &*map_type,
            &*map_transform,
        )
        .map(|tile_pos| tile_storage.get(&tile_pos))
        .flatten()
    };

    if occupied_tile.is_some() {
        return;
    }

    cursor_tile_position(
        &*cursor_world_position,
        &*map_size,
        &*grid_size,
        &*map_type,
        &*map_transform,
    )
    .map(|tile_pos| {
        tile_storage.get(&tile_pos).map(|tile_entity| {
            cmd.entity(tile_entity)
                .insert(HighlightedTile)
                .insert(TileColor(HIGHLIGHT_CLR));
        });
    });
}

pub fn cursor_tile_position(
    cursor_world_position: &CursorWorldPosition,
    map_size: &TilemapSize,
    grid_size: &TilemapGridSize,
    map_type: &TilemapType,
    map_transform: &Transform,
) -> Option<TilePos> {
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
    TilePos::from_world_pos(&cursor_in_map_pos, map_size, grid_size, map_type)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_moves_to_level_spawn_request() {
        let mut app = App::new();
        app.init_resource::<PlayerSpawnRequest>()
            .add_systems(Update, apply_player_spawn);

        app.world_mut()
            .spawn((Player, Transform::from_xyz(10.0, 10.0, 10.0)));
        app.world_mut().resource_mut::<PlayerSpawnRequest>().0 = Some(Vec2::new(-20.0, 35.0));

        app.update();

        let transform = app
            .world_mut()
            .query_filtered::<&Transform, With<Player>>()
            .single(app.world());

        assert_eq!(transform.translation, Vec3::new(-20.0, 35.0, 10.0));
    }
}
