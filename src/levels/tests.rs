use super::*;
use bevy::reflect::serde::ReflectDeserializer;
use bevy_ecs_tilemap::prelude::*;
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
#[ignore = "rewrites asset files"]
fn write_level() {
    let mut app = App::new();
    app.register_type::<LevelIsometric>();
    app.register_type::<super::NpcDeclared>();
    app.register_type::<super::NpcKind>();
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
                scenes: vec![],
                colliders: vec![],
                floors: vec![],
                accesses: vec![],
                npcs: vec![],
                spawn: Vec2::ZERO,
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
