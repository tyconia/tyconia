use super::*;

#[derive(Resource, Debug, Clone)]
pub struct IsoGrid {
    pub origin: Vec2,
    pub cell_size: Vec2,
    pub extent: i32,
    pub major_every: i32,
}

impl Default for IsoGrid {
    fn default() -> Self {
        Self {
            origin: Vec2::ZERO,
            cell_size: Vec2::new(64.0, 32.0),
            extent: 32,
            major_every: 4,
        }
    }
}

impl IsoGrid {
    pub fn half_axis(&self) -> Vec2 {
        self.cell_size * 0.25
    }

    pub fn step_u(&self) -> Vec2 {
        self.half_axis() * 2.0
    }

    pub fn step_v(&self) -> Vec2 {
        Vec2::new(self.step_u().x, -self.step_u().y)
    }

    pub fn cell_to_world(&self, cell: Vec2) -> Vec2 {
        self.origin + self.step_u() * cell.x + self.step_v() * cell.y
    }

    pub fn world_to_cell(&self, world: Vec2) -> Vec2 {
        let u = self.step_u();
        let v = self.step_v();
        let delta = world - self.origin;
        let determinant = u.x * v.y - v.x * u.y;

        if determinant.abs() <= f32::EPSILON {
            return Vec2::ZERO;
        }

        Vec2::new(
            (delta.x * v.y - v.x * delta.y) / determinant,
            (u.x * delta.y - delta.x * u.y) / determinant,
        )
    }

    pub fn snap_cell(&self, world: Vec2) -> IVec2 {
        self.world_to_cell(world).round().as_ivec2()
    }

    pub fn cell_half_extent(&self) -> f32 {
        self.half_axis().length()
    }

    pub fn footprint_half_extents(&self, cells: IVec2) -> Vec2 {
        Vec2::new(
            self.step_u().length() * cells.x as f32 * 0.5,
            self.step_v().length() * cells.y as f32 * 0.5,
        )
    }

    pub fn footprint_center(&self, min: IVec2, max: IVec2) -> Vec2 {
        self.cell_to_world((min.as_vec2() + max.as_vec2()) * 0.5)
    }

    pub fn collider_from_cells(&self, min: IVec2, max: IVec2) -> ColliderDeclared {
        let (min, max) = (min.min(max), min.max(max));
        let cells = max - min + IVec2::ONE;

        ColliderDeclared {
            position: self.footprint_center(min, max),
            half_extents: self.footprint_half_extents(cells),
            rotation_degrees: 0.0,
            screen_space: false,
            vertices: Vec::new(),
            z_min: 0.0,
            height: crate::levels::default_collider_height(),
            role: OcclusionRole::Facade,
        }
    }

    pub fn collider_from_cell_set(&self, cells: &HashSet<IVec2>) -> Option<ColliderDeclared> {
        let min = cells.iter().copied().reduce(IVec2::min)?;
        let max = cells.iter().copied().reduce(IVec2::max)?;
        let width = max.x - min.x + 1;
        let height = max.y - min.y + 1;

        if cells.len() == (width * height) as usize {
            return Some(self.collider_from_cells(min, max));
        }

        let mut loops = self.cell_boundary_loops(cells);
        if loops.len() != 1 {
            return None;
        }

        let mut points = loops.remove(0);
        simplify_polygon(&mut points);
        if points.len() < 3 {
            return None;
        }
        if polygon_signed_area(&points) < 0.0 {
            points.reverse();
        }

        let position = points.iter().copied().sum::<Vec2>() / points.len() as f32;
        let min = points
            .iter()
            .copied()
            .fold(Vec2::splat(f32::INFINITY), Vec2::min);
        let max = points
            .iter()
            .copied()
            .fold(Vec2::splat(f32::NEG_INFINITY), Vec2::max);

        Some(ColliderDeclared {
            position,
            half_extents: (max - min) * 0.5,
            rotation_degrees: 0.0,
            screen_space: false,
            vertices: points.iter().map(|point| *point - position).collect(),
            z_min: 0.0,
            height: crate::levels::default_collider_height(),
            role: OcclusionRole::Facade,
        })
    }

