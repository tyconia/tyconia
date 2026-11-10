use crate::levels::{
    access_side_direction, elevation_screen_offset, level_z, polygon_signed_area,
    rank_from_elevation, AccessKind, GroundTransform, IsoGrid, LevelCollider, LevelEditorState,
    LevelVertical, ObstacleCollider, RuntimeAccess, RuntimeFloor, ACCESS_Z_LAYER,
    FLOOR_SCREEN_RISE, FLOOR_Z_LAYER, PROP_Z_LAYER,
};
use crate::player::{Player, PlayerHeight};
use crate::GameState;
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::mesh::Mesh;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::sprite::AlphaMode2d;

mod mesh;
use mesh::{build_collider_mesh, build_floor_mesh, build_ramp_mesh, build_stairs_mesh};
pub use mesh::stair_step_count;

const CHECKER_TEXTURE_SIZE: u32 = 32;
const CHECKER_CELL_SIZE: u32 = 8;
const FLOOR_PLATE_THICKNESS: f32 = 8.0;
const STAIR_TARGET_RISE: f32 = 8.0;
const MIN_STAIRS: usize = 1;
const MAX_STAIRS: usize = 16;
const FADE_RATE: f32 = 10.0;
const FADE_EPSILON: f32 = 0.005;
const DIM_ALPHA: f32 = 0.35;

pub struct LevelGeometryVisualsPlugin;

impl Plugin for LevelGeometryVisualsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_geometry_assets).add_systems(
            Update,
            (
                sync_collider_meshes,
                sync_vertical_meshes,
                update_ranked_visuals,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Resource)]
struct GeometryMaterials {
    checker_texture: Handle<Image>,
}

#[derive(Component)]
struct ColliderMeshVisual;

#[derive(Component)]
struct VerticalMeshVisual;

#[derive(Component, Clone, Copy)]
pub struct RankedVisual {
    pub rank: i16,
}

#[derive(Component, Clone, Copy)]
pub struct VisualFade {
    pub alpha: f32,
}

#[derive(Component, Clone, Copy)]
struct AccessVisual {
    lower_rank: i16,
    upper_rank: i16,
    ground_y: f32,
}

impl AccessVisual {
    fn display_rank(&self, focus: i16) -> i16 {
        if self.upper_rank < focus.saturating_sub(1) {
            self.upper_rank
        } else if self.lower_rank > focus.saturating_add(1) {
            self.lower_rank
        } else {
            focus
        }
    }
}

fn setup_geometry_assets(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let checker_texture = images.add(checker_image());
    commands.insert_resource(GeometryMaterials { checker_texture });
}

fn checker_material(
    materials: &mut Assets<ColorMaterial>,
    texture: &Handle<Image>,
    alpha: f32,
) -> Handle<ColorMaterial> {
    materials.add(ColorMaterial {
        color: Color::WHITE.with_alpha(alpha),
        texture: Some(texture.clone()),
        alpha_mode: if alpha >= 1.0 - FADE_EPSILON {
            AlphaMode2d::Opaque
        } else {
            AlphaMode2d::Blend
        },
        ..default()
    })
}

fn visual_focus(state: &LevelEditorState, player: Option<&PlayerHeight>) -> Option<i16> {
    if state.active {
        Some(state.level)
    } else {
        player.map(|height| height.rank)
    }
}

fn visual_target_alpha(rank: i16, focus: Option<i16>, editor: bool) -> f32 {
    let Some(focus) = focus else {
        return 0.0;
    };
    target_alpha(visual_readability(rank, focus, editor))
}

fn iso_uv(grid: &IsoGrid, world: Vec2) -> Vec2 {
    let u = grid.step_u();
    let v = grid.step_v();
    let det = u.x * v.y - u.y * v.x;
    if det.abs() <= f32::EPSILON {
        return world / grid.cell_size.y.max(1.0);
    }

    Vec2::new(
        (world.x * v.y - world.y * v.x) / det,
        (u.x * world.y - u.y * world.x) / det,
    )
}

