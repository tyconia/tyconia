use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_ecs_tilemap::prelude::*;

use super::declared::{
    AccessDeclared, ColliderDeclared, FloorDeclared, NpcDeclared, SceneDeclared,
};
use super::ItemId;

#[derive(Component, Reflect, Debug, Clone)]
#[type_path = "tyconia::levels"]
pub struct LevelIsometric {
    pub label: String,
    pub resources: HashMap<String, f32>,
    pub legend: Vec<ItemId>,
    pub surface: Vec<SurfaceDeclared>,
    #[reflect(default)]
    pub scenes: Vec<SceneDeclared>,
    #[reflect(default)]
    pub colliders: Vec<ColliderDeclared>,
    #[reflect(default)]
    pub floors: Vec<FloorDeclared>,
    #[reflect(default)]
    pub accesses: Vec<AccessDeclared>,
    #[reflect(default)]
    pub npcs: Vec<NpcDeclared>,
    #[reflect(default)]
    pub spawn: Vec2,
}

type RawTile = (TilePos, usize);
type RawMaps = Vec<(Vec2, (usize, usize), Vec<RawTile>)>;

impl LevelIsometric {
    pub fn tiles_with_texture_index(&self) -> Result<RawMaps, ValidationError> {
        let maps: Vec<String> = self
            .surface
            .iter()
            .map(|surface| surface.content.clone())
            .collect();

        let map_size = validate_maps(&maps)?;

        let mut tiles = vec![];
        for content in &self.surface {
            let tiles_indexed = parse_map_with_positions(&content.content);
            tiles.push((content.position, map_size, tiles_indexed));
        }

        Ok(tiles)
    }
}

#[derive(Reflect, Debug, Clone)]
#[type_path = "tyconia::levels"]
pub struct SurfaceDeclared {
    pub position: Vec2,
    pub content: String,
}

impl From<(Vec2, String)> for SurfaceDeclared {
    fn from((position, content): (Vec2, String)) -> Self {
        Self { position, content }
    }
}

pub(super) fn legend_to_index(c: char) -> usize {
    match c {
        // Digits first (indices 0–9)
        '0' => 0,
        '1' => 1,
        '2' => 2,
        '3' => 3,
        '4' => 4,
        '5' => 5,
        '6' => 6,
        '7' => 7,
        '8' => 8,
        '9' => 9,

        // Letters next (indices 10+)
        'A' => 10,
        'B' => 11,
        'C' => 12,
        'D' => 13,
        'E' => 14,
        'G' => 15,
        'H' => 16,
        'K' => 17,
        'N' => 18,
        'O' => 19,
        'P' => 20,
        'Q' => 21,
        'R' => 22,
        'S' => 23,
        'U' => 24,
        'Z' => 25,

        // Symbols (indices 26+)
        '@' => 26,
        '#' => 27,
        '$' => 28,
        '%' => 29,
        '&' => 30,
        '*' => 31,
        '+' => 32,
        '=' => 33,
        '^' => 34,
        '~' => 35,
        '?' => 36,
        '!' => 37,

        // Blank
        '_' => usize::MAX,

        _ => panic!("Undefined tile '{}'", c),
    }
}

#[derive(Debug, PartialEq)]
pub enum ValidationError {
    InconsistentWidth,
    InconsistentHeight,
}

pub type ValidationResult = Result<(usize, usize), ValidationError>;

/// Validates that all provided maps have a constant width and height.
/// Returns Ok((width, height)) on success. An empty maps slice returns (0, 0).
pub(super) fn validate_maps(maps: &[String]) -> ValidationResult {
    if maps.is_empty() {
        return Ok((0, 0));
    }

    // Use the first map as the reference.
    let expected_height = maps[0].lines().count();
    let mut expected_width = None;

    for map in maps {
        let lines: Vec<&str> = map.lines().collect();
        let height = lines.len();

        if height != expected_height {
            return Err(ValidationError::InconsistentHeight);
        }

        for line in &lines {
            // Here, we assume that tiles are separated by the pipe character.
            // The number of columns is determined by the number of segments when splitting.
            let width = line.split('|').count();
            match expected_width {
                Some(w) if w != width => return Err(ValidationError::InconsistentWidth),
                None => expected_width = Some(width),
                _ => {}
            }
        }
    }

    Ok((expected_width.unwrap_or(0), expected_height))
}

pub(super) fn parse_map_with_positions(map_str: &str) -> Vec<(TilePos, usize)> {
    let mut output = Vec::new();

    // Iterate over lines with enumeration for the y coordinate.
    for (y, line) in map_str.lines().enumerate() {
        let trimmed_line = line.trim();
        if trimmed_line.is_empty() {
            continue;
        }

        // Split each line on the separator " | "
        let tokens: Vec<&str> = trimmed_line.split('|').map(|token| token.trim()).collect();

        // Iterate over tokens with enumeration for the x coordinate.
        for (x, token) in tokens.iter().enumerate() {
            // We expect each token to be a single character.
            if let Some(c) = token.chars().next() {
                let pos = TilePos {
                    x: x as u32,
                    y: y as u32,
                };
                let index = legend_to_index(c);
                // skip blanks
                if index == usize::MAX {
                    continue;
                }
                output.push((pos, index));
            } else {
                error!("Empty token encountered at line {} column {}", y, x);
            }
        }
    }
    output
}
