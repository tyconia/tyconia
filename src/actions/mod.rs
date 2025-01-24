//! This module handle input to action lifecycles sending events to their respective channels

//use crate::loading::ConfigAssets;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::utils::HashSet;

pub(crate) mod cursors;
pub(crate) mod inter_action;
pub(crate) mod mappings;
pub(crate) mod movement;
pub(crate) mod ui;

pub use cursors::*;
pub use inter_action::*;
pub use mappings::*;
pub use movement::*;
pub use ui::*;

use crate::GameState;

/// This plugin listens for [`DesktopControl`] input and converts into Actions which
/// can then be used as a resource in other systems to act on the player input
pub struct ActionsPlugin;

impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        
        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(Startup, (load_input_map,).chain());

        #[cfg(target_arch = "wasm32")]
        app.add_systems(Startup, (load_input_map,).chain());

        app.add_plugins((mappings::InputMappingsPlugin, cursors::CursorsPlugin))
            .add_event::<movement::MovementAction>()
            .add_event::<ui::UiAction>()
            .add_event::<inter_action::InterAction>()
            .add_systems(
                Update,
                (set_ui_actions,).run_if(in_state(GameState::Playing)),
            );
    }
}

pub trait InputAction {
    fn display(&self) -> String;

    fn desktop_mapping(&self, input_mapping: &Res<InputMappings>) -> Option<InputMappingEntry>;
}

use bevy::reflect::serde::{ReflectDeserializer, ReflectSerializer};
use serde::de::DeserializeSeed;
use std::fs::File;
use std::io::{Read, Write};

pub const DEFAULT_MAPPINGS: &'static str = include_str!("../../assets/config/input_mappings.ron");

// Loads input map
#[cfg(target_arch = "wasm32")]
fn load_input_map(type_registry: Res<AppTypeRegistry>, mut map: ResMut<mappings::InputMappings>) {
    let mut deserializer = ron::de::Deserializer::from_str(&DEFAULT_MAPPINGS).unwrap();
    let type_registry = type_registry.read();
    let reflect_deserializer = ReflectDeserializer::new(&type_registry);

    let partial_reflect_value =
        reflect_deserializer.deserialize(&mut deserializer).unwrap();

    *map = mappings::InputMappings::from_reflect(&*partial_reflect_value).unwrap();
}

// Loads input map
#[cfg(not(target_arch = "wasm32"))]
fn load_input_map(type_registry: Res<AppTypeRegistry>, mut map: ResMut<mappings::InputMappings>) {
    // load platform specific app directories
    let project_dir = directories::ProjectDirs::from(
        env!("PROJECT_QUALIFIER"),
        env!("PROJECT_ORGANIZATION"),
        env!("PROJECT_APPLICATION"),
    )
    .expect("no valid home directory path could be retrieved from the operating system");

    let config_dir = project_dir.config_dir();

    let type_registry = type_registry.read();
    let file_name = std::path::Path::new("input_mappings.ron");
    let file_path = config_dir.join(file_name);

    let ron = File::open(&file_path).map_or_else(
        |err| {
            error!(
                "Failed to open file {}, at {}, Loaded default configuration instead",
                err,
                file_path.to_string_lossy()
            );

            DEFAULT_MAPPINGS.to_string()
        },
        |mut file| {
            let mut ron = String::new();
            file.read_to_string(&mut ron).unwrap();
            ron
        },
    );

    let mut deserializer = ron::de::Deserializer::from_str(&ron).unwrap();
    let reflect_deserializer = ReflectDeserializer::new(&type_registry);

    let partial_reflect_value =
        reflect_deserializer.deserialize(&mut deserializer).unwrap();

    *map = mappings::InputMappings::from_reflect(&*partial_reflect_value).unwrap();
}