fn checker_image() -> Image {
    let size = CHECKER_TEXTURE_SIZE;
    let mut data = vec![0u8; (size * size * 4) as usize];
    for y in 0..size {
        for x in 0..size {
            let light = ((x / CHECKER_CELL_SIZE) + (y / CHECKER_CELL_SIZE)) % 2 == 0;
            let value = if light { 216 } else { 148 };
            let index = ((y * size + x) * 4) as usize;
            data[index] = value;
            data[index + 1] = value;
            data[index + 2] = value;
            data[index + 3] = 255;
        }
    }

    let mut sampler = ImageSamplerDescriptor::nearest();
    sampler.address_mode_u = ImageAddressMode::Repeat;
    sampler.address_mode_v = ImageAddressMode::Repeat;

    let mut image = Image::new_fill(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::Descriptor(sampler);
    image
}

fn sync_collider_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<GeometryMaterials>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    state: Res<LevelEditorState>,
    player: Query<&PlayerHeight, With<Player>>,
    grid: Res<IsoGrid>,
    mut colliders: Query<
        (Entity, &ObstacleCollider, &GroundTransform, &mut Transform),
        (
            With<LevelCollider>,
            Or<(
                Added<LevelCollider>,
                Changed<ObstacleCollider>,
                Changed<GroundTransform>,
            )>,
        ),
    >,
) {
    let focus = visual_focus(&state, player.get_single().ok());

    for (entity, collider, ground, mut transform) in &mut colliders {
        let rank = rank_from_elevation(collider.z_min);
        transform.translation.z = level_z(rank, PROP_Z_LAYER, ground.position.y);
        let mesh = build_collider_mesh(&grid, collider, ground);
        let alpha = visual_target_alpha(rank, focus, state.active);
        let material = checker_material(&mut color_materials, &materials.checker_texture, alpha);
        commands.entity(entity).insert((
            ColliderMeshVisual,
            RankedVisual { rank },
            VisualFade { alpha },
            Mesh2d(meshes.add(mesh)),
            MeshMaterial2d(material),
        ));
    }
}

