use super::{AccessDeclared, AccessKind, FloorDeclared, IsoGrid, LevelIsometric};
use bevy::prelude::*;
use std::collections::HashSet;

pub const FLOOR_SCREEN_RISE: f32 = 35.777088;
pub const LEVEL_Z_BAND: f32 = 1000.0;
pub const LEVEL_DEPTH_PER_Y: f32 = 0.0004;
pub const FLOOR_Z_LAYER: f32 = 100.0;
pub const FLOOR_EDGE_Z_LAYER: f32 = 200.0;
pub const ACCESS_Z_LAYER: f32 = 300.0;
pub const PROP_Z_LAYER: f32 = 400.0;
pub const ACTOR_Z_LAYER: f32 = 500.0;
pub const OVERHEAD_Z_LAYER: f32 = 900.0;
pub const EDITOR_Z_LAYER: f32 = 950.0;
pub const PLAYER_BODY_HEIGHT: f32 = 0.9;
pub const STEP_UP: f32 = 0.2;
pub const STEP_DOWN: f32 = 0.3;
pub const GROUND_EPSILON: f32 = 0.01;

#[derive(Component, Debug, Clone, Default)]
pub struct LevelVertical {
    pub floors: Vec<RuntimeFloor>,
    pub accesses: Vec<RuntimeAccess>,
}

#[derive(Debug, Clone)]
pub struct RuntimeFloor {
    pub elevation: f32,
    pub rank: i16,
    pub cells: HashSet<IVec2>,
}

#[derive(Debug, Clone)]
pub struct RuntimeAccess {
    pub from: Vec2,
    pub to: Vec2,
    pub width: f32,
    pub from_elevation: f32,
    pub to_elevation: f32,
    pub kind: AccessKind,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeightSample {
    pub elevation: f32,
    pub rank: i16,
}

impl LevelVertical {
    pub fn from_level(level: &LevelIsometric) -> Self {
        let floors = level
            .floors
            .iter()
            .map(RuntimeFloor::from)
            .collect::<Vec<_>>();
        let accesses = level
            .accesses
            .iter()
            .map(RuntimeAccess::from)
            .collect::<Vec<_>>();
        Self { floors, accesses }
    }

    pub fn from_parts(floors: Vec<FloorDeclared>, accesses: Vec<AccessDeclared>) -> Self {
        Self {
            floors: floors.iter().map(RuntimeFloor::from).collect(),
            accesses: accesses.iter().map(RuntimeAccess::from).collect(),
        }
    }

    pub fn sample(
        &self,
        grid: &IsoGrid,
        position: Vec2,
        current_elevation: f32,
    ) -> Option<HeightSample> {
        if let Some(sample) = self.sample_band(grid, position, current_elevation) {
            return Some(sample);
        }

        if current_elevation.abs() <= GROUND_EPSILON {
            return Some(HeightSample {
                elevation: 0.0,
                rank: 0,
            });
        }

        None
    }

    /// Find a floor or access at `position` that is reachable from
    /// `current_elevation` without assuming the implicit ground floor. This is
    /// used to distinguish "there is nothing here above ground" from "another
    /// floor sits above this position".
    pub fn sample_band(
        &self,
        grid: &IsoGrid,
        position: Vec2,
        current_elevation: f32,
    ) -> Option<HeightSample> {
        if self.floors.is_empty() && self.accesses.is_empty() {
            return Some(HeightSample {
                elevation: current_elevation,
                rank: rank_from_elevation(current_elevation),
            });
        }

        let mut best_access: Option<HeightSample> = None;
        let mut best_access_distance = f32::INFINITY;
        for access in &self.accesses {
            let Some(elevation) = access.height_at(grid, position) else {
                continue;
            };
            if elevation > current_elevation + STEP_UP || elevation < current_elevation - STEP_DOWN
            {
                continue;
            }
            let distance = (elevation - current_elevation).abs();
            if distance < best_access_distance {
                best_access_distance = distance;
                best_access = Some(HeightSample {
                    elevation,
                    rank: rank_from_elevation(elevation),
                });
            }
        }
        if best_access.is_some() {
            return best_access;
        }

        let cell = grid.snap_cell(position);
        let mut best_floor: Option<HeightSample> = None;
        let mut best_floor_distance = f32::INFINITY;
        for floor in &self.floors {
            if !floor.cells.contains(&cell) {
                continue;
            }
            if floor.elevation > current_elevation + STEP_UP
                || floor.elevation < current_elevation - STEP_DOWN
            {
                continue;
            }
            let distance = (floor.elevation - current_elevation).abs();
            if distance < best_floor_distance {
                best_floor_distance = distance;
                best_floor = Some(HeightSample {
                    elevation: floor.elevation,
                    rank: floor.rank,
                });
            }
        }

        best_floor
    }

