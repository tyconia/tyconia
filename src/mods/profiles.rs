use crate::*;
use bevy::prelude::*;
use bevy::utils::HashSet;
use std::path::PathBuf;

pub struct ModProfilePlugin;

impl Plugin for ModProfilePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ModPack>()
            .init_resource::<PendingModProfile>()
            .add_systems(
                OnEnter(GameState::Playing),
                (init_mod_profile, load_mods_from_profile)
                    .chain()
                    .after(levels::spawn_level),
            );
    }
}

use std::fs;

pub fn init_mod_profile(
    mut cmd: Commands,
    level: Query<Entity, With<Level>>,
    pending: Res<PendingModProfile>,
) {
    let Ok(level) = level.get_single() else {
        warn!("Skipping mod profile setup: expected exactly one level entity");
        return;
    };

    cmd.entity(level).insert(pending.0.clone());
}

use bevy::reflect::serde::ReflectDeserializer;
use serde::de::DeserializeSeed;
use std::io::Read;

#[cfg(target_arch = "wasm32")]
pub fn load_mods_from_profile(
    mut cmd: Commands,
    profile: Query<(Entity, &ModProfileConfig), With<Level>>,
    type_registry: Res<AppTypeRegistry>,
) {
    let type_registry = type_registry.read();
    info!("about to load mods!");

    let mut loaded_mods = HashSet::new();

    let Ok((level_entity, profile_config)) = profile.get_single() else {
        warn!("Skipping mod loading: expected exactly one level entity");
        return;
    };

    cmd.entity(level_entity).insert(ModProfile(loaded_mods));
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_mods_from_profile(
    mut cmd: Commands,
    profile: Query<(Entity, &ModProfileConfig), With<Level>>,
    type_registry: Res<AppTypeRegistry>,
) {
    let type_registry = type_registry.read();
    info!("about to load mods!");
    // load platform specific app directories
    let Some(project_dir) = directories::ProjectDirs::from(
        env!("PROJECT_QUALIFIER"),
        env!("PROJECT_ORGANIZATION"),
        env!("PROJECT_APPLICATION"),
    ) else {
        warn!("Skipping mod loading: no valid home directory path could be retrieved");
        return;
    };

    let mod_dir = project_dir.config_dir().join("mods");

    let mut loaded_mods = HashSet::new();

    let Ok((level_entity, profile_config)) = profile.get_single() else {
        warn!("Skipping mod loading: expected exactly one level entity");
        return;
    };

    for mod_path in
        profile_config
            .0
            .iter()
            .filter_map(|shorthand| match shorthand.0.parse::<Meta>() {
                Ok(meta) => Some(mod_dir.join(meta.mod_name)),
                Err(error) => {
                    warn!(
                        "Skipping invalid mod profile entry {:?}: {error:?}",
                        shorthand.0
                    );
                    None
                }
            })
    {
        let mod_meta_path = mod_path.join("meta.ron");

        match fs::File::open(&mod_meta_path) {
            Ok(mut mod_meta) => {
                let mut ron = String::new();
                if let Err(error) = mod_meta.read_to_string(&mut ron) {
                    error!(
                        "Unable to read ron at {}. {}",
                        mod_meta_path.to_string_lossy(),
                        error
                    );
                    continue;
                }

                let Ok(mut deserializer) = ron::de::Deserializer::from_str(&ron) else {
                    error!("Unable to parse ron at {}", mod_meta_path.to_string_lossy());
                    continue;
                };
                let reflect_deserializer = ReflectDeserializer::new(&type_registry);

                let Ok(partial_reflect_value) = reflect_deserializer.deserialize(&mut deserializer)
                else {
                    error!(
                        "Unable to deserialize ron at {}",
                        mod_meta_path.to_string_lossy()
                    );
                    continue;
                };

                let Some(mod_pack) = ModPack::from_reflect(&*partial_reflect_value) else {
                    error!(
                        "Unable to reflect mod pack from {}",
                        mod_meta_path.to_string_lossy()
                    );
                    continue;
                };

                info!(
                    "loaded mod pack {} at {}",
                    mod_pack.mod_id,
                    mod_path.to_string_lossy()
                );

                loaded_mods.insert((mod_pack, mod_path));
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

#[derive(Component, Reflect, Clone, Debug)]
pub struct ModProfileConfig(pub HashSet<MetaShorthand>);

#[derive(Resource, Reflect, Clone, Debug, Default)]
pub struct PendingModProfile(pub ModProfileConfig);

#[derive(Component, Reflect)]
pub struct ModProfile(pub HashSet<(ModPack, PathBuf)>);

impl Default for ModProfileConfig {
    fn default() -> Self {
        Self(["tyconic_0.0.0-dev".into(), "base_0.0.0-dev".into()].into())
    }
}
