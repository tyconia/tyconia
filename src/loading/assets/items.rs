use bevy::prelude::*;
//use bevy::utils::HashMap;
use std::{collections::HashMap, fs, path::PathBuf};

use crate::*;

#[derive(Debug, PartialEq, Clone, Reflect, Eq, Hash)]
pub enum ItemTextureSource {
    // Specifies the asset path inside textures
    // `~/.config/tyconic/mods/mod_name/assets/textures/item_id.png`
    Path(PathBuf),
    // Infer will derive an asset path like
    // `~/.config/tyconic/mods/mod_name/assets/textures/item_id.png`
    Auto,
    // InferWithStates will derive an asset path like
    // `~/.config/tyconic/mods/mod_name/assets/textures/item_id.png`
    // including graphic states like `item_id--back-facing.png` and `item_id--broken.png`
    AutoWithVariants(Vec<String>),
}

#[derive(Debug, PartialEq, Clone, Reflect, Eq, Hash)]
pub struct ItemTexturePath(pub PathBuf);

#[derive(Component, Reflect)]
pub struct ItemTextureMapSource(pub HashMap<String, ItemTextureSource>);

#[derive(Component, Reflect)]
pub struct ItemTextureMapConfig(pub HashMap<String, ItemTexturePath>);

#[derive(Component, Reflect)]
pub struct ItemTextureMap(pub HashMap<String, Handle<Image>>);

pub struct ItemTextureMapPlugin;

impl Plugin for ItemTextureMapPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ItemTextureMapSource>().add_systems(
            OnEnter(GameState::Playing),
            (
                init_item_asset_map,
                load_item_asset_map,
                loading_item_asset_map,
            )
                .chain()
                .after(load_mods_from_profile),
        );
    }
}

pub const ITEM_ASSETS_RON: &str = "derivations/item_assets.ron";
pub const ITEM_ASSETS_PATH: &str = "assets/textures";

use bevy::reflect::serde::ReflectDeserializer;
use serde::de::DeserializeSeed;
use std::io::Read;

pub fn init_item_asset_map(
    mut cmd: Commands,
    type_registry: Res<AppTypeRegistry>,
    level: Query<(Entity, &ModProfile), With<Level>>,
) {
    let (level_entity, mod_profile) = level.single();
    let item_texture_path = PathBuf::from(ITEM_ASSETS_RON);
    let type_registry = type_registry.read();

    let item_texture_pairs = mod_profile
        .0
        .iter()
        .map(|(mod_pack, path)| {
            // join mod directory with texture path
            let item_texture_path = path.join(item_texture_path.clone());

            (mod_pack, item_texture_path, path)
        })
        .map(|(mod_pack, item_texture_path, mod_directory)| {
            fs::File::open(&item_texture_path)
                .map_or_else(
                    |err| {
                        error!(
                            "Failed to open file {}, at {}, Loaded default configuration instead",
                            err,
                            item_texture_path.to_string_lossy()
                        );
                        None
                    },
                    |mut file| {
                        let mut ron = String::new();
                        file.read_to_string(&mut ron).unwrap();

                        let mut deserializer = ron::de::Deserializer::from_str(&ron).unwrap();
                        let reflect_deserializer = ReflectDeserializer::new(&type_registry);
                        //
                        let partial_reflect_value =
                            reflect_deserializer.deserialize(&mut deserializer).unwrap();

                        let map_config =
                            ItemTextureMapSource::from_reflect(&*partial_reflect_value).unwrap();

                        let map_config = map_config
                            .0
                            .iter()
                            .map(|(item_id, asset_source)| {
                                //let new_item_id = format!(
                                //    "::{}::{}::{}",
                                //    mod_pack.meta.namespace, mod_pack.meta.mod_name, item_id
                                //);
                                let new_item_id =
                                    format!("{}::{}", mod_pack.mod_id.mod_name, item_id);

                                let asset_source = match asset_source {
                                    ItemTextureSource::Auto => ItemTexturePath(
                                        item_texture_path
                                            .join(mod_directory)
                                            .join(ITEM_ASSETS_PATH)
                                            .join(format!("{}.png", item_id)),
                                    ),

                                    ItemTextureSource::Path(path) => ItemTexturePath(path.clone()),

                                    _ => unimplemented!(),
                                };

                                debug!(
                                    "loading item {} at {}",
                                    new_item_id,
                                    asset_source.0.to_string_lossy()
                                );
                                (new_item_id.clone(), asset_source)
                            })
                            .collect::<HashMap<String, ItemTexturePath>>();

                        Some(map_config)
                    },
                )
                .unwrap()
        })
        .fold(HashMap::new(), |mut acc, map| {
            acc.extend(map);
            acc
        });

    cmd.entity(level_entity)
        .insert(ItemTextureMapConfig(item_texture_pairs));
}

