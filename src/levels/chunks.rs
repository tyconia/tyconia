use crate::loading::TextureAssets;
use crate::GameState;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct Occupant(Entity);

#[derive(Debug, Component)]
pub struct Chunk<const N: usize> {
    location: Vec2,
    data: [[Option<Occupant>; N]; N],
}

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilemapPlugin)
            .add_systems(OnEnter(GameState::Playing), (render_chunks,));
    }
}

fn render_chunks(mut cmd: Commands, textures: Res<TextureAssets>) {
    const QUADRANT_SIDE_LENGTH: u32 = 64;
    let texture_handle: Handle<Image> = textures.infinite_io.clone();

    let map_size = TilemapSize {
        x: QUADRANT_SIDE_LENGTH * 2,
        y: QUADRANT_SIDE_LENGTH * 2,
    };
    let quadrant_size = TilemapSize {
        x: QUADRANT_SIDE_LENGTH,
        y: QUADRANT_SIDE_LENGTH,
    };
    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = cmd.spawn_empty().id();
    let tilemap_id = TilemapId(tilemap_entity);

    fill_tilemap_rect(
        TileTextureIndex(0),
        TilePos { x: 0, y: 0 },
        quadrant_size,
        tilemap_id,
        &mut cmd,
        &mut tile_storage,
    );

    //fill_tilemap_rect(
    //    TileTextureIndex(1),
    //    TilePos {
    //        x: QUADRANT_SIDE_LENGTH,
    //        y: 0,
    //    },
    //    quadrant_size,
    //    tilemap_id,
    //    &mut cmd,
    //    &mut tile_storage,
    //);
    //
    //fill_tilemap_rect(
    //    TileTextureIndex(2),
    //    TilePos {
    //        x: 0,
    //        y: QUADRANT_SIDE_LENGTH,
    //    },
    //    quadrant_size,
    //    tilemap_id,
    //    &mut cmd,
    //    &mut tile_storage,
    //);
    //
    //fill_tilemap_rect(
    //    TileTextureIndex(3),
    //    TilePos {
    //        x: QUADRANT_SIDE_LENGTH,
    //        y: QUADRANT_SIDE_LENGTH,
    //    },
    //    quadrant_size,
    //    tilemap_id,
    //    &mut cmd,
    //    &mut tile_storage,
    //);

    let tile_size = TilemapTileSize { x: 128.0, y: 128.0 };
    let grid_size = (TilemapTileSize { x: 125.0, y: 66.0 }).into();
    //let grid_size = (TilemapTileSize { x: -125.0, y: -66.0 }).into();
    let map_type = TilemapType::Isometric(IsoCoordSystem::Diamond);

    cmd.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        size: map_size,
        storage: tile_storage,
        texture: TilemapTexture::Single(texture_handle),
        tile_size,
        map_type,
        transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
        render_settings: TilemapRenderSettings { y_sort: true, render_chunk_size: UVec2::new(3, 1) },
        ..Default::default()
    });
}

fn initialize_chunks(mut cmd: Commands, textures: Res<TextureAssets>) {
    for x in (-10)..10 {
        for y in (-10)..10 {
            cmd.spawn(
                (Chunk {
                    location: Vec2::new(x as f32, y as f32),
                    data: [[None; 8]; 8],
                }),
            );
        }
    }

    cmd.spawn((
        Sprite::from_image(textures.infinite_io.clone()),
        Transform::from_translation(Vec3::new(117. / 2., 32., 0.)),
    ));
    cmd.spawn((
        Sprite::from_image(textures.infinite_io.clone()),
        Transform::from_translation(Vec3::new(0., 0., 1.)),
    ));
    cmd.spawn((
        Sprite::from_image(textures.infinite_io.clone()),
        Transform::from_translation(Vec3::new(117., 0., 1.)),
    ));
}