fn sync_vertical_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<GeometryMaterials>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    state: Res<LevelEditorState>,
    player: Query<&PlayerHeight, With<Player>>,
    grid: Res<IsoGrid>,
    instances: Query<(Entity, &LevelVertical), Changed<LevelVertical>>,
    old_visuals: Query<(Entity, &Parent), With<VerticalMeshVisual>>,
) {
    let focus = visual_focus(&state, player.get_single().ok());

    for (instance_entity, vertical) in &instances {
        for (visual, parent) in &old_visuals {
            if parent.get() == instance_entity {
                commands.entity(visual).despawn_recursive();
            }
        }

        for floor in &vertical.floors {
            let mesh = build_floor_mesh(&grid, floor);
            let ground_y = if floor.cells.is_empty() {
                0.0
            } else {
                floor
                    .cells
                    .iter()
                    .map(|cell| grid.cell_to_world(cell.as_vec2()).y)
                    .sum::<f32>()
                    / floor.cells.len() as f32
            };
            let rank = floor.rank;
            let z = level_z(rank, FLOOR_Z_LAYER, ground_y);
            let alpha = visual_target_alpha(rank, focus, state.active);
            let material =
                checker_material(&mut color_materials, &materials.checker_texture, alpha);
            commands.entity(instance_entity).with_children(|parent| {
                parent.spawn((
                    VerticalMeshVisual,
                    RankedVisual { rank },
                    VisualFade { alpha },
                    Mesh2d(meshes.add(mesh)),
                    MeshMaterial2d(material),
                    Transform::from_xyz(0.0, 0.0, z),
                ));
            });
        }

        for access in &vertical.accesses {
            let mesh = match access.kind {
                AccessKind::Ramp => build_ramp_mesh(&grid, access),
                AccessKind::Stairs => build_stairs_mesh(&grid, access),
            };
            let ground_y = (access.from.y + access.to.y) * 0.5;
            let lower_rank = access.visual_rank();
            let upper_rank = rank_from_elevation(access.from_elevation.max(access.to_elevation));
            let access_visual = AccessVisual {
                lower_rank,
                upper_rank,
                ground_y,
            };
            let rank = focus
                .map(|focus| access_visual.display_rank(focus))
                .unwrap_or(lower_rank);
            let z = level_z(rank, ACCESS_Z_LAYER, ground_y);
            let alpha = visual_target_alpha(rank, focus, state.active);
            let material =
                checker_material(&mut color_materials, &materials.checker_texture, alpha);
            commands.entity(instance_entity).with_children(|parent| {
                parent.spawn((
                    VerticalMeshVisual,
                    access_visual,
                    RankedVisual { rank },
                    VisualFade { alpha },
                    Mesh2d(meshes.add(mesh)),
                    MeshMaterial2d(material),
                    Transform::from_xyz(0.0, 0.0, z),
                ));
            });
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VisualReadability {
    Normal,
    Dim,
    Hidden,
}

fn visual_readability(rank: i16, focus: i16, editor: bool) -> VisualReadability {
    if editor {
        match focus - rank {
            0 => VisualReadability::Normal,
            1 => VisualReadability::Dim,
            _ => VisualReadability::Hidden,
        }
    } else if rank > focus {
        VisualReadability::Hidden
    } else {
        VisualReadability::Normal
    }
}

fn target_alpha(readability: VisualReadability) -> f32 {
    match readability {
        VisualReadability::Normal => 1.0,
        VisualReadability::Dim => DIM_ALPHA,
        VisualReadability::Hidden => 0.0,
    }
}

fn update_ranked_visuals(
    state: Res<LevelEditorState>,
    player: Query<&PlayerHeight, With<Player>>,
    time: Res<Time>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    mut visuals: Query<(
        &mut RankedVisual,
        &mut VisualFade,
        &mut Visibility,
        &mut Transform,
        Option<&AccessVisual>,
        Option<&mut MeshMaterial2d<ColorMaterial>>,
        Option<&mut Sprite>,
    )>,
) {
    let focus = visual_focus(&state, player.get_single().ok()).unwrap_or(0);
    let smoothing = 1.0 - (-FADE_RATE * time.delta_secs()).exp();

    for (mut ranked, mut fade, mut visibility, mut transform, access, material, sprite) in
        &mut visuals
    {
        if let Some(access) = access {
            let display_rank = access.display_rank(focus);
            ranked.rank = display_rank;
            transform.translation.z = level_z(display_rank, ACCESS_Z_LAYER, access.ground_y);
        }

        let readability = visual_readability(ranked.rank, focus, state.active);
        let target = target_alpha(readability);
        fade.alpha += (target - fade.alpha) * smoothing;
        if (target - fade.alpha).abs() <= FADE_EPSILON {
            fade.alpha = target;
        }
        let alpha = fade.alpha.clamp(0.0, 1.0);

        *visibility = if alpha <= FADE_EPSILON {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };

        if let Some(material) = material {
            if let Some(material) = color_materials.get_mut(&material.0) {
                material.color = material.color.with_alpha(alpha);
                material.alpha_mode = if alpha >= 1.0 - FADE_EPSILON {
                    AlphaMode2d::Opaque
                } else {
                    AlphaMode2d::Blend
                };
            }
        }

        if let Some(mut sprite) = sprite {
            sprite.color = Color::WHITE.with_alpha(alpha);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mesh::triangulate_polygon;
    use super::*;
    use bevy::render::mesh::{Indices, VertexAttributeValues};

    #[test]
    fn triangulates_convex_and_concave_polygons() {
        let square = vec![
            Vec2::new(-1.0, -1.0),
            Vec2::new(1.0, -1.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(-1.0, 1.0),
        ];
        assert_eq!(triangulate_polygon(&square).len(), 2);

        let concave = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(2.0, 0.0),
            Vec2::new(2.0, 1.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(1.0, 2.0),
            Vec2::new(0.0, 2.0),
        ];
        assert_eq!(triangulate_polygon(&concave).len(), 4);
    }

    #[test]
    fn collider_mesh_has_top_and_sides() {
        let grid = IsoGrid::default();
        let collider = ObstacleCollider::iso(Vec2::new(32.0, 32.0));
        let ground = GroundTransform {
            position: Vec2::ZERO,
            rotation: 0.0,
        };
        let mesh = build_collider_mesh(&grid, &collider, &ground);
        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .and_then(|positions| positions.as_float3())
            .unwrap();
        let colors = mesh.attribute(Mesh::ATTRIBUTE_COLOR).unwrap();
        let max_y = positions
            .iter()
            .map(|position| position[1])
            .fold(f32::MIN, f32::max);
        assert!(positions.len() > 4);
        assert_eq!(positions.len(), colors.len());
        assert!(max_y > grid.step_u().length());
    }

    #[test]
    fn iso_uv_matches_grid_coordinates() {
        let grid = IsoGrid::default();
        let cell = Vec2::new(3.0, -2.0);
        assert!((iso_uv(&grid, grid.cell_to_world(cell)) - cell).length() < 0.001);
    }

    #[test]
    fn elevated_floor_top_uv_stays_on_the_ground_grid() {
        let grid = IsoGrid::default();
        let floor = RuntimeFloor {
            elevation: 1.0,
            rank: 1,
            cells: [IVec2::ZERO].into_iter().collect(),
        };
        let mesh = build_floor_mesh(&grid, &floor);
        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .and_then(|positions| positions.as_float3())
            .unwrap();
        let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0)
        else {
            panic!("floor mesh is missing float UVs");
        };
        let offset = Vec2::new(0.0, elevation_screen_offset(floor.elevation));

        let top_start = positions.len() - 4;
        for index in top_start..positions.len() {
            let world = Vec2::new(positions[index][0], positions[index][1]);
            let expected = iso_uv(&grid, world - offset);
            let actual = Vec2::from_array(uvs[index]);
            assert!((actual - expected).length() < 0.001);
        }

        let Some(Indices::U32(indices)) = mesh.indices() else {
            panic!("floor mesh is missing U32 indices");
        };
        assert!(indices[indices.len() - 6..]
            .iter()
            .all(|index| *index as usize >= top_start));
    }

    #[test]
    fn adjacent_floor_cells_mesh_as_one_surface() {
        let grid = IsoGrid::default();
        let cells = (0..2)
            .flat_map(|y| (0..2).map(move |x| IVec2::new(x, y)))
            .collect();
        let floor = RuntimeFloor {
            elevation: 0.0,
            rank: 0,
            cells,
        };
        let mesh = build_floor_mesh(&grid, &floor);
        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .and_then(|positions| positions.as_float3())
            .unwrap();

        assert_eq!(positions.len(), 36);

        let Some(Indices::U32(indices)) = mesh.indices() else {
            panic!("floor mesh is missing U32 indices");
        };
        let top_start = positions.len() - 4;
        assert!(indices[indices.len() - 6..]
            .iter()
            .all(|index| *index as usize >= top_start));
    }

    #[test]
    fn closer_ground_points_draw_in_front() {
        let near = level_z(0, FLOOR_Z_LAYER, -128.0);
        let far = level_z(0, FLOOR_Z_LAYER, 128.0);
        assert!(near > far);
        assert!(near < level_z(1, FLOOR_Z_LAYER, 0.0));
    }

    #[test]
    fn editor_readability_dims_lower_adjacent_and_hides_upper_ranks() {
        assert_eq!(visual_readability(2, 2, true), VisualReadability::Normal);
        assert_eq!(visual_readability(3, 2, true), VisualReadability::Hidden);
        assert_eq!(visual_readability(4, 2, true), VisualReadability::Hidden);
        assert_eq!(visual_readability(1, 2, true), VisualReadability::Dim);
        assert_eq!(visual_readability(0, 2, true), VisualReadability::Hidden);
    }

    #[test]
    fn gameplay_cutaway_hides_all_ranks_above_the_player() {
        assert_eq!(visual_readability(2, 2, false), VisualReadability::Normal);
        assert_eq!(visual_readability(3, 2, false), VisualReadability::Hidden);
        assert_eq!(visual_readability(4, 2, false), VisualReadability::Hidden);
        assert_eq!(visual_readability(1, 2, false), VisualReadability::Normal);
    }

    #[test]
    fn fade_targets_match_readability() {
        assert_eq!(target_alpha(VisualReadability::Normal), 1.0);
        assert_eq!(target_alpha(VisualReadability::Dim), DIM_ALPHA);
        assert_eq!(target_alpha(VisualReadability::Hidden), 0.0);
        assert_eq!(visual_target_alpha(3, Some(2), false), 0.0);
        assert_eq!(visual_target_alpha(2, Some(2), false), 1.0);
        assert_eq!(visual_target_alpha(2, None, false), 0.0);
    }

    #[test]
    fn lower_adjacent_access_renders_at_the_focused_level() {
        let access = AccessVisual {
            lower_rank: 1,
            upper_rank: 2,
            ground_y: 0.0,
        };

        assert_eq!(access.display_rank(2), 2);
        assert_eq!(access.display_rank(1), 1);
        assert_eq!(access.display_rank(3), 3);
        assert_eq!(access.display_rank(4), 2);

        let rank = access.display_rank(3);
        assert_eq!(
            visual_readability(rank, 3, false),
            VisualReadability::Normal
        );
        assert_eq!(visual_readability(rank, 3, true), VisualReadability::Normal);
    }

    #[test]
    fn stair_steps_follow_total_rise() {
        assert_eq!(stair_step_count(FLOOR_SCREEN_RISE), 5);
        assert_eq!(stair_step_count(FLOOR_SCREEN_RISE * 2.0), 9);
        assert_eq!(stair_step_count(0.0), 1);
    }

    #[test]
    fn stair_risers_span_the_full_width_in_uv_space() {
        let grid = IsoGrid::default();
        let access = RuntimeAccess {
            from: grid.cell_to_world(Vec2::ZERO),
            to: grid.cell_to_world(Vec2::new(4.0, 0.0)),
            width: 32.0,
            from_elevation: 0.0,
            to_elevation: 1.0,
            kind: AccessKind::Stairs,
        };
        let mesh = build_stairs_mesh(&grid, &access);
        let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0)
        else {
            panic!("stair mesh is missing float UVs");
        };

        let riser_uvs = &uvs[0..4];
        let min_u = riser_uvs
            .iter()
            .map(|uv| uv[0])
            .fold(f32::INFINITY, f32::min);
        let max_u = riser_uvs
            .iter()
            .map(|uv| uv[0])
            .fold(f32::NEG_INFINITY, f32::max);
        let min_v = riser_uvs
            .iter()
            .map(|uv| uv[1])
            .fold(f32::INFINITY, f32::min);
        let max_v = riser_uvs
            .iter()
            .map(|uv| uv[1])
            .fold(f32::NEG_INFINITY, f32::max);

        assert!((max_u - min_u - 32.0 / grid.step_u().length()).abs() < 0.001);
        assert!(max_v - min_v > 0.1);
    }

    #[test]
    fn ramp_and_stairs_draw_tops_after_vertical_faces() {
        let grid = IsoGrid::default();
        let width = grid.step_u().length();
        let from = grid.cell_to_world(Vec2::ZERO);
        let to = grid.cell_to_world(Vec2::new(1.0, 0.0));

        for kind in [AccessKind::Ramp, AccessKind::Stairs] {
            let access = RuntimeAccess {
                from,
                to,
                width,
                from_elevation: 0.0,
                to_elevation: 1.0,
                kind,
            };
            let mesh = match kind {
                AccessKind::Ramp => build_ramp_mesh(&grid, &access),
                AccessKind::Stairs => build_stairs_mesh(&grid, &access),
            };
            let positions = mesh
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .and_then(|positions| positions.as_float3())
                .unwrap();
            let Some(Indices::U32(indices)) = mesh.indices() else {
                panic!("access mesh is missing U32 indices");
            };

            let top_start = positions.len() - 4;
            assert!(indices[indices.len() - 6..]
                .iter()
                .all(|index| *index as usize >= top_start));
        }
    }
}