    /// Find the highest walkable height at `position`, regardless of the
    /// elevation band the caller currently occupies. This is used when routing
    /// toward a destination that lives on another floor.
    pub fn sample_any(&self, grid: &IsoGrid, position: Vec2) -> Option<HeightSample> {
        if self.floors.is_empty() && self.accesses.is_empty() {
            return Some(HeightSample {
                elevation: 0.0,
                rank: 0,
            });
        }

        let cell = grid.snap_cell(position);
        let mut best: Option<HeightSample> = None;

        for floor in &self.floors {
            if !floor.cells.contains(&cell) {
                continue;
            }
            if best
                .map(|sample| floor.elevation > sample.elevation)
                .unwrap_or(true)
            {
                best = Some(HeightSample {
                    elevation: floor.elevation,
                    rank: floor.rank,
                });
            }
        }

        for access in &self.accesses {
            let Some(elevation) = access.height_at(grid, position) else {
                continue;
            };
            if best
                .map(|sample| elevation > sample.elevation)
                .unwrap_or(true)
            {
                best = Some(HeightSample {
                    elevation,
                    rank: rank_from_elevation(elevation),
                });
            }
        }

        best.or(Some(HeightSample {
            elevation: 0.0,
            rank: 0,
        }))
    }
}

impl From<&FloorDeclared> for RuntimeFloor {
    fn from(floor: &FloorDeclared) -> Self {
        Self {
            elevation: floor.elevation,
            rank: rank_from_elevation(floor.elevation),
            cells: floor.cells.iter().copied().collect(),
        }
    }
}

impl From<&AccessDeclared> for RuntimeAccess {
    fn from(access: &AccessDeclared) -> Self {
        Self {
            from: access.from,
            to: access.to,
            width: access.width.max(1.0),
            from_elevation: access.from_elevation,
            to_elevation: access.to_elevation,
            kind: access.kind,
        }
    }
}

pub fn access_side_direction(grid: &IsoGrid, from: Vec2, to: Vec2) -> Vec2 {
    let delta = to - from;
    if delta.length_squared() <= f32::EPSILON {
        return Vec2::Y;
    }

    let direction = delta.normalize();
    let perpendicular = Vec2::new(-direction.y, direction.x);
    let u = grid.step_u().normalize_or_zero();
    let v = grid.step_v().normalize_or_zero();
    let u_score = u.dot(perpendicular).abs();
    let v_score = v.dot(perpendicular).abs();
    let mut side = if u_score >= v_score { u } else { v };
    if side.dot(perpendicular) < 0.0 {
        side = -side;
    }
    side
}

impl RuntimeAccess {
    pub fn height_at(&self, grid: &IsoGrid, position: Vec2) -> Option<f32> {
        let delta = self.to - self.from;
        let length_squared = delta.length_squared();
        if length_squared <= f32::EPSILON {
            return None;
        }

        let length = length_squared.sqrt();
        let direction = delta / length;
        let side = access_side_direction(grid, self.from, self.to);
        let determinant = direction.x * side.y - direction.y * side.x;
        if determinant.abs() <= f32::EPSILON {
            return None;
        }

        let relative = position - self.from;
        let along = (relative.x * side.y - relative.y * side.x) / determinant;
        let across = (direction.x * relative.y - direction.y * relative.x) / determinant;
        if !(0.0..=length).contains(&along) || across.abs() > self.width * 0.5 {
            return None;
        }

        let t = along / length;
        Some(self.from_elevation + (self.to_elevation - self.from_elevation) * t)
    }