    pub fn cell_boundary_loops(&self, cells: &HashSet<IVec2>) -> Vec<Vec<Vec2>> {
        let mut outgoing: HashMap<IVec2, Vec<IVec2>> = HashMap::new();

        for &cell in cells {
            let corners = [
                IVec2::new(cell.x * 2 - 1, cell.y * 2 - 1),
                IVec2::new(cell.x * 2 + 1, cell.y * 2 - 1),
                IVec2::new(cell.x * 2 + 1, cell.y * 2 + 1),
                IVec2::new(cell.x * 2 - 1, cell.y * 2 + 1),
            ];

            for (direction, edge) in [
                (IVec2::X, (1, 2)),
                (IVec2::Y, (2, 3)),
                (-IVec2::X, (3, 0)),
                (-IVec2::Y, (0, 1)),
            ] {
                if !cells.contains(&(cell + direction)) {
                    outgoing
                        .entry(corners[edge.0])
                        .or_default()
                        .push(corners[edge.1]);
                }
            }
        }

        let mut loops = Vec::new();

        while let Some((&start, _)) = outgoing.iter().next() {
            let mut loop_keys = Vec::new();
            let mut previous = None;
            let mut current = start;

            loop {
                let Some(candidates) = outgoing.get_mut(&current) else {
                    break;
                };
                if candidates.is_empty() {
                    break;
                }

                loop_keys.push(current);
                let next = if candidates.len() == 1 {
                    candidates.remove(0)
                } else {
                    let index = match previous {
                        Some(previous) => choose_next_boundary_edge(previous, current, candidates),
                        None => 0,
                    };
                    candidates.remove(index)
                };

                previous = Some(current);
                current = next;
                if current == start {
                    break;
                }
                if loop_keys.len() > cells.len() * 8 + 8 {
                    break;
                }
            }

            if current == start && loop_keys.len() >= 3 {
                loops.push(
                    loop_keys
                        .into_iter()
                        .map(|key| self.cell_to_world(key.as_vec2() * 0.5))
                        .collect(),
                );
            }

            outgoing.retain(|_, edges| !edges.is_empty());
        }

        loops
    }

    pub fn rect_cells_from_collider(&self, collider: &ColliderDeclared) -> Option<(IVec2, IVec2)> {
        if collider.screen_space
            || collider.rotation_degrees.abs() > 0.01
            || !collider.vertices.is_empty()
        {
            return None;
        }

        let step_length = self.step_u().length();
        if step_length <= f32::EPSILON {
            return None;
        }

        let cells = IVec2::new(
            (collider.half_extents.x * 2.0 / step_length).round() as i32,
            (collider.half_extents.y * 2.0 / step_length).round() as i32,
        );
        if cells.x < 1 || cells.y < 1 {
            return None;
        }

        let expected_half_extents = self.footprint_half_extents(cells);
        if (expected_half_extents - collider.half_extents).length_squared() > 0.001 {
            return None;
        }

        let center = self.world_to_cell(collider.position);
        let min = IVec2::new(
            (center.x - (cells.x as f32 - 1.0) * 0.5).round() as i32,
            (center.y - (cells.y as f32 - 1.0) * 0.5).round() as i32,
        );
        let max = min + cells - IVec2::ONE;
        let rebuilt = self.collider_from_cells(min, max);

        if (rebuilt.position - collider.position).length_squared() > 0.001 {
            return None;
        }

        Some((min, max))
    }

    pub fn cells_from_collider(&self, collider: &ColliderDeclared) -> Option<Vec<IVec2>> {
        if collider.screen_space || collider.rotation_degrees.abs() > 0.01 {
            return None;
        }

        if !collider.vertices.is_empty() {
            return self.cells_from_polygon(collider);
        }

        let (min, max) = self.rect_cells_from_collider(collider)?;
        Some(
            (min.y..=max.y)
                .flat_map(|y| (min.x..=max.x).map(move |x| IVec2::new(x, y)))
                .collect(),
        )
    }

