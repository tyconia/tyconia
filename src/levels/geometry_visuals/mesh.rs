use super::*;
use bevy::render::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

#[derive(Default)]
struct MeshBuilder {
    positions: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

impl MeshBuilder {
    fn vertex(&mut self, position: Vec2, uv: Vec2) -> u32 {
        self.vertex_shaded(position, uv, 1.0)
    }

    fn vertex_shaded(&mut self, position: Vec2, uv: Vec2, shade: f32) -> u32 {
        let index = self.positions.len() as u32;
        self.positions.push([position.x, position.y, 0.0]);
        self.uvs.push([uv.x, uv.y]);
        self.colors.push([shade, shade, shade, 1.0]);
        index
    }

    fn triangle(&mut self, a: u32, b: u32, c: u32) {
        self.indices.extend([a, b, c]);
    }

    #[allow(clippy::too_many_arguments)]
    fn quad(
        &mut self,
        a: Vec2,
        b: Vec2,
        c: Vec2,
        d: Vec2,
        uv_a: Vec2,
        uv_b: Vec2,
        uv_c: Vec2,
        uv_d: Vec2,
        shade: f32,
    ) {
        let a = self.vertex_shaded(a, uv_a, shade);
        let b = self.vertex_shaded(b, uv_b, shade);
        let c = self.vertex_shaded(c, uv_c, shade);
        let d = self.vertex_shaded(d, uv_d, shade);
        self.triangle(a, b, c);
        self.triangle(a, c, d);
    }

    fn add_top_polygon(&mut self, polygon: &[Vec2], offset: Vec2, uv: impl Fn(Vec2) -> Vec2) {
        let triangles = triangulate_polygon(polygon);
        if triangles.is_empty() {
            return;
        }

        let base = self.positions.len() as u32;
        for point in polygon {
            let world = *point + offset;
            self.vertex(world, uv(world));
        }
        for triangle in triangles {
            self.triangle(
                base + triangle[0] as u32,
                base + triangle[1] as u32,
                base + triangle[2] as u32,
            );
        }
    }

    fn build(self) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            self.positions
                .into_iter()
                .map(|position| Vec3::from_array(position))
                .collect::<Vec<_>>(),
        );
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_UV_0,
            self.uvs
                .into_iter()
                .map(|uv| Vec2::from_array(uv))
                .collect::<Vec<_>>(),
        );
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_COLOR,
            self.colors
                .into_iter()
                .map(Vec4::from_array)
                .collect::<Vec<_>>(),
        );
        mesh.insert_indices(Indices::U32(self.indices));
        mesh
    }
}

pub(super) fn build_collider_mesh(
    grid: &IsoGrid,
    collider: &ObstacleCollider,
    ground: &GroundTransform,
) -> Mesh {
    let unit = grid.step_u().length();
    let height = (collider.height * unit).max(1.0);
    let top_offset = Vec2::new(0.0, height).rotate(Vec2::from_angle(-ground.rotation));
    let rotation = Vec2::from_angle(ground.rotation);
    let to_world = |point: Vec2| rotation.rotate(point) + ground.position;
    let mut builder = MeshBuilder::default();
    let vertices = &collider.vertices;

    if vertices.len() < 3 {
        return builder.build();
    }

    let mut along = 0.0;
    for index in 0..vertices.len() {
        let start = vertices[index];
        let end = vertices[(index + 1) % vertices.len()];
        let length = (end - start).length();
        if length <= f32::EPSILON {
            continue;
        }

        let start_top = start + top_offset;
        let end_top = end + top_offset;
        let u0 = along / unit;
        let u1 = (along + length) / unit;
        let v1 = height / unit;
        let outward = -(end - start).perp() / length;
        let shade = 0.62 + 0.12 * outward.x.clamp(-1.0, 1.0);

        builder.quad(
            start,
            end,
            end_top,
            start_top,
            Vec2::new(u0, 0.0),
            Vec2::new(u1, 0.0),
            Vec2::new(u1, v1),
            Vec2::new(u0, v1),
            shade,
        );
        along += length;
    }

    let top_polygon = vertices
        .iter()
        .map(|point| *point + top_offset)
        .collect::<Vec<_>>();
    builder.add_top_polygon(&top_polygon, Vec2::ZERO, |point| {
        iso_uv(grid, to_world(point))
    });
    builder.build()
}