    pub fn visual_rank(&self) -> i16 {
        rank_from_elevation(self.from_elevation.min(self.to_elevation))
    }
}

pub fn elevation_screen_offset(elevation: f32) -> f32 {
    elevation * FLOOR_SCREEN_RISE
}

pub fn rank_from_elevation(elevation: f32) -> i16 {
    elevation.round() as i16
}

pub fn level_z(rank: i16, layer: f32, ground_y: f32) -> f32 {
    rank as f32 * LEVEL_Z_BAND + layer - ground_y * LEVEL_DEPTH_PER_Y
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid() -> IsoGrid {
        IsoGrid::default()
    }

    fn floor(elevation: f32, rank: i16, cells: &[IVec2]) -> FloorDeclared {
        FloorDeclared {
            elevation,
            rank,
            cells: cells.to_vec(),
        }
    }

    fn ramp(from: Vec2, to: Vec2, from_elevation: f32, to_elevation: f32) -> AccessDeclared {
        AccessDeclared {
            from,
            to,
            width: 64.0,
            from_elevation,
            to_elevation,
            kind: AccessKind::Ramp,
        }
    }

    #[test]
    fn floor_rise_matches_one_iso_grid_edge() {
        let grid = grid();
        assert!((FLOOR_SCREEN_RISE - grid.step_u().length()).abs() < 0.001);
    }

    #[test]
    fn rank_bands_separate_levels_and_layers() {
        assert!(level_z(1, FLOOR_Z_LAYER, 0.0) > level_z(0, ACTOR_Z_LAYER, 0.0));
        assert!(level_z(1, ACTOR_Z_LAYER, 0.0) > level_z(1, ACCESS_Z_LAYER, 0.0));
        assert!(level_z(1, ACCESS_Z_LAYER, 0.0) > level_z(1, FLOOR_Z_LAYER, 0.0));
        assert!(level_z(1, FLOOR_Z_LAYER, 0.0) > level_z(0, OVERHEAD_Z_LAYER, 0.0));
    }

    #[test]
    fn ramp_height_interpolates_between_floors() {
        let grid = grid();
        let start = grid.cell_to_world(Vec2::new(0.0, 0.0));
        let end = grid.cell_to_world(Vec2::new(1.0, 0.0));
        let vertical = LevelVertical::from_parts(
            vec![
                floor(0.0, 0, &[IVec2::new(0, 0)]),
                floor(1.0, 1, &[IVec2::new(1, 0)]),
            ],
            vec![ramp(start, end, 0.0, 1.0)],
        );

        let midpoint = (start + end) * 0.5;
        let sample = vertical
            .sample(&grid, midpoint, 0.5)
            .expect("ramp midpoint should be walkable");

        assert!((sample.elevation - 0.5).abs() < 0.01);
        assert_eq!(sample.rank, 1);
    }

    #[test]
    fn access_visual_rank_uses_its_lower_band() {
        let grid = grid();
        let start = grid.cell_to_world(Vec2::ZERO);
        let end = grid.cell_to_world(Vec2::new(1.0, 0.0));
        let access = RuntimeAccess::from(&ramp(start, end, 0.2, 2.4));

        assert_eq!(access.visual_rank(), 0);
        assert_eq!(access.visual_rank(), rank_from_elevation(0.2));
    }

    #[test]
    fn player_rank_on_an_access_follows_sampled_elevation() {
        let grid = grid();
        let start = grid.cell_to_world(Vec2::ZERO);
        let end = grid.cell_to_world(Vec2::new(1.0, 0.0));
        let vertical = LevelVertical::from_parts(Vec::new(), vec![ramp(start, end, 0.0, 1.0)]);

        let low = vertical
            .sample(&grid, start + (end - start) * 0.2, 0.0)
            .expect("low ramp point should be walkable");
        let high = vertical
            .sample(&grid, start + (end - start) * 0.75, 0.6)
            .expect("high ramp point should be walkable");

        assert_eq!(low.rank, 0);
        assert_eq!(high.rank, 1);
    }

    #[test]
    fn runtime_floor_rank_is_derived_from_elevation() {
        let runtime_floor = RuntimeFloor::from(&floor(1.2, 0, &[IVec2::ZERO]));

        assert_eq!(runtime_floor.rank, 1);
    }

    #[test]
    fn access_width_follows_the_iso_grid_axis() {
        let grid = grid();
        let start = grid.cell_to_world(Vec2::ZERO);
        let end = grid.cell_to_world(Vec2::new(1.0, 0.0));
        let mut access = RuntimeAccess::from(&ramp(start, end, 0.0, 1.0));
        access.width = grid.step_u().length();
        let width = access.width;
        let side = access_side_direction(&grid, start, end);
        let grid_axis = grid.step_v().normalize();

        assert!((side.dot(grid_axis).abs() - 1.0).abs() < 0.001);

        let midpoint = (start + end) * 0.5;
        assert!(access
            .height_at(&grid, midpoint + side * width * 0.49)
            .is_some());
        assert!(access
            .height_at(&grid, midpoint + side * width * 0.51)
            .is_none());
    }

    #[test]
    fn direct_step_to_a_higher_floor_is_blocked() {
        let grid = grid();
        let vertical = LevelVertical::from_parts(vec![floor(1.0, 1, &[IVec2::ZERO])], Vec::new());

        let center = grid.cell_to_world(Vec2::ZERO);
        assert!(vertical.sample(&grid, center, 0.0).unwrap().elevation == 0.0);
        assert!(vertical.sample(&grid, center, 1.0).unwrap().elevation == 1.0);
        let outside = grid.cell_to_world(Vec2::new(1.0, 0.0));
        assert!(vertical.sample(&grid, outside, 1.0).is_none());
    }

    #[test]
    fn overlapping_spiral_segments_use_the_nearest_elevation() {
        let grid = grid();
        let start = grid.cell_to_world(Vec2::new(0.0, 0.0));
        let end = grid.cell_to_world(Vec2::new(4.0, 0.0));
        let vertical = LevelVertical::from_parts(
            Vec::new(),
            vec![ramp(start, end, 0.0, 1.0), ramp(start, end, 1.0, 2.0)],
        );

        let midpoint = (start + end) * 0.5;
        let lower = vertical
            .sample(&grid, midpoint, 0.5)
            .expect("lower turn should be walkable");
        let upper = vertical
            .sample(&grid, midpoint, 1.5)
            .expect("upper turn should be walkable");

        assert!((lower.elevation - 0.5).abs() < 0.01);
        assert!((upper.elevation - 1.5).abs() < 0.01);
    }

    #[test]
    fn spiral_chain_reaches_the_upper_floor() {
        let grid = grid();
        let center = grid.cell_to_world(Vec2::new(2.0, 2.0));
        let radius = 4.0;
        let steps = 12;
        let mut accesses = Vec::new();

        for index in 0..steps {
            let a = std::f32::consts::TAU * index as f32 / steps as f32;
            let b = std::f32::consts::TAU * (index + 1) as f32 / steps as f32;
            accesses.push(ramp(
                center + Vec2::from_angle(a) * radius,
                center + Vec2::from_angle(b) * radius,
                index as f32 / steps as f32,
                (index + 1) as f32 / steps as f32,
            ));
        }

        let vertical = LevelVertical::from_parts(
            vec![
                floor(0.0, 0, &[IVec2::new(2, 2)]),
                floor(1.0, 1, &[IVec2::new(2, 2)]),
            ],
            accesses,
        );

        let sample = vertical
            .sample(
                &grid,
                center + Vec2::from_angle(std::f32::consts::PI) * radius,
                0.5,
            )
            .expect("mid-spiral should be walkable");
        assert!((sample.elevation - 0.5).abs() < 0.05);
    }

    #[test]
    fn sample_any_finds_the_highest_floor_at_a_position() {
        let grid = grid();
        let cell = IVec2::new(2, 2);
        let vertical = LevelVertical::from_parts(
            vec![floor(0.0, 0, &[cell]), floor(2.0, 2, &[cell])],
            Vec::new(),
        );
        let position = grid.cell_to_world(Vec2::new(2.0, 2.0));

        let sample = vertical
            .sample_any(&grid, position)
            .expect("stacked floors should be walkable");

        assert_eq!(sample.elevation, 2.0);
        assert_eq!(sample.rank, 2);
        assert!(vertical.sample(&grid, position, 0.0).unwrap().elevation == 0.0);
    }

    #[test]
    fn sample_any_uses_the_highest_ramp_end() {
        let grid = grid();
        let start = grid.cell_to_world(Vec2::new(0.0, 0.0));
        let end = grid.cell_to_world(Vec2::new(1.0, 0.0));
        let vertical = LevelVertical::from_parts(Vec::new(), vec![ramp(start, end, 0.0, 1.0)]);

        let sample = vertical
            .sample_any(&grid, end)
            .expect("ramp end should be walkable");

        assert!((sample.elevation - 1.0).abs() < 0.001);
        assert_eq!(sample.rank, 1);
    }
}
