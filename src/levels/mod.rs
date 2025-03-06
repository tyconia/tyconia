mod chunks;
mod config;
mod editor;
mod logistics;
mod mini_game;
mod pack;

use crate::GameState;
use bevy::prelude::*;
use bevy::utils::HashMap;
use std::time::*;

pub use chunks::*;
pub use config::*;
pub use editor::*;
pub use logistics::*;
pub use mini_game::*;
pub use pack::*;

pub struct LevelsPlugin;

impl Plugin for LevelsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Playing),
            (spawn_level, init_map, spawn_map).chain(),
        )
        .add_plugins((
            //ChunkPlugin,
            TransportPlugin,
            ResearchEditorPlugin,
            //ModsMenuPlugin,
            //ToolBarPlugin,
        ));

        #[cfg(not(target_arch = "wasm32"))]
        app.register_type::<LevelManager>()
            .register_type::<ItemId>()
            .add_plugins(scripts::LevelsScriptingPlugin);
    }
}

#[derive(Component, Reflect, Default)]
pub struct LevelManager {
    pub iso: HashMap<String, LevelIsometric>,
}

#[derive(Component, Reflect)]
pub struct Level {
    //pub created_at: Instant,
    pub total_play_time: Duration,
}

#[derive(Component, Reflect, Debug)]
pub struct LevelIsometric {
    pub label: String,
    pub resources: HashMap<String, f32>,
    pub legend: Vec<ItemId>,
    pub surface: Vec<SurfaceDeclared>,
}

use crate::loading::ItemTextureMap;

type RawTile = (TilePos, usize);
type TexturedTile = (TilePos, Handle<Image>);
type RawMaps = Vec<(Vec2, (usize, usize), Vec<RawTile>)>;
type TexturedMaps = Vec<(Vec2, (usize, usize), Vec<TexturedTile>)>;

pub fn init_map(mut cmd: Commands, level: Query<(Entity, &ItemTextureMap), With<Level>>) {}
pub fn spawn_map(mut cmd: Commands, level: Query<(Entity, &ItemTextureMap), With<Level>>) {}

impl LevelIsometric {
    pub fn generate(
        &self,
        cmd: &mut Commands,
        maps: RawMaps,
        level: Query<(Entity, &ItemTextureMap), With<Level>>,
    ) {
        let (level_entity, item_texture_map) = level.single();
        let legend_textures = self.legend_textures(item_texture_map);
        let mut level = cmd.entity(level_entity);

        maps.into_iter().rev().enumerate().for_each(
            |(layer, (tilemap_position, size, indexed_tiles))| {
                level.with_children(|parent| {
                    let mut tilemap_entity_cmd = parent.spawn_empty();
                    let tilemap_entity_id = tilemap_entity_cmd.id();
                    let tilemap_id = TilemapId(tilemap_entity_id);

                    let map_size = TilemapSize::new(size.0 as u32, size.1 as u32);
                    let grid_size = TilemapGridSize::new(32., 64.);
                    let map_type = TilemapType::Isometric(IsoCoordSystem::Diamond);

                    indexed_tiles.into_iter().for_each(|(position, index)| {
                        tilemap_entity_cmd.with_children(|parent| {
                            parent.spawn(TileBundle {
                                position,
                                tilemap_id,
                                texture_index: TileTextureIndex(index as u32),
                                ..default()
                            });
                        });
                    });

                    parent.spawn(TilemapBundle {
                        grid_size,
                        map_type,
                        size: map_size,
                        texture: TilemapTexture::Vector(legend_textures.clone()),
                        transform: Transform::from_translation(tilemap_position.extend(0.))
                            * get_tilemap_center_transform(
                                &map_size,
                                &grid_size,
                                &map_type,
                                layer as f32,
                            ),
                        ..Default::default()
                    });
                });
            },
        );
    }

    pub fn legend_textures(&self, textures: &ItemTextureMap) -> Vec<Handle<Image>> {
        self.legend
            .as_slice()
            .into_iter()
            .map(|ItemId(id)| textures.0.get(id).unwrap().clone())
            .collect()
    }