pub(super) fn build_floor_mesh(grid: &IsoGrid, floor: &RuntimeFloor) -> Mesh {
    let unit = grid.step_u().length();
    let mut builder = MeshBuilder::default();
    let height_offset = Vec2::new(0.0, elevation_screen_offset(floor.elevation));
    let bottom_offset = Vec2::new(0.0, -FLOOR_PLATE_THICKNESS);

    for boundary in grid.cell_boundary_loops(&floor.cells) {
        let mut along = 0.0;
        for index in 0..boundary.len() {
            let start = boundary[index] + height_offset;
            let end = boundary[(index + 1) % boundary.len()] + height_offset;
            let length = (end - start).length();
            if length <= f32::EPSILON {
                continue;
            }
            let u0 = along / unit;
            let u1 = (along + length) / unit;
            builder.quad(
                start,
                end,
                end + bottom_offset,
                start + bottom_offset,
                Vec2::new(u0, 0.0),
                Vec2::new(u1, 0.0),
                Vec2::new(u1, FLOOR_PLATE_THICKNESS / unit),
                Vec2::new(u0, FLOOR_PLATE_THICKNESS / unit),
                0.62,
            );
            along += length;
        }
    }

    for (min, max) in floor_rectangles(&floor.cells) {
        let corners = rect_corners(grid, min, max, height_offset);
        builder.add_top_polygon(&corners, Vec2::ZERO, |point| {
            iso_uv(grid, point - height_offset)
        });
    }

    builder.build()
}

fn rect_corners(grid: &IsoGrid, min: IVec2, max: IVec2, offset: Vec2) -> [Vec2; 4] {
    let center = grid.cell_to_world((min.as_vec2() + max.as_vec2()) * 0.5) + offset;
    let size = (max - min + IVec2::ONE).as_vec2();
    let half_u = grid.step_u() * size.x * 0.5;
    let half_v = grid.step_v() * size.y * 0.5;
    [
        center + half_u + half_v,
        center + half_u - half_v,
        center - half_u - half_v,
        center - half_u + half_v,
    ]
}

fn floor_rectangles(cells: &std::collections::HashSet<IVec2>) -> Vec<(IVec2, IVec2)> {
    let mut remaining = cells.clone();
    let mut rectangles = Vec::new();

    while let Some(&start) = remaining.iter().min_by_key(|cell| (cell.y, cell.x)) {
        let mut width = 1;
        while remaining.contains(&IVec2::new(start.x + width, start.y)) {
            width += 1;
        }

        let mut height = 1;
        'height: loop {
            for offset_x in 0..width {
                if !remaining.contains(&IVec2::new(start.x + offset_x, start.y + height)) {
                    break 'height;
                }
            }
            height += 1;
        }

        for offset_y in 0..height {
            for offset_x in 0..width {
                remaining.remove(&IVec2::new(start.x + offset_x, start.y + offset_y));
            }
        }

        rectangles.push((start, IVec2::new(start.x + width - 1, start.y + height - 1)));
    }

    rectangles
}

struct AccessEnds {
    low_ground: Vec2,
    high_ground: Vec2,
    low_elevation: f32,
    high_elevation: f32,
    side: Vec2,
    length: f32,
}

fn access_ends(grid: &IsoGrid, access: &RuntimeAccess) -> Option<AccessEnds> {
    let delta = access.to - access.from;
    let length = delta.length();
    if length <= f32::EPSILON {
        return None;
    }

    let side = access_side_direction(grid, access.from, access.to) * access.width * 0.5;
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

    Some(AccessEnds {
        low_ground,
        high_ground,
        low_elevation,
        high_elevation,
        side,
        length,
    })
}