fn print_input_map(type_registry: ResMut<AppTypeRegistry>, map: Res<mappings::InputMappings>) {
    let type_registry = type_registry.read();

    let mut map = mappings::InputMappings::default();

    map.ui_actions.insert(
        ui::UiAction::Menu,
        mappings::InputMappingEntry {
            primary: vec![DesktopControl::Key(KeyCode::Escape)],
            secondary: vec![DesktopControl::Key(KeyCode::KeyP)],
            ..default()
        },
    );

    map.ui_actions.insert(
        ui::UiAction::HotbarSlot(0),
        mappings::InputMappingEntry {
            primary: vec![DesktopControl::Key(KeyCode::Digit1)],
            ..default()
        },
    );
    map.ui_actions.insert(
        ui::UiAction::HotbarSlot(1),
        mappings::InputMappingEntry {
            primary: vec![DesktopControl::Key(KeyCode::Digit2)],
            ..default()
        },
    );
    map.ui_actions.insert(
        ui::UiAction::HotbarSlot(2),
        mappings::InputMappingEntry {
            primary: vec![DesktopControl::Key(KeyCode::Digit3)],
            ..default()
        },
    );
    map.ui_actions.insert(
        ui::UiAction::HotbarSlot(3),
        mappings::InputMappingEntry {
            primary: vec![DesktopControl::Key(KeyCode::Digit4)],
            ..default()
        },
    );
    map.ui_actions.insert(
        ui::UiAction::HotbarSlot(4),
        mappings::InputMappingEntry {
            primary: vec![DesktopControl::Key(KeyCode::Digit5)],
            ..default()
        },
    );
    map.ui_actions.insert(
        ui::UiAction::HotbarSlot(5),
        mappings::InputMappingEntry {
            primary: vec![DesktopControl::Key(KeyCode::Digit6)],
            ..default()
        },
    );
    map.ui_actions.insert(
        ui::UiAction::HotbarSlot(6),
        mappings::InputMappingEntry {
            primary: vec![DesktopControl::Key(KeyCode::Digit7)],
            ..default()
        },
    );

    map.movement_actions.insert(
        movement::MovementAction::North,
        mappings::InputMappingEntry {
            primary: vec![DesktopControl::Key(KeyCode::KeyW)],
            secondary: vec![DesktopControl::Key(KeyCode::ArrowUp)],
            ..default()
        },
    );
    map.movement_actions.insert(
        movement::MovementAction::South,
        mappings::InputMappingEntry {
            primary: vec![DesktopControl::Key(KeyCode::KeyS)],
            secondary: vec![DesktopControl::Key(KeyCode::ArrowDown)],
            ..default()
        },
    );
    map.movement_actions.insert(
        movement::MovementAction::East,
        mappings::InputMappingEntry {
            primary: vec![DesktopControl::Key(KeyCode::KeyD)],
            secondary: vec![DesktopControl::Key(KeyCode::ArrowRight)],
            ..default()
        },
    );
    map.movement_actions.insert(
        movement::MovementAction::West,
        mappings::InputMappingEntry {
            primary: vec![DesktopControl::Key(KeyCode::KeyA)],
            secondary: vec![DesktopControl::Key(KeyCode::ArrowLeft)],
            ..default()
        },
    );

    let serializer = ReflectSerializer::new(&map, &type_registry);
    let pretty_config = ron::ser::PrettyConfig::default()
        .compact_arrays(true)
        .depth_limit(3);

    let ron = ron::ser::to_string_pretty(&serializer, pretty_config).unwrap();

    let qualifier = env!("PROJECT_QUALIFIER");
    let organization = env!("PROJECT_ORGANIZATION");
    let application = env!("PROJECT_APPLICATION");
    let project_dir = directories::ProjectDirs::from(qualifier, organization, application).unwrap();
    let config_dir = project_dir.config_dir();

    let file_name = std::path::Path::new("input_mappings.ron");
    let file_path = config_dir.join(file_name);

    //input_mappings_config
    match File::create(&file_path) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(ron.as_bytes()) {
                error!(
                    "Failed to write data to file: {}, path is {}",
                    e,
                    file_path.to_string_lossy()
                );
            } else {
                info!("Data successfully written to {}", file_path.display());
            }
        }
        Err(e) => error!("Failed to create file: {}", e),
    }

    match File::open(&file_path) {
        Ok(mut file) => {
            let mut ron = String::new();
            file.read_to_string(&mut ron).unwrap();

            let mut deserializer = ron::de::Deserializer::from_str(&ron).unwrap();
            let reflect_deserializer = ReflectDeserializer::new(&type_registry);

            let reflect_value: Box<dyn PartialReflect> =
                reflect_deserializer.deserialize(&mut deserializer).unwrap();

            info!("Data is {:?}", reflect_value);
        }
        Err(e) => error!(
            "Failed to open file {}, at {}",
            e,
            file_path.to_string_lossy()
        ),
    }
}

/// Mouse wheel movement to Zoom actions
pub fn set_ui_actions(
    mut ui_actions: EventWriter<ui::UiAction>,
    mut mouse_wheel: EventReader<MouseWheel>,
) {
    let mw_movement: f32 = mouse_wheel.read().map(|m| m.y).sum();
    if mw_movement.round() as isize != 0 {
        ui_actions.send(ui::UiAction::Zoom(mw_movement.round() as isize));
    }
}

/// Keyboard movement to player movement actions
/// TODO: Support touch input
pub fn set_movement_actions(
    mut player_movement_action: EventWriter<movement::MovementAction>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut held: Local<HashSet<KeyCode>>,
) {
    const KEYS: [KeyCode; 8] = [
        KeyCode::KeyW,
        KeyCode::KeyS,
        KeyCode::KeyA,
        KeyCode::KeyD,
        KeyCode::ArrowUp,
        KeyCode::ArrowDown,
        KeyCode::ArrowLeft,
        KeyCode::ArrowRight,
    ];

    // Add new keys to the 'held' set
    for key in KEYS.iter() {
        if keyboard_input.just_pressed(*key) {
            held.insert(*key);
        }
    }

    (match &KEYS
        .iter()
        .map(|k| held.contains(&*k))
        .collect::<Vec<bool>>()[0..4]
    {
        &[true, false, false, false] => Some(movement::MovementAction::North),
        _ => None,
    })
    .map(|a| {
        player_movement_action.send(a);
    });

    // Remove keys that were just released
    for key in KEYS.iter() {
        if keyboard_input.just_released(*key) {
            held.remove(&*key);
        }
    }
}
