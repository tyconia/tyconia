use crate::*;
use bevy::prelude::*;
use bevy::utils::HashSet;
use std::path::PathBuf;

pub struct ModProfilePlugin;

impl Plugin for ModProfilePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ModPack>().add_systems(
            OnEnter(GameState::Playing),
            (init_mod_profile, load_mods_from_profile)
                .chain()
                .after(levels::spawn_level),
        );
    }
}

use std::fs;

pub fn init_mod_profile(mut cmd: Commands, level: Query<Entity, With<Level>>) {
    let level = level.single();

    cmd.entity(level).insert(ModProfileConfig::default());
}

use bevy::reflect::serde::ReflectDeserializer;
use serde::de::DeserializeSeed;
use std::io::Read;

pub fn load_mods_from_profile(
    mut cmd: Commands,
    profile: Query<(Entity, &ModProfileConfig), With<Level>>,
    type_registry: Res<AppTypeRegistry>,
) {
    let type_registry = type_registry.read();
    info!("about to load mods!");
    // load platform specific app directories
    let project_dir = directories::ProjectDirs::from(
        env!("PROJECT_QUALIFIER"),
        env!("PROJECT_ORGANIZATION"),
        env!("PROJECT_APPLICATION"),
    )
    .expect("no valid home directory path could be retrieved from the operating system");

    let mod_dir = project_dir.config_dir().join("mods");

    let mut loaded_mods = HashSet::new();

    let (level_entity, profile_config) = profile.single();

    for mod_path in profile_config.0.iter().map(|shorthand| {
        let meta: Meta = shorthand.0.parse().unwrap();
        mod_dir.join(meta.mod_name)
    }) {
        let mod_meta_path = mod_path.join("meta.ron");

        match fs::File::open(&mod_meta_path) {
            Ok(mut mod_meta) => {
                let mut ron = String::new();
                mod_meta.read_to_string(&mut ron).map_or_else(
                    |err| {
                        error!(
                            "Unable to read ron at {}. {}",
                            mod_meta_path.to_string_lossy(),
                            err
                        );
                    },
                    |_| {
                        let mut deserializer = ron::de::Deserializer::from_str(&ron).unwrap();
                        let reflect_deserializer = ReflectDeserializer::new(&type_registry);

                        let partial_reflect_value =
                            reflect_deserializer.deserialize(&mut deserializer).unwrap();

                        let mod_pack = ModPack::from_reflect(&*partial_reflect_value).unwrap();

                        info!(
                            "loaded mod pack {} at {}",
                            mod_pack.mod_id,
                            mod_path.to_string_lossy()
                        );

                        loaded_mods.insert((mod_pack, mod_path));
                    },
                );
            }

            Err(error) => {
                error!(
                    "Failed to open mod meta file at {}. {}",
                    mod_meta_path.to_string_lossy(),
                    error
                );
            }
        }
    }

    cmd.entity(level_entity).insert(ModProfile(loaded_mods));
}

#[derive(Component, Reflect)]
pub struct ModProfileConfig(pub Vec<MetaShorthand>);

#[derive(Component, Reflect)]
pub struct ModProfile(pub HashSet<(ModPack, PathBuf)>);

impl ModProfile {
    const SCENARIO_PATH: &'static str = "scenarios";
    const MIGRATION_PATH: &'static str = "migrations";
    const ASSET_PATH: &'static str = "assets";

    pub fn scenario(&self) -> Option<PathBuf> {
        self.0.iter().find(|(_, _)| true);

        None
    }
}

impl Default for ModProfileConfig {
    fn default() -> Self {
        Self(vec!["tyconic_0.0.0-dev".into(), "base_0.0.0-dev".into()])
    }
}
