//! Rendering for NPCs. Each NPC is drawn as a simple isometric trapezoid with a
//! color and silhouette that differ by kind.

use super::*;
use crate::levels::{elevation_screen_offset, level_z, NpcKind, RankedVisual, ACTOR_Z_LAYER};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

#[derive(Resource)]
pub(super) struct NpcVisualAssets {
    civilian_mesh: Handle<Mesh>,
    guard_mesh: Handle<Mesh>,
    civilian_color: Color,
    guard_color: Color,
}

pub(super) fn setup_npc_visual_assets(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let civilian_mesh = meshes.add(build_trapezoid_mesh(19.0, 12.0, 24.0));
    let guard_mesh = meshes.add(build_trapezoid_mesh(23.0, 14.0, 28.0));

    commands.insert_resource(NpcVisualAssets {
        civilian_mesh,
        guard_mesh,
        civilian_color: Color::srgb(0.35, 0.78, 0.72),
        guard_color: Color::srgb(0.88, 0.34, 0.27),
    });
}

fn build_trapezoid_mesh(bottom_width: f32, top_width: f32, height: f32) -> Mesh {
    let half_bottom = bottom_width * 0.5;
    let half_top = top_width * 0.5;
    let positions = vec![
        Vec3::new(-half_bottom, 0.0, 0.0),
        Vec3::new(half_bottom, 0.0, 0.0),
        Vec3::new(half_top, height, 0.0),
        Vec3::new(-half_top, height, 0.0),
    ];
    let uvs = vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(1.0, 1.0),
        Vec2::new(0.0, 1.0),
    ];
    let colors = vec![Vec4::ONE; 4];

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(vec![0, 1, 2, 0, 2, 3]));
    mesh
}

pub(super) fn attach_npc_visuals(
    mut commands: Commands,
    assets: Res<NpcVisualAssets>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    visuals: Query<(Entity, &NpcVisual), Added<NpcVisual>>,
) {
    for (entity, visual) in &visuals {
        let (mesh, color) = match visual.kind {
            NpcKind::Civilian => (&assets.civilian_mesh, assets.civilian_color),
            NpcKind::Guard => (&assets.guard_mesh, assets.guard_color),
        };
        let material = materials.add(ColorMaterial { color, ..default() });

        commands
            .entity(entity)
            .insert((Mesh2d(mesh.clone()), MeshMaterial2d(material)));
    }
}

pub(super) fn sync_npc_visuals(
    mut npcs: Query<(&NpcHeight, &Children, &mut Transform), (With<Npc>, Without<NpcVisual>)>,
    mut visuals: Query<(&mut Transform, &mut RankedVisual), With<NpcVisual>>,
) {
    for (height, children, mut transform) in &mut npcs {
        transform.translation.z = level_z(height.rank, ACTOR_Z_LAYER, transform.translation.y);

        for child in children.iter() {
            if let Ok((mut visual_transform, mut ranked)) = visuals.get_mut(*child) {
                visual_transform.translation.y = elevation_screen_offset(height.elevation);
                ranked.rank = height.rank;
            }
        }
    }
}

pub(super) fn npc_debug_gizmos(
    mut gizmos: Gizmos,
    npcs: Query<(
        &GlobalTransform,
        &Npc,
        &NpcFacing,
        &NpcVision,
        &NpcMemory,
        &NpcNavigation,
        &NpcMode,
    )>,
) {
    for (transform, npc, facing, vision, memory, navigation, mode) in &npcs {
        let origin = transform.translation().xy();
        let facing = facing.0.normalize_or_zero();
        let color = mode_color(*mode, npc.kind);

        if facing != Vec2::ZERO {
            let left = Vec2::from_angle(-vision.half_angle).rotate(facing);
            let right = Vec2::from_angle(vision.half_angle).rotate(facing);
            gizmos.line_2d(origin, origin + left * vision.range, color);
            gizmos.line_2d(origin, origin + right * vision.range, color);
        }

        if let Some(last_known) = memory.last_known {
            let cross = Vec2::splat(6.0);
            gizmos.line_2d(last_known - cross, last_known + cross, color);
            gizmos.line_2d(
                last_known + Vec2::new(-cross.x, cross.y),
                last_known + Vec2::new(cross.x, -cross.y),
                color,
            );
            gizmos.line_2d(origin, last_known, color.with_alpha(0.35));
        }

        if let Some(destination) = navigation.destination {
            gizmos.line_2d(origin, destination, color.with_alpha(0.5));
        }
    }
}

fn mode_color(mode: NpcMode, kind: NpcKind) -> Color {
    match mode {
        NpcMode::Idle => Color::srgba(0.72, 0.72, 0.72, 0.55),
        NpcMode::Patrol => Color::srgba(0.35, 0.78, 0.72, 0.6),
        NpcMode::Investigate => Color::srgba(0.95, 0.75, 0.25, 0.75),
        NpcMode::Chase => match kind {
            NpcKind::Civilian => Color::srgba(0.95, 0.55, 0.3, 0.85),
            NpcKind::Guard => Color::srgba(0.95, 0.2, 0.18, 0.9),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::render::mesh::VertexAttributeValues;

    #[test]
    fn trapezoid_mesh_has_four_vertices_and_two_triangles() {
        let mesh = build_trapezoid_mesh(20.0, 12.0, 24.0);
        let Some(VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("trapezoid is missing positions");
        };

        assert_eq!(positions.len(), 4);
        assert_eq!(mesh.indices().map(|indices| indices.len()), Some(6));

        let Some(VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            unreachable!();
        };
        assert!(positions.iter().any(|position| position[1] > 20.0));
    }
}
