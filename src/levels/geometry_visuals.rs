use crate::levels::{
    access_side_direction, elevation_screen_offset, iso_corners, level_z, point_in_polygon,
    polygon_signed_area, rank_from_elevation, AccessKind, GroundTransform, IsoGrid, LevelCollider,
    LevelEditorState, LevelVertical, ObstacleCollider, OcclusionRole, RuntimeAccess, RuntimeFloor,
    SceneDeclared, ACCESS_Z_LAYER, FLOOR_SCREEN_RISE, FLOOR_Z_LAYER, GROUND_EPSILON,
    PLAYER_BODY_HEIGHT, PROP_Z_LAYER,
};
use crate::player::{Player, PlayerHeight};
use crate::GameState;
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::math::Rect;
use bevy::prelude::*;
use bevy::render::mesh::Mesh;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::sprite::AlphaMode2d;
use std::collections::HashSet;

mod mesh;
pub use mesh::stair_step_count;
use mesh::{build_collider_mesh, build_floor_mesh, build_ramp_mesh, build_stairs_mesh};

const CHECKER_TEXTURE_SIZE: u32 = 32;
const CHECKER_CELL_SIZE: u32 = 8;
const FLOOR_PLATE_THICKNESS: f32 = 8.0;
const STAIR_TARGET_RISE: f32 = 8.0;
const MIN_STAIRS: usize = 1;
const MAX_STAIRS: usize = 16;
const FADE_RATE: f32 = 10.0;
const FADE_EPSILON: f32 = 0.005;
const DIM_ALPHA: f32 = 0.35;
const XRAY_ALPHA: f32 = 0.22;
const EDITOR_UPPER_ALPHA: f32 = 0.18;
const SURFACE_SOLID_MARGIN: f32 = 0.12;
const OCCLUSION_MARGIN: f32 = 8.0;
const STRUCTURE_DEPTH_EPSILON: f32 = 0.01;
const OCCLUSION_PLAYER_HALF_WIDTH: f32 = 18.0;
const OCCLUSION_BODY_HEIGHT: f32 = PLAYER_BODY_HEIGHT * FLOOR_SCREEN_RISE;

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

/// The continuous elevation range a walkable surface occupies. Visibility is
/// based on this rather than the rounded rank, so ramps do not pop as the
/// player crosses an elevation boundary.
#[derive(Component, Clone, Copy)]
struct SurfaceVisual {
    min_elevation: f32,
    max_elevation: f32,
}

/// Screen-space footprint used to decide whether a surface above the player
/// would actually cover them.
#[derive(Component, Clone)]
enum SurfaceFootprint {
    Cells(HashSet<IVec2>),
    Polygon(Vec<Vec2>),
}

impl SurfaceFootprint {
    fn contains(&self, grid: &IsoGrid, elevation: f32, point: Vec2) -> bool {
        match self {
            SurfaceFootprint::Cells(cells) => {
                let adjusted = point - Vec2::Y * elevation_screen_offset(elevation);
                cells.contains(&grid.snap_cell(adjusted))
            }
            SurfaceFootprint::Polygon(vertices) => point_in_polygon(point, vertices),
        }
    }

    /// Probe slightly around the player so walking along a surface edge does
    /// not rapidly toggle the fade.
    fn occludes(&self, grid: &IsoGrid, elevation: f32, point: Vec2) -> bool {
        const PROBES: [Vec2; 5] = [Vec2::ZERO, Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y];

        PROBES
            .iter()
            .any(|direction| self.contains(grid, elevation, point + *direction * OCCLUSION_MARGIN))
    }
}

/// World-space footprint of a structure volume. The polygon is already resolved
/// to world/screen space by `StructureOcclusion::from_scene`/`from_collider`.
#[derive(Clone, Debug)]
pub struct OcclusionFootprint {
    pub points: Vec<Vec2>,
}

impl OcclusionFootprint {
    pub fn new(points: Vec<Vec2>) -> Self {
        Self { points }
    }

    pub fn contains_ground(&self, point: Vec2) -> bool {
        point_in_polygon(point, &self.points)
    }

    fn ground_bounds(&self) -> Option<Rect> {
        if self.points.is_empty() {
            return None;
        }
        let mut min = Vec2::splat(f32::INFINITY);
        let mut max = Vec2::splat(f32::NEG_INFINITY);
        for point in &self.points {
            min = min.min(*point);
            max = max.max(*point);
        }
        Some(Rect::from_corners(min, max))
    }