pub(super) fn build_ramp_mesh(grid: &IsoGrid, access: &RuntimeAccess) -> Mesh {
    let unit = grid.step_u().length();
    let mut builder = MeshBuilder::default();
    let Some(ends) = access_ends(grid, access) else {
        return builder.build();
    };

    let start = ends.low_ground + Vec2::new(0.0, elevation_screen_offset(ends.low_elevation));
    let end = ends.high_ground + Vec2::new(0.0, elevation_screen_offset(ends.high_elevation));
    let low_end = ends.high_ground + Vec2::new(0.0, elevation_screen_offset(ends.low_elevation));
    let low_start = ends.low_ground + Vec2::new(0.0, elevation_screen_offset(ends.low_elevation));
    let rise = (ends.high_elevation - ends.low_elevation).abs() * FLOOR_SCREEN_RISE;
    let u_end = ends.length / unit;
    let v_side = access.width / unit;
    let v_rise = rise / unit;

    builder.quad(
        start + ends.side,
        end + ends.side,
        low_end + ends.side,
        low_start + ends.side,
        Vec2::new(0.0, 0.0),
        Vec2::new(u_end, 0.0),
        Vec2::new(u_end, v_rise),
        Vec2::new(0.0, v_rise),
        0.58,
    );
    builder.quad(
        end - ends.side,
        start - ends.side,
        low_start - ends.side,
        low_end - ends.side,
        Vec2::new(u_end, 0.0),
        Vec2::new(0.0, 0.0),
        Vec2::new(0.0, v_rise),
        Vec2::new(u_end, v_rise),
        0.72,
    );
    builder.quad(
        end + ends.side,
        end - ends.side,
        low_end - ends.side,
        low_end + ends.side,
        Vec2::new(0.0, 0.0),
        Vec2::new(v_side, 0.0),
        Vec2::new(v_side, v_rise),
        Vec2::new(0.0, v_rise),
        0.65,
    );

    builder.quad(
        start + ends.side,
        start - ends.side,
        end - ends.side,
        end + ends.side,
        Vec2::new(0.0, 0.0),
        Vec2::new(0.0, v_side),
        Vec2::new(u_end, v_side),
        Vec2::new(u_end, 0.0),
        1.0,
    );

    builder.build()
}

pub fn stair_step_count(rise: f32) -> usize {
    ((rise.abs() / STAIR_TARGET_RISE).ceil() as usize).clamp(MIN_STAIRS, MAX_STAIRS)
}

