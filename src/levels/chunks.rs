use crate::loading::TextureAssets;
use crate::GameState;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

pub struct ChunkPlugin;

impl ChunkPlugin {
    pub const FLOOR_LAYER: f32 = 1.;
    pub const COUNTERTOP_LAYER: f32 = 2.;
}

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilemapPlugin).add_systems(
            OnEnter(GameState::Playing),
            (render_floors, render_countertops).chain(),
        );
    }
}

fn render_countertops(mut cmd: Commands, textures: Res<TextureAssets>) {
    const QUADRANT_SIDE_LENGTH: u32 = 32;
    let inserter: Handle<Image> = textures.isometric_inserters.clone();
    let infinite_io: Handle<Image> = textures.infinite_io.clone();
    let belts: Handle<Image> = textures.isometric_belts.clone();

    let map_size = TilemapSize {
        x: QUADRANT_SIDE_LENGTH * 2,
        y: QUADRANT_SIDE_LENGTH * 2,
    };

    let filled_sized = TilemapSize {
        x: QUADRANT_SIDE_LENGTH,
        y: QUADRANT_SIDE_LENGTH,
    };

    let mut floor_storage = TileStorage::empty(map_size);
    let tilemap_entity = cmd.spawn_empty().id();
    let tilemap_id = TilemapId(tilemap_entity);

    //fill_tilemap_rect(
    //    TileTextureIndex(0),
    //    TilePos { x: 0, y: 0 },
    //    filled_sized,
    //    tilemap_id,
    //    &mut cmd,
    //    &mut floor_storage,
    //);
    //
    //fill_tilemap_rect(
    //    TileTextureIndex(1),
    //    TilePos {
    //        x: QUADRANT_SIDE_LENGTH,
    //        y: QUADRANT_SIDE_LENGTH,
    //    },
    //    filled_sized,
    //    tilemap_id,
    //    &mut cmd,
    //    &mut floor_storage,
    //);
    //
    //
    for x in 0..20 {
        for y in 0..20 {
            let tile = cmd
                .spawn(TileBundle {
                    position: TilePos { x, y },
                    texture_index: TileTextureIndex(match (x % 2, y % 2) {
                        (0, _) => 1,
                        _ => 0,
                    }),
                    tilemap_id: TilemapId(tilemap_entity),
                    ..default()
                })
                .id();

            floor_storage.set(&TilePos { x, y }, tile);
        }
    }

    for y in 0..20 {
        let x = 20;
        let tile = cmd
            .spawn(TileBundle {
                position: TilePos { x, y },
                texture_index: TileTextureIndex(2),
                tilemap_id: TilemapId(tilemap_entity),
                ..default()
            })
            .id();

        floor_storage.set(&TilePos { x, y }, tile);
    }

    let tile_size = TilemapTileSize { x: 32.0, y: 32.0 };
    let grid_size = (TilemapTileSize { x: 32.0, y: 16.0 }).into();
    let map_type = TilemapType::Isometric(IsoCoordSystem::Diamond);

    cmd.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        size: map_size,
        storage: floor_storage.clone(),
        texture: TilemapTexture::Vector(
            [inserter.clone(), infinite_io.clone(), belts.clone()].into(),
        ),
        tile_size,
        map_type,
        transform: get_tilemap_center_transform(
            &map_size,
            &grid_size,
            &map_type,
            ChunkPlugin::COUNTERTOP_LAYER,
        ),
        render_settings: TilemapRenderSettings {
            y_sort: true,
            render_chunk_size: UVec2::new(3, 1),
        },
        ..Default::default()
    });
}

#[derive(Debug, Component)]
pub struct OriginalColor(pub TileColor);

fn render_floors(mut cmd: Commands, textures: Res<TextureAssets>) {
    const QUADRANT_SIDE_LENGTH: u32 = 32;
    let floor_handle: Handle<Image> = textures.isometric_kitchen_floors.clone();

    let map_size = TilemapSize {
        x: QUADRANT_SIDE_LENGTH * 2,
        y: QUADRANT_SIDE_LENGTH * 2,
    };

    let mut floor_storage = TileStorage::empty(map_size);
    let tilemap_entity = cmd.spawn_empty().id();
    let tilemap_id = TilemapId(tilemap_entity);

    cmd.entity(tilemap_id.0).with_children(|parent| {
        for x in 0..map_size.x {
            for y in 0..map_size.y {
                let color = {
                    /// Size of Tile on a checkerboard pattern
                    const FLOOR_TILE_SIZE: u32 = 2;
                    let tile_x = x / FLOOR_TILE_SIZE;
                    let tile_y = y / FLOOR_TILE_SIZE;
                    //if (tile_x + tile_y) % 2 == 0 {
                    //    TileColor(Color::BLACK)
                    //} else {
                    TileColor::default()
                    //}
                };

                let tile_pos = TilePos { x, y };

                let tile_entity = parent
                    .spawn((
                        TileBundle {
                            position: tile_pos,
                            tilemap_id,
                            texture_index: TileTextureIndex(0),
                            color,
                            ..Default::default()
                        },
                        OriginalColor(color),
                    ))
                    .id();
                floor_storage.set(&tile_pos, tile_entity);
            }
        }
    });

    let tile_size = TilemapTileSize { x: 32.0, y: 32.0 };
    let grid_size = (TilemapTileSize { x: 32.0, y: 16.0 }).into();
    let map_type = TilemapType::Isometric(IsoCoordSystem::Diamond);

    cmd.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        size: map_size,
        storage: floor_storage.clone(),
        texture: TilemapTexture::Vector([floor_handle.clone()].into()),
        tile_size,
        map_type,
        transform: get_tilemap_center_transform(
            &map_size,
            &grid_size,
            &map_type,
            ChunkPlugin::FLOOR_LAYER,
        ),
        render_settings: TilemapRenderSettings {
            y_sort: true,
            render_chunk_size: UVec2::new(3, 1),
        },
        ..Default::default()
    });
}