    /// Following `level_z`, smaller ground Y values are closer to the camera.
    fn front_y(&self) -> f32 {
        self.ground_bounds()
            .map(|bounds| bounds.min.y)
            .unwrap_or(f32::INFINITY)
    }
}

/// A structure that participates in the elevation-aware cutaway rules.
/// Floors and accesses use `SurfaceVisual`; buildings use this component.
#[derive(Component, Clone, Debug)]
pub struct StructureOcclusion {
    pub footprint: OcclusionFootprint,
    pub z_min: f32,
    pub z_max: f32,
    pub role: OcclusionRole,
}

impl StructureOcclusion {
    pub fn from_scene(scene: &SceneDeclared) -> Option<Self> {
        let occlusion = scene.occlusion.as_ref()?;
        let points = if occlusion.footprint.is_empty() {
            iso_corners(occlusion.half_extents)
                .map(|point| point * scene.scale + scene.position)
                .to_vec()
        } else {
            occlusion
                .footprint
                .iter()
                .map(|point| *point * scene.scale + scene.position)
                .collect()
        };

        Some(Self {
            footprint: OcclusionFootprint::new(points),
            z_min: occlusion.z_min,
            z_max: occlusion.z_min + occlusion.height.max(0.01),
            role: occlusion.role,
        })
    }

    pub fn from_collider(collider: &ObstacleCollider, ground: &GroundTransform) -> Self {
        Self {
            footprint: OcclusionFootprint::new(ground.world_corners(collider)),
            z_min: collider.z_min,
            z_max: collider.z_max(),
            role: collider.role,
        }
    }
}

#[derive(Clone, Copy)]
struct StructurePlayer {
    screen: Vec2,
    ground: Vec2,
    elevation: f32,
}

#[derive(Clone, Copy)]
struct VisualFocus {
    elevation: f32,
    rank: i16,
    editor: bool,
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

fn visual_focus(state: &LevelEditorState, player: Option<&PlayerHeight>) -> Option<VisualFocus> {
    if state.active {
        Some(VisualFocus {
            elevation: state.level as f32,
            rank: state.level,
            editor: true,
        })
    } else {
        player.map(|height| VisualFocus {
            elevation: height.elevation,
            rank: height.rank,
            editor: false,
        })
    }
}

/// Visibility for props, actors, and scene sprites, which keep the original
/// rank cutaway behavior.
fn ranked_target_alpha(rank: i16, focus: Option<VisualFocus>) -> f32 {
    let Some(focus) = focus else {
        return 0.0;
    };

    if focus.editor {
        match focus.rank - rank {
            0 => 1.0,
            1 => DIM_ALPHA,
            _ => 0.0,
        }
    } else if rank > focus.rank {
        0.0
    } else {
        1.0
    }
}

/// Visibility for walkable surfaces. In gameplay, an upper surface stays solid
/// unless its footprint covers the player, in which case it becomes an x-ray
/// layer. The editor keeps the focused floor solid and shows one floor above
/// and below for context.
fn surface_target_alpha(
    surface: &SurfaceVisual,
    footprint: Option<&SurfaceFootprint>,
    focus: Option<VisualFocus>,
    grid: &IsoGrid,
    player_screen: Option<Vec2>,
) -> f32 {
    let Some(focus) = focus else {
        return 0.0;
    };

    if focus.editor {
        let current = surface.min_elevation <= focus.elevation + 0.5
            && surface.max_elevation >= focus.elevation - 0.5;
        if current {
            return 1.0;
        }

        let below = focus.elevation - surface.max_elevation;
        if below > 0.5 && below <= 1.5 {
            return DIM_ALPHA;
        }

        let above = surface.min_elevation - focus.elevation;
        if above > 0.5 && above <= 1.5 {
            return EDITOR_UPPER_ALPHA;
        }

        return 0.0;
    }

    if surface.min_elevation <= focus.elevation + SURFACE_SOLID_MARGIN {
        return 1.0;
    }

    match (footprint, player_screen) {
        (Some(footprint), Some(player_screen))
            if footprint.occludes(grid, surface.min_elevation, player_screen) =>
        {
            XRAY_ALPHA
        }
        _ => 1.0,
    }
}

fn player_structure_pose(player: Option<(&Transform, &PlayerHeight)>) -> Option<StructurePlayer> {
    player.map(|(transform, height)| StructurePlayer {
        screen: transform.translation.xy() + Vec2::Y * elevation_screen_offset(height.elevation),
        ground: transform.translation.xy(),
        elevation: height.elevation,
    })
}

/// Convex hull (counter-clockwise) of a point set via Andrew's monotone chain.
/// Returns the input unchanged when it has fewer than three distinct points.
fn convex_hull(points: &[Vec2]) -> Vec<Vec2> {
    let mut points = points.to_vec();
    points.sort_by(|a, b| {
        a.x.partial_cmp(&b.x)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal))
    });
    points.dedup_by(|a, b| (*a - *b).length_squared() <= 1e-6);
    if points.len() < 3 {
        return points;
    }

    let cross = |o: Vec2, a: Vec2, b: Vec2| (a - o).perp_dot(b - o);
    let mut hull: Vec<Vec2> = Vec::with_capacity(points.len() + 1);
    for &point in &points {
        while hull.len() >= 2 && cross(hull[hull.len() - 2], hull[hull.len() - 1], point) <= 0.0 {
            hull.pop();
        }
        hull.push(point);
    }
    let lower = hull.len() + 1;
    for &point in points.iter().rev() {
        while hull.len() >= lower && cross(hull[hull.len() - 2], hull[hull.len() - 1], point) <= 0.0
        {
            hull.pop();
        }
        hull.push(point);
    }
    hull.pop();
    hull
}