    fn cells_from_polygon(&self, collider: &ColliderDeclared) -> Option<Vec<IVec2>> {
        let points: Vec<Vec2> = collider
            .vertices
            .iter()
            .map(|vertex| *vertex + collider.position)
            .collect();
        if points.len() < 3 {
            return None;
        }

        let mut cell_min = Vec2::splat(f32::INFINITY);
        let mut cell_max = Vec2::splat(f32::NEG_INFINITY);
        for point in &points {
            let doubled_cell = self.world_to_cell(*point) * 2.0;
            if (doubled_cell - doubled_cell.round()).length_squared() > 0.01 {
                return None;
            }
            let cell = self.world_to_cell(*point);
            cell_min = cell_min.min(cell);
            cell_max = cell_max.max(cell);
        }

        let min = IVec2::new(cell_min.x.floor() as i32, cell_min.y.floor() as i32);
        let max = IVec2::new(cell_max.x.ceil() as i32, cell_max.y.ceil() as i32);

        let mut cells = Vec::new();
        for y in min.y..=max.y {
            for x in min.x..=max.x {
                let center = self.cell_to_world(Vec2::new(x as f32, y as f32));
                if point_in_polygon(center, &points) {
                    cells.push(IVec2::new(x, y));
                }
            }
        }
        if cells.is_empty() {
            return None;
        }

        let cell_area = {
            let u = self.step_u();
            let v = self.step_v();
            (u.x * v.y - v.x * u.y).abs()
        };
        let polygon_area = polygon_signed_area(&points).abs();
        if (polygon_area - cells.len() as f32 * cell_area).abs() > cell_area * 0.01 {
            return None;
        }

        Some(cells)
    }
}

pub(super) fn choose_next_boundary_edge(
    previous: IVec2,
    current: IVec2,
    candidates: &[IVec2],
) -> usize {
    let incoming = (current - previous).as_vec2();
    let mut best = 0;
    let mut best_angle = f32::INFINITY;

    for (index, candidate) in candidates.iter().enumerate() {
        let outgoing = (*candidate - current).as_vec2();
        let angle = incoming.perp_dot(outgoing).atan2(incoming.dot(outgoing));
        if angle < best_angle {
            best_angle = angle;
            best = index;
        }
    }

    best
}

pub(super) fn simplify_polygon(points: &mut Vec<Vec2>) {
    while points.len() > 3 {
        let mut removed = false;

        for index in 0..points.len() {
            let previous = points[(index + points.len() - 1) % points.len()];
            let current = points[index];
            let next = points[(index + 1) % points.len()];
            let cross = (current - previous).perp_dot(next - current);
            let continues = (current - previous).dot(next - current) > 0.0;

            if cross.abs() <= 0.001 && continues {
                points.remove(index);
                removed = true;
                break;
            }
        }

        if !removed {
            break;
        }
    }
}

pub(super) fn cell_touches_set(cell: IVec2, cells: &HashSet<IVec2>) -> bool {
    cells.contains(&cell)
        || [IVec2::X, -IVec2::X, IVec2::Y, -IVec2::Y]
            .into_iter()
            .any(|direction| cells.contains(&(cell + direction)))
}

pub(super) fn merge_new_collider(
    grid: &IsoGrid,
    colliders: &mut Vec<ColliderDeclared>,
    collider: ColliderDeclared,
) -> ColliderDeclared {
    let Some(new_cells) = grid.cells_from_collider(&collider) else {
        colliders.push(collider.clone());
        return collider;
    };

    let mut merged_cells: HashSet<IVec2> = new_cells.into_iter().collect();
    let mut consumed = Vec::new();

    loop {
        let mut found = None;

        for (index, existing) in colliders.iter().enumerate() {
            if consumed.contains(&index) || !existing.same_vertical_range(&collider) {
                continue;
            }

            let Some(existing_cells) = grid.cells_from_collider(existing) else {
                continue;
            };

            if existing_cells
                .iter()
                .any(|cell| cell_touches_set(*cell, &merged_cells))
            {
                merged_cells.extend(existing_cells);
                found = Some(index);
                break;
            }
        }

        let Some(index) = found else {
            break;
        };
        consumed.push(index);
    }

    if consumed.is_empty() {
        colliders.push(collider.clone());
        return collider;
    }

    let mut merged = if let Some(merged_collider) = grid.collider_from_cell_set(&merged_cells) {
        vec![merged_collider]
    } else {
        colliders_from_cell_set(grid, &merged_cells)
    };
    if merged.is_empty() {
        colliders.push(collider.clone());
        return collider;
    }
    for fragment in &mut merged {
        fragment.copy_vertical_from(&collider);
    }
    let result = merged[0].clone();

    consumed.sort_unstable_by(|a, b| b.cmp(a));
    for index in consumed {
        colliders.remove(index);
    }

    colliders.extend(merged);
    result
}

