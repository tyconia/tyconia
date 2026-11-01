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

    fn bindable(&self) -> BindableAction;
}

use bevy::reflect::serde::{ReflectDeserializer, ReflectSerializer};
use serde::de::DeserializeSeed;
use std::fs::File;
use std::io::{Read, Write};

pub const DEFAULT_MAPPINGS: &'static str = include_str!("../../assets/config/input_mappings.ron");

// Loads input map
#[cfg(target_arch = "wasm32")]
fn load_input_map(type_registry: Res<AppTypeRegistry>, mut map: ResMut<mappings::InputMappings>) {
    if let Some(defaults) = default_input_mappings(&type_registry) {
        *map = defaults;
    }
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

    let partial_reflect_value = reflect_deserializer.deserialize(&mut deserializer).unwrap();

    *map = mappings::InputMappings::from_reflect(&*partial_reflect_value).unwrap();
}

pub fn default_input_mappings(type_registry: &AppTypeRegistry) -> Option<InputMappings> {
    let mut deserializer = ron::de::Deserializer::from_str(DEFAULT_MAPPINGS).ok()?;
    let registry = type_registry.read();
    let reflect_deserializer = ReflectDeserializer::new(&registry);
    let partial_reflect_value = reflect_deserializer.deserialize(&mut deserializer).ok()?;
    InputMappings::from_reflect(&*partial_reflect_value)
}

pub fn save_input_map(type_registry: &AppTypeRegistry, map: &InputMappings) {
    let registry = type_registry.read();
    let serializer = ReflectSerializer::new(map, &registry);
    let pretty_config = ron::ser::PrettyConfig::default()
        .compact_arrays(true)
        .depth_limit(3);

    let ron = match ron::ser::to_string_pretty(&serializer, pretty_config) {
        Ok(ron) => ron,
        Err(error) => {
            error!("Failed to serialize input mappings: {}", error);
            return;
        }
    };

    let Some(project_dir) = directories::ProjectDirs::from(
        env!("PROJECT_QUALIFIER"),
        env!("PROJECT_ORGANIZATION"),
        env!("PROJECT_APPLICATION"),
    ) else {
        error!("No valid configuration directory could be found");
        return;
    };

    let config_dir = project_dir.config_dir();
    if let Err(error) = std::fs::create_dir_all(config_dir) {
        error!("Failed to create configuration directory: {}", error);
        return;
    }

    let file_path = config_dir.join("input_mappings.ron");
    match File::create(&file_path) {
        Ok(mut file) => {
            if let Err(error) = file.write_all(ron.as_bytes()) {
                error!(
                    "Failed to write input mappings to {}: {}",
                    file_path.to_string_lossy(),
                    error
                );
            }
        }
        Err(error) => error!(
            "Failed to create {}: {}",
            file_path.to_string_lossy(),
            error
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

    for key in KEYS.iter() {
        if keyboard_input.just_released(*key) {
            held.remove(&*key);
        }
    }
}