fn edge_intersection(a: Vec2, b: Vec2, c: Vec2, d: Vec2) -> Vec2 {
    let r = b - a;
    let s = d - c;
    let denominator = r.perp_dot(s);
    if denominator.abs() <= f32::EPSILON {
        return b;
    }
    a + r * ((c - a).perp_dot(s) / denominator)
}

/// Sutherland-Hodgman clip of `subject` against a convex, counter-clockwise
/// `clip` polygon. Returns the (possibly empty) intersection polygon.
fn clip_polygon(subject: &[Vec2], clip: &[Vec2]) -> Vec<Vec2> {
    if subject.len() < 3 || clip.len() < 3 {
        return Vec::new();
    }

    let side = |a: Vec2, b: Vec2, point: Vec2| (b - a).perp_dot(point - a);
    let mut output = subject.to_vec();

    for index in 0..clip.len() {
        if output.is_empty() {
            break;
        }
        let a = clip[index];
        let b = clip[(index + 1) % clip.len()];
        let input = std::mem::take(&mut output);
        let mut previous = *input.last().unwrap();
        let mut previous_inside = side(a, b, previous) >= 0.0;

        for &current in &input {
            let current_inside = side(a, b, current) >= 0.0;
            if current_inside {
                if !previous_inside {
                    output.push(edge_intersection(previous, current, a, b));
                }
                output.push(current);
            } else if previous_inside {
                output.push(edge_intersection(previous, current, a, b));
            }
            previous = current;
            previous_inside = current_inside;
        }
    }

    output
}

/// The player's body as a screen-space polygon: an iso diamond footprint swept
/// over the body height. More accurate than an axis-aligned rectangle.
fn player_body_polygon(player: StructurePlayer) -> Vec<Vec2> {
    let half = Vec2::new(
        OCCLUSION_PLAYER_HALF_WIDTH + OCCLUSION_MARGIN,
        (OCCLUSION_PLAYER_HALF_WIDTH + OCCLUSION_MARGIN) * 0.5,
    );
    let half_height = (OCCLUSION_BODY_HEIGHT + OCCLUSION_MARGIN * 2.0) * 0.5;
    let bottom = iso_corners(half).map(|point| point + player.screen - Vec2::Y * half_height);
    let top = iso_corners(half).map(|point| point + player.screen + Vec2::Y * half_height);
    let mut points = bottom.to_vec();
    points.extend_from_slice(&top);
    convex_hull(&points)
}

/// The screen silhouette of a structure volume: its footprint swept between
/// `z_min` and `z_max`.
fn structure_silhouette(structure: &StructureOcclusion) -> Vec<Vec2> {
    let low = structure.z_min.min(structure.z_max);
    let high = structure.z_min.max(structure.z_max);
    let mut points = Vec::with_capacity(structure.footprint.points.len() * 2);
    points.extend(
        structure
            .footprint
            .points
            .iter()
            .map(|point| *point + Vec2::Y * elevation_screen_offset(low)),
    );
    points.extend(
        structure
            .footprint
            .points
            .iter()
            .map(|point| *point + Vec2::Y * elevation_screen_offset(high)),
    );
    convex_hull(&points)
}