pub(super) fn remove_cell_from_colliders(
    grid: &IsoGrid,
    colliders: &mut Vec<ColliderDeclared>,
    cell: IVec2,
) -> Option<usize> {
    let index = colliders.iter().rposition(|collider| {
        grid.cells_from_collider(collider)
            .is_some_and(|cells| cells.contains(&cell))
    })?;
    let mut cells: HashSet<IVec2> = grid
        .cells_from_collider(&colliders[index])
        .expect("matched collider should convert to cells")
        .into_iter()
        .collect();
    let removed = colliders.remove(index);

    cells.remove(&cell);
    let mut fragments = colliders_from_cell_set(grid, &cells);
    for fragment in &mut fragments {
        fragment.copy_vertical_from(&removed);
    }
    let fragment_count = fragments.len();
    colliders.extend(fragments);
    Some(fragment_count)
}

pub(super) fn colliders_from_cell_set(
    grid: &IsoGrid,
    cells: &HashSet<IVec2>,
) -> Vec<ColliderDeclared> {
    let mut colliders = Vec::new();

    for component in connected_cell_components(cells) {
        if let Some(collider) = grid.collider_from_cell_set(&component) {
            colliders.push(collider);
        } else {
            colliders.extend(decompose_cell_set_into_rects(grid, component));
        }
    }

    colliders
}

pub(super) fn connected_cell_components(cells: &HashSet<IVec2>) -> Vec<HashSet<IVec2>> {
    let mut remaining = cells.clone();
    let mut components = Vec::new();

    while let Some(&start) = remaining.iter().next() {
        remaining.remove(&start);
        let mut component = HashSet::from([start]);
        let mut stack = vec![start];

        while let Some(cell) = stack.pop() {
            for direction in [IVec2::X, -IVec2::X, IVec2::Y, -IVec2::Y] {
                let neighbor = cell + direction;
                if remaining.remove(&neighbor) {
                    component.insert(neighbor);
                    stack.push(neighbor);
                }
            }
        }

        components.push(component);
    }

    components
}

pub(super) fn decompose_cell_set_into_rects(
    grid: &IsoGrid,
    mut cells: HashSet<IVec2>,
) -> Vec<ColliderDeclared> {
    let mut colliders = Vec::new();

    while let Some(&start) = cells.iter().min_by_key(|cell| (cell.y, cell.x)) {
        let mut width = 1;
        while cells.contains(&IVec2::new(start.x + width, start.y)) {
            width += 1;
        }

        let mut height = 1;
        'height: loop {
            for offset_x in 0..width {
                if !cells.contains(&IVec2::new(start.x + offset_x, start.y + height)) {
                    break 'height;
                }
            }
            height += 1;
        }

        for offset_y in 0..height {
            for offset_x in 0..width {
                cells.remove(&IVec2::new(start.x + offset_x, start.y + offset_y));
            }
        }

        colliders.push(
            grid.collider_from_cells(start, IVec2::new(start.x + width - 1, start.y + height - 1)),
        );
    }

    colliders
}

pub(super) fn refresh_level_collider_entities(
    commands: &mut Commands,
    instance_entity: Entity,
    old_colliders: impl IntoIterator<Item = Entity>,
    colliders: &[ColliderDeclared],
) {
    for entity in old_colliders {
        commands.entity(entity).despawn_recursive();
    }

    commands.entity(instance_entity).with_children(|parent| {
        for collider in colliders {
            let ground = GroundTransform::from_declared(collider);
            let rank = rank_from_elevation(collider.z_min);
            parent.spawn((
                LevelCollider,
                ObstacleCollider::from_declared(collider),
                ground,
                Transform::from_xyz(
                    collider.position.x,
                    collider.position.y + elevation_screen_offset(collider.z_min),
                    level_z(rank, PROP_Z_LAYER, collider.position.y),
                )
                .with_rotation(Quat::from_rotation_z(ground.rotation)),
            ));
        }
    });
}