pub fn load_item_asset_map(
    mut cmd: Commands,
    level: Query<(Entity, &ItemTextureMapConfig), With<Level>>,
    asset_server: Res<AssetServer>,
) {
    let (level_entity, item_texture_config) = level.single();

    let assets = item_texture_config
        .0
        .clone()
        .into_iter()
        .map(|(id, config)| {
            let handle = asset_server.load(config.0);
            (id, handle)
        })
        .collect();

    cmd.entity(level_entity).insert(ItemTextureMap(assets));
}

// TODO:
// * texture not loaded is 0 units of loading
// * texture loading is 1 unit of loading
// * texture loaded or failed is 2 units of loading
pub fn loading_item_asset_map(
    //mut cmd: Commands,
    level: Query<(Entity, &ItemTextureMap), With<Level>>,
    asset_server: Res<AssetServer>,
    mut loaded_assets: Local<Vec<String>>,
) {
    let (_level_entity, item_texture_map) = level.single();

    let mut loaded_assets_buffer = vec![];
    let total_loading_units = (item_texture_map.0.iter().count() * 2) as f32;
    let mut current_loading_units = 0.;

    for (ident, handle) in item_texture_map
        .0
        .iter()
        .filter(|(ident, _)| !loaded_assets.contains(ident))
    {
        let loading_unit = if let Some(load_state) = asset_server.get_load_state(handle) {
            match load_state {
                bevy::asset::LoadState::NotLoaded => Some(0),
                bevy::asset::LoadState::Loading => Some(1),
                bevy::asset::LoadState::Loaded => {
                    loaded_assets_buffer.push(ident.clone());
                    None
                }
                bevy::asset::LoadState::Failed(asset_load_err) => {
                    error!("failed to load texture {}. {}", ident, asset_load_err);
                    loaded_assets_buffer.push(ident.clone());
                    None
                }
            }
        } else {
            error!("no load state found {}", ident);
            None
        };

        loading_unit.map(|unit| current_loading_units += unit as f32);
    }

    loaded_assets.extend(loaded_assets_buffer);
    current_loading_units = current_loading_units + (loaded_assets.iter().count() * 2) as f32;

    info!(
        "item texture progress {} / {}. {}%",
        current_loading_units / 2.,
        total_loading_units / 2.,
        current_loading_units / total_loading_units * 100.
    );
}

mod tests {
    use super::*;

    #[test]
    fn write_asset_map() {
        let base_asset_map_source = [
            ("auto_arm".into(), ItemTextureSource::Auto),
            ("mover_belt".into(), ItemTextureSource::Auto),
            ("infinite_io".into(), ItemTextureSource::Auto),
        ];

        let tyconic_asset_map_source = [
            ("pizza_slice".into(), ItemTextureSource::Auto),
            ("hamburger".into(), ItemTextureSource::Auto),
            ("hot_choco".into(), ItemTextureSource::Auto),
            ("beef_slab".into(), ItemTextureSource::Auto),
            ("potato_medium".into(), ItemTextureSource::Auto),
            ("french_fries".into(), ItemTextureSource::Auto),
            ("cheese_wheel".into(), ItemTextureSource::Auto),
            ("bread_loaf".into(), ItemTextureSource::Auto),
        ];

        let base_asset_map_source = ItemTextureMapSource(base_asset_map_source.into());
        let tyconic_asset_map_source = ItemTextureMapSource(tyconic_asset_map_source.into());

        let mut app = App::new();
        app.register_type::<ItemTextureMapSource>();
        app.add_systems(Startup, move |type_registry: Res<AppTypeRegistry>| {
            let type_registry = type_registry.read();

            let reflect_serializer = bevy::reflect::serde::ReflectSerializer::new(
                &base_asset_map_source,
                &type_registry,
            );

            let serialized = ron::ser::to_string_pretty(
                &reflect_serializer,
                ron::ser::PrettyConfig::new().depth_limit(6),
            )
            .unwrap();

            let file_path = std::path::Path::new("assets/mods/base/derivations/item_assets.ron");
            std::fs::write(&file_path, serialized).unwrap();
        })
        .add_systems(Startup, move |type_registry: Res<AppTypeRegistry>| {
            let type_registry = type_registry.read();

            let reflect_serializer = bevy::reflect::serde::ReflectSerializer::new(
                &tyconic_asset_map_source,
                &type_registry,
            );

            let serialized = ron::ser::to_string_pretty(
                &reflect_serializer,
                ron::ser::PrettyConfig::new().depth_limit(6),
            )
            .unwrap();

            let file_path = std::path::Path::new("assets/mods/tyconic/derivations/item_assets.ron");
            std::fs::write(&file_path, serialized).unwrap();
        });

        app.run();
    }
}