/// Fraction of the player body hidden by the structure, using true polygon
/// overlap instead of an axis-aligned bounding-box approximation.
fn polygon_coverage(structure: &StructureOcclusion, player: StructurePlayer) -> f32 {
    let body = player_body_polygon(player);
    let body_area = polygon_signed_area(&body).abs();
    if body_area <= f32::EPSILON {
        return 0.0;
    }

    let silhouette = structure_silhouette(structure);
    if silhouette.len() < 3 {
        return 0.0;
    }

    let clipped = clip_polygon(&body, &silhouette);
    if clipped.len() < 3 {
        return 0.0;
    }

    (polygon_signed_area(&clipped).abs() / body_area).clamp(0.0, 1.0)
}

fn editor_structure_alpha(structure: &StructureOcclusion, focus: VisualFocus) -> f32 {
    let current =
        structure.z_min <= focus.elevation + 0.5 && structure.z_max >= focus.elevation - 0.5;
    if current {
        return 1.0;
    }

    let below = focus.elevation - structure.z_max;
    if below > 0.5 && below <= 1.5 {
        return DIM_ALPHA;
    }

    let above = structure.z_min - focus.elevation;
    if above > 0.5 && above <= 1.5 {
        return EDITOR_UPPER_ALPHA;
    }

    0.0
}

/// Visibility for buildings and other solid structure volumes.
///
/// Unlike the legacy rank cutaway, this uses the player's continuous elevation
/// and the structure's vertical span:
/// - a structure entirely below the player stays solid context,
/// - a roof/ceiling above an inside player is removed so the interior opens up,
/// - an upper storey covering an inside player becomes an x-ray layer,
/// - a facade intersecting the player and sitting in front of them is ghosted
///   only around the screen-space player body.
fn structure_target_alpha(
    structure: &StructureOcclusion,
    focus: Option<VisualFocus>,
    player: Option<StructurePlayer>,
) -> f32 {
    let Some(focus) = focus else {
        return 0.0;
    };
    if focus.editor {
        return editor_structure_alpha(structure, focus);
    }

    let Some(player) = player else {
        return 0.0;
    };

    let feet = player.elevation;
    let head = feet + PLAYER_BODY_HEIGHT;
    let covers_ground = structure.footprint.contains_ground(player.ground);
    let is_roof = structure.role.is_roof_like();
    let entirely_above = structure.z_min >= head - GROUND_EPSILON;
    let entirely_below = structure.z_max <= feet + GROUND_EPSILON;
    let intersects_body = structure.z_min < head && structure.z_max > feet;
    let in_front = structure.footprint.front_y() < player.ground.y - STRUCTURE_DEPTH_EPSILON;

    if is_roof && covers_ground && entirely_above {
        return 0.0;
    }

    if entirely_below {
        return 1.0;
    }

    if entirely_above {
        if covers_ground {
            return XRAY_ALPHA;
        }
        if is_roof && in_front && polygon_coverage(structure, player) > 0.0 {
            return XRAY_ALPHA;
        }
        return 1.0;
    }

    let overlap = polygon_coverage(structure, player);
    if intersects_body && in_front && overlap > 0.0 {
        return (1.0 - overlap * (1.0 - XRAY_ALPHA)).clamp(XRAY_ALPHA, 1.0);
    }

    1.0
}