pub(super) fn build_stairs_mesh(grid: &IsoGrid, access: &RuntimeAccess) -> Mesh {
    let unit = grid.step_u().length();
    let mut builder = MeshBuilder::default();
    let Some(ends) = access_ends(grid, access) else {
        return builder.build();
    };

    let rise_pixels = (ends.high_elevation - ends.low_elevation).abs() * FLOOR_SCREEN_RISE;
    let steps = stair_step_count(rise_pixels);
    let run = ends.high_ground - ends.low_ground;
    let rise = ends.high_elevation - ends.low_elevation;
    let low_offset = Vec2::new(0.0, elevation_screen_offset(ends.low_elevation));
    let v_side = access.width / unit;

    for index in 0..steps {
        let t0 = index as f32 / steps as f32;
        let t1 = (index + 1) as f32 / steps as f32;
        let ground0 = ends.low_ground + run * t0;
        let ground1 = ends.low_ground + run * t1;
        let elevation0 = ends.low_elevation + rise * t0;
        let elevation1 = ends.low_elevation + rise * t1;
        let top0 = ground0 + Vec2::new(0.0, elevation_screen_offset(elevation1));
        let top1 = ground1 + Vec2::new(0.0, elevation_screen_offset(elevation1));
        let riser0 = ground0 + Vec2::new(0.0, elevation_screen_offset(elevation0));
        let bottom0 = ground0 + low_offset;
        let bottom1 = ground1 + low_offset;
        let u0 = ends.length * t0 / unit;
        let u1 = ends.length * t1 / unit;
        let height = (elevation1 - ends.low_elevation).abs() * FLOOR_SCREEN_RISE / unit;
        let riser_height = (elevation1 - elevation0).abs() * FLOOR_SCREEN_RISE / unit;

        builder.quad(
            riser0 + ends.side,
            riser0 - ends.side,
            top0 - ends.side,
            top0 + ends.side,
            Vec2::new(u0, 0.0),
            Vec2::new(u0 + v_side, 0.0),
            Vec2::new(u0 + v_side, riser_height),
            Vec2::new(u0, riser_height),
            0.78,
        );
        builder.quad(
            top0 + ends.side,
            top1 + ends.side,
            bottom1 + ends.side,
            bottom0 + ends.side,
            Vec2::new(u0, 0.0),
            Vec2::new(u1, 0.0),
            Vec2::new(u1, height),
            Vec2::new(u0, height),
            0.58,
        );
        builder.quad(
            top1 - ends.side,
            top0 - ends.side,
            bottom0 - ends.side,
            bottom1 - ends.side,
            Vec2::new(u1, 0.0),
            Vec2::new(u0, 0.0),
            Vec2::new(u0, height),
            Vec2::new(u1, height),
            0.72,
        );
    }

    let high = ends.high_ground + Vec2::new(0.0, elevation_screen_offset(ends.high_elevation));
    let low = ends.high_ground + low_offset;
    let height = (ends.high_elevation - ends.low_elevation).abs() * FLOOR_SCREEN_RISE / unit;

    builder.quad(
        high + ends.side,
        high - ends.side,
        low - ends.side,
        low + ends.side,
        Vec2::new(0.0, 0.0),
        Vec2::new(v_side, 0.0),
        Vec2::new(v_side, height),
        Vec2::new(0.0, height),
        0.65,
    );

    for index in 0..steps {
        let t0 = index as f32 / steps as f32;
        let t1 = (index + 1) as f32 / steps as f32;
        let ground0 = ends.low_ground + run * t0;
        let ground1 = ends.low_ground + run * t1;
        let elevation1 = ends.low_elevation + rise * t1;
        let top0 = ground0 + Vec2::new(0.0, elevation_screen_offset(elevation1));
        let top1 = ground1 + Vec2::new(0.0, elevation_screen_offset(elevation1));
        let u0 = ends.length * t0 / unit;
        let u1 = ends.length * t1 / unit;

        builder.quad(
            top0 + ends.side,
            top0 - ends.side,
            top1 - ends.side,
            top1 + ends.side,
            Vec2::new(u0, 0.0),
            Vec2::new(u0, v_side),
            Vec2::new(u1, v_side),
            Vec2::new(u1, 0.0),
            1.0,
        );
    }

    builder.build()
}

pub(super) fn triangulate_polygon(polygon: &[Vec2]) -> Vec<[usize; 3]> {
    if polygon.len() < 3 {
        return Vec::new();
    }

    let mut indices = (0..polygon.len()).collect::<Vec<_>>();
    if polygon_signed_area(polygon) < 0.0 {
        indices.reverse();
    }

    let mut triangles = Vec::new();
    let mut guard = 0usize;
    let guard_limit = polygon.len() * polygon.len();

    while indices.len() > 3 && guard < guard_limit {
        let mut ear = None;
        for index in 0..indices.len() {
            let previous = indices[(index + indices.len() - 1) % indices.len()];
            let current = indices[index];
            let next = indices[(index + 1) % indices.len()];
            let a = polygon[previous];
            let b = polygon[current];
            let c = polygon[next];
            if (b - a).perp_dot(c - b) <= 0.0001 {
                continue;
            }

            let mut contains = false;
            for &candidate in &indices {
                if candidate == previous || candidate == current || candidate == next {
                    continue;
                }
                if point_in_triangle(polygon[candidate], a, b, c) {
                    contains = true;
                    break;
                }
            }
            if contains {
                continue;
            }

            ear = Some((index, previous, current, next));
            break;
        }

        let Some((index, previous, current, next)) = ear else {
            break;
        };
        triangles.push([previous, current, next]);
        indices.remove(index);
        guard += 1;
    }

    if indices.len() == 3 {
        triangles.push([indices[0], indices[1], indices[2]]);
    }

    triangles
}

fn point_in_triangle(point: Vec2, a: Vec2, b: Vec2, c: Vec2) -> bool {
    let ab = (b - a).perp_dot(point - a);
    let bc = (c - b).perp_dot(point - b);
    let ca = (a - c).perp_dot(point - c);
    (ab >= 0.0 && bc >= 0.0 && ca >= 0.0) || (ab <= 0.0 && bc <= 0.0 && ca <= 0.0)
}