    pub fn apply_texture(
        &self,
        tiles: &RawMaps,
        textures: Query<&ItemTextureMap, With<Level>>,
    ) -> TexturedMaps {
        let textures = textures.single();

        tiles
            .into_iter()
            .map(|(pos, map_size, map)| {
                let texture_map = map
                    .into_iter()
                    .map(|(tile_pos, index)| {
                        let item_id = self.legend.get(*index).unwrap();
                        let handle_image = textures.0.get(&item_id.0).unwrap().clone();
                        (*tile_pos, handle_image)
                    })
                    .collect::<Vec<TexturedTile>>();

                (pos.clone(), map_size.clone(), texture_map)
            })
            .collect::<Vec<_>>()
    }

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

#[derive(Reflect, Debug)]
pub struct SurfaceDeclared {
    pub position: Vec2,
    pub content: String,
}

impl From<(Vec2, String)> for SurfaceDeclared {
    fn from((position, content): (Vec2, String)) -> Self {
        Self { position, content }
    }
}

fn legend_to_index(c: char) -> usize {
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

use bevy_ecs_tilemap::prelude::*;

use std::fmt;

#[derive(Debug, PartialEq)]
pub enum ValidationError {
    InconsistentWidth,
    InconsistentHeight,
}

pub type ValidationResult = Result<(usize, usize), ValidationError>;

/// Validates that all provided maps have a constant width and height.
/// Returns Ok((width, height)) on success. An empty maps slice returns (0, 0).
fn validate_maps(maps: &[String]) -> ValidationResult {
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

fn parse_map_with_positions(map_str: &str) -> Vec<(TilePos, usize)> {
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

#[derive(Component)]
pub enum LevelLoadStatus {
    AssetsLoading,
    AssetsLoaded,
}

impl Default for Level {
    fn default() -> Self {
        Self {
            //created_at: Instant::now(),
            total_play_time: Duration::from_secs(1),
        }
    }
}

pub fn spawn_level(mut cmd: Commands) {
    cmd.spawn((
        Level::default(),
        LevelManager::default(),
        LevelLoadStatus::AssetsLoading,
        StateScoped(GameState::Playing),
    ));
}

#[cfg(not(target_arch = "wasm32"))]
pub mod scripts {
    use crate::*;
    use bevy::prelude::*;
    use bevy_mod_scripting::{
        core::{
            bindings::{function::script_function::*, script_value::ScriptValue, *},
            error::InteropError,
            handler::event_handler,
        },
        rhai::RhaiScriptingPlugin,
        script_bindings,
    };
    use std::sync::Arc;
    pub struct LevelsScriptingPlugin;

    impl Plugin for LevelsScriptingPlugin {
        fn build(&self, app: &mut App) {
            app.add_systems(Startup, register_levels)
                //.add_systems(
                //    OnEnter(GameState::Playing),
                //    (trigger_callback(
                //        callbacks::OnLevelStart,
                //        LevelManager { current: 12 },
                //    ),)
                //        .after(mods::load_scripts)
                //        .chain(),
                //)
                .register_type::<LevelManager>()
                .add_systems(
                    Update,
                    (
                        trigger_callback(callbacks::OnLevelStart, LevelManager),
                        event_handler::<callbacks::OnLevelStart, RhaiScriptingPlugin>,
                    )
                        .chain()
                        .run_if(in_state(GameState::Playing)),
                );
        }
    }

    #[derive(Component, Reflect, Clone)]
    pub struct LevelManager;

    use bevy::reflect::TypeRegistration;

    #[allow(dead_code)]
    #[script_bindings(name = "levels")]
    impl LevelManager {
        pub fn create(ctx: FunctionCallContext) {
            info!("Created level");
        }

        pub fn register(ctx: FunctionCallContext, label: String) -> Result<(), InteropError> {
            let world = ctx.world()?;
            let mut lvl_query = ScriptQueryBuilder::default();

            let lvl_manager = world
                .get_component_id(std::any::TypeId::of::<LevelManager>())?
                .unwrap();

            let lvl_registration = TypeRegistration::of::<LevelManager>();
            let lvl_registration = ScriptTypeRegistration::new(Arc::new(lvl_registration));
            let lvl_component = ScriptComponentRegistration::new(lvl_registration, lvl_manager);

            lvl_query.with_component(lvl_component);

            let ScriptQueryResult { entity, .. } = world.query(lvl_query)?.pop_front().unwrap();
            let level_manager = world.get_component(entity, lvl_manager)?.unwrap();
            level_manager.downcast::<LevelManager>(world).unwrap();
            info!("Registered level {}", label);
            // stuff are supposed to happen here

            Ok(())
        }
    }

    pub mod callbacks {
        use bevy_mod_scripting::core::callback_labels;

        callback_labels!(
            OnLevelConfigure => "on_level_configure",
            OnLevelStart => "on_level_start",
            OnLevelEnd => "on_level_end"
        );

        impl Clone for OnLevelStart {
            fn clone(&self) -> Self {
                Self
            }
        }
    }
}

pub mod post_processing {
    pub fn level(serialized: &str) -> String {
        let legend_comment = r#"For every tile character is an index corresponding to an item_id in this legend.
    // The mapping is defined as follows:
    //   * Digits ('0' through '9') are assigned indices 0 to 9.
    //   * Letters ('A', 'B', 'C', 'D', 'E', 'G', 'H', 'K', 'N', 'O', 'P', 'Q', 'R', 'S', 'U', 'Z')
    //     are assigned indices 10 to 25.
    //   * Symbols (e.g., '@', '#', '$', '%', '&', '*', '+', '=', '^', '~', '?', '!')
    //     are assigned indices 26 and above.
    //   * The blank symbol ('_') indicates no occupying tile.
    //   * Any character not covered will be treated as if blank."#;
        let surface_comment = r#"The `surface:` field holds every tilemap layout as a multiline string:
    //   * Each line corresponds to a row in the level grid.
    //   * Tiles each row are separated by the pipe character (`|`).
    //   * Each tile is represented by a single character defined in the legend.
    //   * Blank tiles are denoted by '_' and are omitted from world gen.
    //   * Edit this layout to modify the level's structure and tile placement.
    //   * Tip: Use a monospace font for readability."#;

        let serialized =
            insert_comment_preserve_indent_no_regex(&serialized, legend_comment, "legend:");

        let serialized =
            insert_comment_preserve_indent_no_regex(&serialized, surface_comment, "surface:");

        let serialized = wrap_surface_entries(&serialized);

        serialized
    }
    /// Wraps a single line containing a `content:` field by replacing the quoted
    /// string with a raw string literal. It converts literal "\n" sequences to actual newlines,
    /// so that the first line appears immediately after r#", and subsequent lines are indented with 18 spaces.
    pub fn wrap_content_line(line: &str) -> String {
        // Look for the "content:" token.
        if let Some(content_idx) = line.find("content:") {
            // Split the line into the part before "content:" and after.
            let (before, after_content) = line.split_at(content_idx);
            // Now, after_content should start with "content:".
            // Find the first double quote in after_content.
            if let Some(start_quote_idx) = after_content.find('\"') {
                let prefix = &after_content[..start_quote_idx]; // includes "content:" and any spaces before the quote
                let remainder = &after_content[start_quote_idx..]; // starts with the opening quote

                // Find the closing quote.
                if let Some(end_quote_idx) = remainder.rfind('\"') {
                    // Determine if there's a trailing comma after the closing quote.
                    let (quoted, trailing) = if remainder[end_quote_idx..].starts_with("\",") {
                        (&remainder[1..end_quote_idx], ",")
                    } else {
                        (&remainder[1..end_quote_idx], "")
                    };

                    // Replace literal "\n" sequences with actual newline characters.
                    let processed = quoted.replace("\\n", "\n");
                    // Split into lines.
                    let lines: Vec<&str> = processed.lines().collect();
                    let fixed_indent = "                    "; // 20 spaces for subsequent lines
                    let new_content = if lines.is_empty() {
                        format!("r#\"\"#")
                    } else {
                        let first_line = lines[0];
                        let mut out = format!("r#\"{}", first_line);
                        for line in lines.iter().skip(1) {
                            out.push('\n');
                            out.push_str(fixed_indent);
                            out.push_str(line);
                        }
                        out.push_str("\"#");
                        out
                    };
                    // Reassemble the line: original indentation + prefix + new raw string + trailing comma.
                    return format!("{}{}{}{}", before, prefix, new_content, trailing);
                }
            }
        }
        // If not a content line, return unchanged.
        line.to_string()
    }

    /// Processes an entire RON string and, for lines within the surface array that contain
    /// a `content:` field, wraps the quoted string as a raw string literal.
    pub fn wrap_surface_entries(ron_str: &str) -> String {
        let mut result = String::new();
        let mut in_surface = false;

        for line in ron_str.lines() {
            let trimmed = line.trim_start();
            // Detect the start of the surface array.
            if trimmed.starts_with("surface:") {
                in_surface = true;
                result.push_str(line);
                result.push('\n');
                continue;
            }
            // Detect the end of the surface array.
            if in_surface && (trimmed.starts_with("],") || trimmed.starts_with("]")) {
                in_surface = false;
                result.push_str(line);
                result.push('\n');
                continue;
            }
            // When inside the surface array, look for lines containing "content:".
            if in_surface && trimmed.contains("content:") {
                let new_line = wrap_content_line(line);
                result.push_str(&new_line);
                result.push('\n');
            } else {
                result.push_str(line);
                result.push('\n');
            }
        }
        result
    }

    pub fn wrap_map_entries(ron_str: &str) -> String {
        let mut result = String::new();
        let mut in_maps = false;

        for line in ron_str.lines() {
            let trimmed = line.trim_start();

            // Detect the start of the surface content.
            if trimmed.starts_with("content:") {
                in_maps = true;
                result.push_str(line);
                result.push('\n');
                continue;
            }

            // Detect the end of the maps array.
            if in_maps && (trimmed.starts_with("],") || trimmed.starts_with("]")) {
                in_maps = false;
                result.push_str(line);
                result.push('\n');
                continue;
            }

            if in_maps {
                // Process only lines that look like map entry string literals.
                if trimmed.starts_with("\"") {
                    let wrapped = wrap_map_entry_with_fixed_indent(line);
                    result.push_str(&wrapped);
                    result.push('\n');
                } else {
                    result.push_str(line);
                    result.push('\n');
                }
            } else {
                // Outside the maps array, copy the line as-is.
                result.push_str(line);
                result.push('\n');
            }
        }
        result
    }

    /// Wraps a single map entry in a raw string literal.
    /// Converts literal "\n" sequences into actual newlines and reindents
    /// subsequent lines with a fixed indent of 18 spaces.
    pub fn wrap_map_entry_with_fixed_indent(line: &str) -> String {
        // Capture the original indentation.
        let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();
        let trimmed = line.trim_start();
        if !trimmed.starts_with("\"") {
            return line.to_string();
        }
        // Remove the leading quote.
        let mut content = trimmed.strip_prefix("\"").unwrap_or(trimmed);

        // Remove the trailing quote (and optional trailing comma).
        let mut trailing = "";
        if content.ends_with("\",") {
            content = &content[..content.len() - 2];
            trailing = ",";
        } else if content.ends_with("\"") {
            content = &content[..content.len() - 1];
        }

        // Replace literal "\n" with actual newline characters.
        let content = content.replace("\\n", "\n");

        // Split the content into lines.
        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() {
            return format!("{}r#\"\"#{}", indent, trailing);
        }

        // The first line remains immediately after r#"
        let first_line = lines[0];
        // For subsequent lines, use a fixed indent of 20 spaces.
        let fixed_indent = "                    "; // 20 spaces
        let mut reindented = first_line.to_string();
        for line in lines.iter().skip(1) {
            reindented.push('\n');
            reindented.push_str(fixed_indent);
            reindented.push_str(line);
        }

        // Wrap the result in a raw string literal.
        format!("{}r#\"{}\"#{}", indent, reindented, trailing)
    }

    pub fn insert_comment_preserve_indent_no_regex(
        ron_str: &str,
        comment: &str,
        marker: &str,
    ) -> String {
        let mut result = String::new();

        // Process each line individually.
        for line in ron_str.lines() {
            // Check if the trimmed line starts with the marker.
            let trimmed = line.trim_start();
            if trimmed.starts_with(marker) {
                // Capture the leading whitespace manually.
                let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();
                // Insert the comment with the same indent.
                result.push_str(&format!("{}// {}\n", indent, comment));
            }
            // Add the original line back.
            result.push_str(line);
            result.push('\n');
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use bevy::reflect::serde::{ReflectDeserializer, ReflectSerializer};
    use serde::de::DeserializeSeed;
    use std::fs;
    use std::io::Read;
    #[test]
    fn test_valid_maps() {
        let maps = vec![
            r#"_ | A | 1
B | C | 2
D | E | 3"#
                .to_string(),
            r#"_ | F | 4
G | H | 5
I | J | 6"#
                .to_string(),
        ];

        assert_eq!(validate_maps(&maps), Ok((3, 3)));
    }

    #[test]
    fn test_invalid_width() {
        let maps = vec![
            r#"_ | A | 1
B | C | 2
D | E | 3"#
                .to_string(),
            r#"_ | F
G | H | 5
I | J | 6"#
                .to_string(), // Second map has a row with fewer columns.
        ];

        assert_eq!(
            validate_maps(&maps),
            Err(ValidationError::InconsistentWidth)
        );
    }

    #[test]
    fn test_invalid_height() {
        let maps = vec![
            r#"_ | A | 1
B | C | 2"#
                .to_string(), // Only 2 rows
            r#"_ | F | 4
G | H | 5
I | J | 6"#
                .to_string(), // 3 rows
        ];

        assert_eq!(
            validate_maps(&maps),
            Err(ValidationError::InconsistentHeight)
        );
    }

    #[test]
    fn test_empty_maps() {
        let maps: Vec<String> = vec![];
        assert_eq!(validate_maps(&maps), Ok((0, 0)));
    }
    #[test]
    fn write_level() {
        let mut app = App::new();
        app.register_type::<LevelIsometric>();
        app.add_systems(Startup, |type_registry: Res<AppTypeRegistry>| {
            let type_registry = type_registry.read();
            let mut map = HashMap::default();
            map.insert("money".into(), 200.);

            let level = LevelIsometric {
                label: "prologue_restaurant".into(),
                legend: vec!["base::auto_arm".into(), "base::mover_belt".into()],
                surface: vec![
                    (
                        Vec2::ZERO,
                        r#"_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | 0 | 1 | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | 0 | 1 | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | 0 | 1 | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | 0 | 1 | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | 0 | 1 | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | 0 | 1 | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | 0 | 1 | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _"#.to_string(),
                    )
                        .into(),
                    (
                        Vec2::ZERO,
                        r#"_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _
_ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _ | _"#.to_string(),
                    )
                        .into(),
                ],
                resources: map,
            };

            let reflect_serializer =
                bevy::reflect::serde::ReflectSerializer::new(&level, &type_registry);

            let serialized = ron::ser::to_string_pretty(
                &reflect_serializer,
                ron::ser::PrettyConfig::new()
                    .depth_limit(4)
                    .indentor("  ".into()),
            )
            .unwrap();

            let serialized = super::post_processing::level(&serialized);

            let file_path = std::path::Path::new("assets/mods/tyconic/assets/levels/nothing.ron");
            fs::write(&file_path, serialized).unwrap();

            let mut file = fs::File::open(file_path).unwrap();
            let mut ron = String::new();
            file.read_to_string(&mut ron).unwrap();
            let mut deserializer = ron::de::Deserializer::from_str(&ron).unwrap();
            let reflect_deserializer = ReflectDeserializer::new(&type_registry);

            let partial_reflect_value =
                reflect_deserializer.deserialize(&mut deserializer).unwrap();
            let level_declared = LevelIsometric::from_reflect(&*partial_reflect_value).unwrap();
            let file_path = std::path::Path::new("assets/mods/tyconic/assets/levels/out.txt");
            fs::write(
                &file_path,
                format!("{:#?}", level_declared.tiles_with_texture_index()),
            )
            .unwrap();
        });

        app.run();
    }

    #[test]
    fn test_legend_to_index() {
        // Digits
        assert_eq!(legend_to_index('0'), 0);
        assert_eq!(legend_to_index('5'), 5);

        // Letters
        assert_eq!(legend_to_index('A'), 10);
        assert_eq!(legend_to_index('Z'), 25);

        // Symbols
        assert_eq!(legend_to_index('@'), 26);
        assert_eq!(legend_to_index('!'), 37);

        // Blank should return usize::MAX
        assert_eq!(legend_to_index('_'), usize::MAX);

        // An undefined symbol should panic
        let result = std::panic::catch_unwind(|| legend_to_index('F'));
        assert!(result.is_err(), "Expected panic on undefined symbol");
    }

    #[test]
    fn test_parse_map_with_positions() {
        let map_str = r#"_ | A | 1 | G | %
H | 3 | _ | O | !
K | 0 | 2 | D | _"#;

        let result = parse_map_with_positions(map_str);

        // Expected tokens per row (skipping blanks):
        // Row 0: tokens: "A", "1", "G", "%"  → 4 tokens
        // Row 1: tokens: "H", "3", "O", "!"  → 4 tokens
        // Row 2: tokens: "K", "0", "2", "D"  → 4 tokens
        // Total = 12 tokens.
        assert_eq!(result.len(), 12);

        // Check a few known positions and their tile indices:
        // Row 0, token "A" at x = 1, y = 0 → index should be 10.
        assert_eq!(result[0].0, TilePos { x: 1, y: 0 });
        assert_eq!(result[0].1, 10);

        // Row 0, token "1" at x = 2, y = 0 → index should be 1.
        assert_eq!(result[1].0, TilePos { x: 2, y: 0 });
        assert_eq!(result[1].1, 1);

        // Row 1, token "H" at x = 0, y = 1 → index should be 16.
        // Row 1 tokens: "H" (x=0), "3" (x=1), skip "_" at x=2, "O" (x=3), "!" (x=4)
        assert_eq!(result[4].0, TilePos { x: 0, y: 1 });
        assert_eq!(result[4].1, 16);

        // Ensure that blank tokens are never included.
        for &(_, index) in result.iter() {
            assert!(index != usize::MAX, "Blank tile should not be in output");
        }
    }
}