fn player_screen_position(player: Option<(&Transform, &PlayerHeight)>) -> Option<Vec2> {
    player.map(|(transform, height)| {
        transform.translation.xy() + Vec2::Y * elevation_screen_offset(height.elevation)
    })
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
    player: Query<(&Transform, &PlayerHeight), With<Player>>,
    grid: Res<IsoGrid>,
    mut colliders: Query<
        (Entity, &ObstacleCollider, &GroundTransform, &mut Transform),
        (
            With<LevelCollider>,
            Without<Player>,
            Or<(
                Added<LevelCollider>,
                Changed<ObstacleCollider>,
                Changed<GroundTransform>,
            )>,
        ),
    >,
) {
    let player = player.get_single().ok();
    let focus = visual_focus(&state, player.map(|(_, height)| height));
    let structure_player = player_structure_pose(player);

    for (entity, collider, ground, mut transform) in &mut colliders {
        let rank = rank_from_elevation(collider.z_min);
        transform.translation.z = level_z(rank, PROP_Z_LAYER, ground.position.y);
        let mesh = build_collider_mesh(&grid, collider, ground);
        let structure = StructureOcclusion::from_collider(collider, ground);
        let alpha = if collider.role == OcclusionRole::Prop {
            ranked_target_alpha(rank, focus)
        } else {
            structure_target_alpha(&structure, focus, structure_player)
        };
        let material = checker_material(&mut color_materials, &materials.checker_texture, alpha);
        commands.entity(entity).insert((
            ColliderMeshVisual,
            RankedVisual { rank },
            structure,
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
    player: Query<(&Transform, &PlayerHeight), With<Player>>,
    grid: Res<IsoGrid>,
    instances: Query<(Entity, &LevelVertical), Changed<LevelVertical>>,
    old_visuals: Query<(Entity, &Parent), With<VerticalMeshVisual>>,
) {
    let player = player.get_single().ok();
    let focus = visual_focus(&state, player.map(|(_, height)| height));
    let player_screen = player_screen_position(player);

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
            let surface = SurfaceVisual {
                min_elevation: floor.elevation,
                max_elevation: floor.elevation,
            };
            let footprint = SurfaceFootprint::Cells(floor.cells.iter().copied().collect());
            let alpha =
                surface_target_alpha(&surface, Some(&footprint), focus, &grid, player_screen);
            let material =
                checker_material(&mut color_materials, &materials.checker_texture, alpha);
            commands.entity(instance_entity).with_children(|parent| {
                parent.spawn((
                    VerticalMeshVisual,
                    RankedVisual { rank },
                    surface,
                    footprint.clone(),
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
                .map(|focus| access_visual.display_rank(focus.rank))
                .unwrap_or(lower_rank);
            let z = level_z(rank, ACCESS_Z_LAYER, ground_y);
            let surface = SurfaceVisual {
                min_elevation: access.from_elevation.min(access.to_elevation),
                max_elevation: access.from_elevation.max(access.to_elevation),
            };
            let footprint = SurfaceFootprint::Polygon(access_screen_footprint(&grid, access));
            let alpha =
                surface_target_alpha(&surface, Some(&footprint), focus, &grid, player_screen);
            let material =
                checker_material(&mut color_materials, &materials.checker_texture, alpha);
            commands.entity(instance_entity).with_children(|parent| {
                parent.spawn((
                    VerticalMeshVisual,
                    access_visual,
                    surface,
                    footprint.clone(),
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

/// Screen-space trapezoid covering the top of a ramp or staircase.
fn access_screen_footprint(grid: &IsoGrid, access: &RuntimeAccess) -> Vec<Vec2> {
    let (low_ground, high_ground, low_elevation, high_elevation) =
        if access.from_elevation <= access.to_elevation {
            (
                access.from,
                access.to,
                access.from_elevation,
                access.to_elevation,
            )
        } else {
            (
                access.to,
                access.from,
                access.to_elevation,
                access.from_elevation,
            )
        };
    let side = access_side_direction(grid, access.from, access.to) * access.width * 0.5;
    let low = low_ground + Vec2::Y * elevation_screen_offset(low_elevation);
    let high = high_ground + Vec2::Y * elevation_screen_offset(high_elevation);

    vec![low + side, high + side, high - side, low - side]
}

fn update_ranked_visuals(
    state: Res<LevelEditorState>,
    grid: Res<IsoGrid>,
    player: Query<(&Transform, &PlayerHeight), With<Player>>,
    time: Res<Time>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    mut visuals: Query<
        (
            &mut RankedVisual,
            &mut VisualFade,
            &mut Visibility,
            &mut Transform,
            Option<&AccessVisual>,
            Option<&SurfaceVisual>,
            Option<&SurfaceFootprint>,
            Option<&StructureOcclusion>,
            Option<&mut MeshMaterial2d<ColorMaterial>>,
            Option<&mut Sprite>,
        ),
        Without<Player>,
    >,
) {
    let player = player.get_single().ok();
    let focus = visual_focus(&state, player.map(|(_, height)| height));
    let player_screen = player_screen_position(player);
    let structure_player = player_structure_pose(player);
    let focus_rank = focus.map(|focus| focus.rank).unwrap_or(0);
    let smoothing = 1.0 - (-FADE_RATE * time.delta_secs()).exp();

    for (
        mut ranked,
        mut fade,
        mut visibility,
        mut transform,
        access,
        surface,
        footprint,
        structure,
        material,
        sprite,
    ) in &mut visuals
    {
        if let Some(access) = access {
            let display_rank = access.display_rank(focus_rank);
            ranked.rank = display_rank;
            transform.translation.z = level_z(display_rank, ACCESS_Z_LAYER, access.ground_y);
        }

        let target = if let Some(structure) = structure {
            if structure.role == OcclusionRole::Prop {
                ranked_target_alpha(ranked.rank, focus)
            } else {
                structure_target_alpha(structure, focus, structure_player)
            }
        } else if let Some(surface) = surface {
            surface_target_alpha(surface, footprint, focus, &grid, player_screen)
        } else {
            ranked_target_alpha(ranked.rank, focus)
        };
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

    fn focus(elevation: f32, editor: bool) -> VisualFocus {
        VisualFocus {
            elevation,
            rank: rank_from_elevation(elevation),
            editor,
        }
    }

    fn surface(elevation: f32) -> SurfaceVisual {
        SurfaceVisual {
            min_elevation: elevation,
            max_elevation: elevation,
        }
    }

    fn structure(
        points: Vec<Vec2>,
        z_min: f32,
        z_max: f32,
        role: OcclusionRole,
    ) -> StructureOcclusion {
        StructureOcclusion {
            footprint: OcclusionFootprint::new(points),
            z_min,
            z_max,
            role,
        }
    }

    fn structure_player(ground: Vec2, elevation: f32) -> StructurePlayer {
        StructurePlayer {
            screen: ground + Vec2::Y * elevation_screen_offset(elevation),
            ground,
            elevation,
        }
    }

    fn player_building(offset: Vec2) -> Vec<Vec2> {
        iso_corners(Vec2::splat(24.0))
            .map(|point| point + offset)
            .to_vec()
    }

    #[test]
    fn foreground_structure_ghosts_only_where_it_covers_the_player() {
        let grid = IsoGrid::default();
        let ground = grid.cell_to_world(Vec2::ZERO);
        let structure = structure(
            player_building(ground + Vec2::new(0.0, -16.0)),
            0.0,
            1.0,
            OcclusionRole::Facade,
        );
        let player = structure_player(ground, 0.0);

        let alpha = structure_target_alpha(&structure, Some(focus(0.0, false)), Some(player));

        assert!((XRAY_ALPHA..1.0).contains(&alpha), "alpha was {alpha}");
    }

    #[test]
    fn structure_below_the_player_stays_solid_context() {
        let grid = IsoGrid::default();
        let ground = grid.cell_to_world(Vec2::ZERO);
        let structure = structure(player_building(ground), -2.0, -0.2, OcclusionRole::Facade);
        let player = structure_player(ground, 0.0);

        assert_eq!(
            structure_target_alpha(&structure, Some(focus(0.0, false)), Some(player)),
            1.0
        );
    }

    #[test]
    fn structure_behind_the_player_stays_solid() {
        let grid = IsoGrid::default();
        let ground = grid.cell_to_world(Vec2::ZERO);
        let structure = structure(
            player_building(ground + Vec2::new(0.0, 40.0)),
            0.0,
            1.0,
            OcclusionRole::Facade,
        );
        let player = structure_player(ground, 0.0);

        assert_eq!(
            structure_target_alpha(&structure, Some(focus(0.0, false)), Some(player)),
            1.0
        );
    }

    #[test]
    fn structure_with_empty_footprint_stays_solid() {
        let grid = IsoGrid::default();
        let ground = grid.cell_to_world(Vec2::ZERO);
        let structure = structure(Vec::new(), 0.0, 1.0, OcclusionRole::Facade);
        let player = structure_player(ground, 0.0);

        assert_eq!(
            structure_target_alpha(&structure, Some(focus(0.0, false)), Some(player)),
            1.0
        );
    }

    #[test]
    fn convex_hull_drops_interior_points() {
        let hull = convex_hull(&[
            Vec2::new(-1.0, -1.0),
            Vec2::new(1.0, -1.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(-1.0, 1.0),
            Vec2::ZERO,
        ]);

        assert_eq!(hull.len(), 4);
        assert!((polygon_signed_area(&hull).abs() - 4.0).abs() < 0.001);
    }

    #[test]
    fn clip_polygon_intersects_overlapping_squares() {
        let subject = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(2.0, 0.0),
            Vec2::new(2.0, 2.0),
            Vec2::new(0.0, 2.0),
        ];
        let clip = vec![
            Vec2::new(1.0, 1.0),
            Vec2::new(3.0, 1.0),
            Vec2::new(3.0, 3.0),
            Vec2::new(1.0, 3.0),
        ];

        let intersection = clip_polygon(&subject, &clip);
        assert!((polygon_signed_area(&intersection).abs() - 1.0).abs() < 0.001);
    }

    #[test]
    fn player_body_polygon_is_centered_on_the_player() {
        let player = structure_player(Vec2::new(50.0, 20.0), 0.0);
        let body = player_body_polygon(player);
        let center = body.iter().copied().sum::<Vec2>() / body.len() as f32;

        assert!(center.distance(player.screen) < 0.001);
    }

    #[test]
    fn roof_above_an_inside_player_is_removed() {
        let grid = IsoGrid::default();
        let ground = grid.cell_to_world(Vec2::ZERO);
        let structure = structure(player_building(ground), 2.0, 2.5, OcclusionRole::Roof);
        let player = structure_player(ground, 0.0);

        assert_eq!(
            structure_target_alpha(&structure, Some(focus(0.0, false)), Some(player)),
            0.0
        );
    }

    #[test]
    fn upper_storey_over_an_inside_player_becomes_xray() {
        let grid = IsoGrid::default();
        let ground = grid.cell_to_world(Vec2::ZERO);
        let structure = structure(player_building(ground), 2.0, 2.5, OcclusionRole::Interior);
        let player = structure_player(ground, 1.0);

        assert_eq!(
            structure_target_alpha(&structure, Some(focus(1.0, false)), Some(player)),
            XRAY_ALPHA
        );
    }

    #[test]
    fn structure_editor_focus_shows_current_and_adjacent_bands() {
        let grid = IsoGrid::default();
        let ground = grid.cell_to_world(Vec2::ZERO);
        let player = structure_player(ground, 1.0);
        let editor = focus(1.0, true);

        let current = structure(player_building(ground), 0.0, 2.0, OcclusionRole::Facade);
        let below = structure(player_building(ground), -0.2, 0.0, OcclusionRole::Facade);
        let above = structure(player_building(ground), 2.0, 2.5, OcclusionRole::Facade);

        assert_eq!(
            structure_target_alpha(&current, Some(editor), Some(player)),
            1.0
        );
        assert_eq!(
            structure_target_alpha(&below, Some(editor), Some(player)),
            DIM_ALPHA
        );
        assert_eq!(
            structure_target_alpha(&above, Some(editor), Some(player)),
            EDITOR_UPPER_ALPHA
        );
    }

    #[test]
    fn scene_occlusion_uses_local_footprint_and_height() {
        let scene = SceneDeclared {
            asset: "building.png".into(),
            position: Vec2::new(10.0, 20.0),
            scale: 1.0,
            z: 0.0,
            elevation: 1.0,
            occlusion: Some(crate::levels::SceneOcclusionDeclared {
                z_min: 1.0,
                height: 2.0,
                footprint: vec![
                    Vec2::new(-8.0, -8.0),
                    Vec2::new(8.0, -8.0),
                    Vec2::new(8.0, 8.0),
                    Vec2::new(-8.0, 8.0),
                ],
                half_extents: Vec2::ZERO,
                role: OcclusionRole::Roof,
            }),
        };
        let structure = StructureOcclusion::from_scene(&scene).expect("scene should occlude");

        assert_eq!(structure.z_min, 1.0);
        assert_eq!(structure.z_max, 3.0);
        assert_eq!(structure.role, OcclusionRole::Roof);
        let points = structure.footprint.points;
        assert!(points.contains(&Vec2::new(2.0, 12.0)));
        assert!(points.contains(&Vec2::new(18.0, 28.0)));
    }

    #[test]
    fn ranked_visibility_keeps_props_at_or_below_the_player() {
        let focus = focus(2.0, false);

        assert_eq!(ranked_target_alpha(2, Some(focus)), 1.0);
        assert_eq!(ranked_target_alpha(1, Some(focus)), 1.0);
        assert_eq!(ranked_target_alpha(3, Some(focus)), 0.0);
        assert_eq!(ranked_target_alpha(2, None), 0.0);
    }

    #[test]
    fn editor_surfaces_show_the_current_and_adjacent_floors() {
        let grid = IsoGrid::default();
        let focus = focus(2.0, true);

        assert_eq!(
            surface_target_alpha(&surface(2.0), None, Some(focus), &grid, None),
            1.0
        );
        assert_eq!(
            surface_target_alpha(&surface(1.0), None, Some(focus), &grid, None),
            DIM_ALPHA
        );
        assert_eq!(
            surface_target_alpha(&surface(3.0), None, Some(focus), &grid, None),
            EDITOR_UPPER_ALPHA
        );
        assert_eq!(
            surface_target_alpha(&surface(0.0), None, Some(focus), &grid, None),
            0.0
        );
        assert_eq!(
            surface_target_alpha(&surface(4.0), None, Some(focus), &grid, None),
            0.0
        );
    }

    #[test]
    fn upper_surfaces_stay_solid_when_they_do_not_cover_the_player() {
        let grid = IsoGrid::default();
        let focus = focus(0.0, false);
        let surface = surface(2.0);
        let footprint = SurfaceFootprint::Cells([IVec2::ZERO].into_iter().collect::<HashSet<_>>());
        let player_screen = grid.cell_to_world(Vec2::new(5.0, 0.0));

        assert_eq!(
            surface_target_alpha(
                &surface,
                Some(&footprint),
                Some(focus),
                &grid,
                Some(player_screen)
            ),
            1.0
        );
    }

    #[test]
    fn upper_surfaces_become_xray_when_they_cover_the_player() {
        let grid = IsoGrid::default();
        let focus = focus(0.0, false);
        let surface = surface(2.0);
        let footprint = SurfaceFootprint::Cells([IVec2::ZERO].into_iter().collect::<HashSet<_>>());
        let player_screen = Vec2::Y * elevation_screen_offset(2.0);

        assert_eq!(
            surface_target_alpha(
                &surface,
                Some(&footprint),
                Some(focus),
                &grid,
                Some(player_screen)
            ),
            XRAY_ALPHA
        );
    }

    #[test]
    fn elevated_floor_occludes_the_player_one_iso_step_away() {
        let grid = IsoGrid::default();
        let focus = focus(0.0, false);
        let surface = surface(1.0);
        let footprint = SurfaceFootprint::Cells([IVec2::ZERO].into_iter().collect::<HashSet<_>>());
        // A floor at elevation 1 is drawn one screen-rise above its cells. The
        // player standing one cell down-screen ends up underneath it.
        let player_screen = grid.cell_to_world(Vec2::new(1.0, -1.0));

        assert_eq!(
            surface_target_alpha(
                &surface,
                Some(&footprint),
                Some(focus),
                &grid,
                Some(player_screen)
            ),
            XRAY_ALPHA
        );
    }

    #[test]
    fn occlusion_probes_add_a_small_margin_at_surface_edges() {
        let grid = IsoGrid::default();
        let footprint = SurfaceFootprint::Cells([IVec2::ZERO].into_iter().collect::<HashSet<_>>());
        let just_outside = grid.cell_to_world(Vec2::ZERO) + Vec2::X * 36.0;

        assert!(!footprint.contains(&grid, 0.0, just_outside));
        assert!(footprint.occludes(&grid, 0.0, just_outside));
    }

    #[test]
    fn access_footprint_tracks_the_ramp_surface() {
        let grid = IsoGrid::default();
        let access = RuntimeAccess {
            from: grid.cell_to_world(Vec2::ZERO),
            to: grid.cell_to_world(Vec2::new(2.0, 0.0)),
            width: grid.step_u().length(),
            from_elevation: 0.0,
            to_elevation: 1.0,
            kind: AccessKind::Ramp,
        };
        let footprint = SurfaceFootprint::Polygon(access_screen_footprint(&grid, &access));
        let midpoint = (access.from + access.to) * 0.5 + Vec2::Y * elevation_screen_offset(0.5);

        assert!(footprint.contains(&grid, 0.5, midpoint));
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
