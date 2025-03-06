use bevy::prelude::*;
//use bevy::utils::HashMap;
use std::{collections::HashMap, fs, path::PathBuf};

use crate::*;

/// Allows an item to be built on the map.
#[derive(Component, Debug, PartialEq, Clone, Reflect, Eq, Hash)]
pub enum BuildingAssetSource {
    // Specifies the asset path inside textures
    // `~/.config/tyconic/mods/mod_name/assets/textures/item_id.png`
    Path(PathBuf),
    // Infer will derive an asset path like
    // `~/.config/tyconic/mods/mod_name/assets/textures/item_id.png`
    Infer,
    // InferItem will derive an asset path from item_assets
    InferItem,
    // InferWithStates will derive an asset path like
    // `~/.config/tyconic/mods/mod_name/assets/textures/item_id.png`
    // including graphic states like `item_id--back-facing.png` and `item_id--broken.png`
    InferWithStates(Vec<String>),
}

#[derive(Component, Reflect)]
pub struct BuildingAssetMapConfig(pub HashMap<String, BuildingAssetSource>);

#[derive(Component, Reflect)]
pub struct BuildingAssetMap(pub HashMap<String, Handle<Image>>);

pub struct BuildingAssetMapPlugin;

impl Plugin for BuildingAssetMapConfig {
    fn build(&self, app: &mut App) {
        app.register_type::<BuildingAssetMapConfig>()
            .register_type::<BuildingAssetMap>()
            //.add_systems(
            //    OnEnter(GameState::Playing),
            //    (init_item_asset_map, load_item_asset_map,).chain().after(load_mods_from_profile),
            //)
            ;
    }
}

//use bevy::reflect::serde::ReflectDeserializer;
//use serde::de::DeserializeSeed;
//use std::io::Read;
//
//pub fn init_item_asset_map(
//    mut cmd: Commands,
//    type_registry: Res<AppTypeRegistry>,
//    level: Query<(Entity, &ModProfile), With<Level>>,
//) {
//    let (level_entity, mod_profile) = level.single();
//    let item_asset_path = PathBuf::from("derivations/item_assets.ron");
//    let type_registry = type_registry.read();
//
//    let item_asset_pairs = mod_profile
//        .0
//        .iter()
//        .map(|(mod_pack, path)| {
//            let item_asset_path = path.join(item_asset_path.clone());
//
//            (mod_pack, item_asset_path, path)
//        })
//        .map(|(mod_pack, item_asset_path, mod_directory)| {
//            fs::File::open(&item_asset_path)
//                .map_or_else(
//                    |err| {
//                        error!(
//                            "Failed to open file {}, at {}, Loaded default configuration instead",
//                            err,
//                            item_asset_path.to_string_lossy()
//                        );
//                        None
//                    },
//                    |mut file| {
//                        let mut ron = String::new();
//                        file.read_to_string(&mut ron).unwrap();
//
//                        let mut deserializer = ron::de::Deserializer::from_str(&ron).unwrap();
//                        let reflect_deserializer = ReflectDeserializer::new(&type_registry);
//                        //
//                        let partial_reflect_value =
//                            reflect_deserializer.deserialize(&mut deserializer).unwrap();
//
//                        let map_config =
//                            ItemAssetMapConfig::from_reflect(&*partial_reflect_value).unwrap();
//
//                        let map_config = map_config
//                            .0
//                            .iter()
//                            .map(|(item_id, asset_source)| {
//                                let new_item_id = format!(
//                                    "::{}::{}::{}",
//                                    mod_pack.meta.namespace, mod_pack.meta.mod_name, item_id
//                                );
//
//                                let asset_source = match asset_source {
//                                    ItemAssetSource::Infer => ItemAssetSource::Path(
//                                        item_asset_path
//                                            .join(mod_directory)
//                                            .join("assets/textures")
//                                            .join(format!("{}.png", item_id))
//                                    ),
//
//                                    ItemAssetSource::Path(path) => {
//                                        ItemAssetSource::Path(path.clone())
//                                    }
//                                    _ => unimplemented!(),
//                                };
//
//                                info!("loading item {} at {:?}", new_item_id, asset_source);
//                                (new_item_id.clone(), asset_source)
//                            })
//                            .collect::<HashMap<String, ItemAssetSource>>();
//
//                        Some(map_config)
//                    },
//                )
//                .unwrap()
//        })
//        .fold(
//            HashMap::new(),
//            |mut acc, map: HashMap<String, ItemAssetSource>| {
//                acc.extend(map);
//                acc
//            },
//        );
//
//    cmd.entity(level_entity)
//        .insert(ItemAssetMapConfig(item_asset_pairs));
//}
//
//pub fn load_item_asset_map(
//    mut cmd: Commands,
//    level: Query<(Entity, &ItemAssetMapConfig), With<Level>>,
//    asset_server: Res<AssetServer>,
//) {
//    let (level_entity, item_asset_config) = level.single();
//
//    let assets = item_asset_config
//        .0
//        .clone()
//        .into_iter()
//        .map(|(id, config)| {
//            let path: PathBuf = match config {
//                ItemAssetSource::Infer => format!("{}.png", id).into(),
//                ItemAssetSource::Path(path) => path,
//                _ => unimplemented!(),
//            };
//
//            let handle = asset_server.load(path);
//
//            info!("{} is loaded", id);
//
//            (id, handle)
//        })
//        .collect();
//
//    cmd.entity(level_entity).insert(ItemAssetMap(assets));
//}
//
//mod tests {
//    use super::*;
//
//    #[test]
//    fn write_asset_map() {
//        let asset_map = [
//            ("auto_arm".into(), ItemAssetSource::Infer),
//            ("mover_belt".into(), ItemAssetSource::Infer),
//            ("infinite_io".into(), ItemAssetSource::Infer),
//        ];
//
//        let map = ItemAssetMapConfig(asset_map.into());
//
//        let mut app = App::new();
//        app.register_type::<ItemAssetMapConfig>();
//        app.add_systems(Startup, move |type_registry: Res<AppTypeRegistry>| {
//            let type_registry = type_registry.read();
//
//            let reflect_serializer =
//                bevy::reflect::serde::ReflectSerializer::new(&map, &type_registry);
//
//            let serialized = ron::ser::to_string_pretty(
//                &reflect_serializer,
//                ron::ser::PrettyConfig::new().depth_limit(6),
//            )
//            .unwrap();
//
//            let file_path = std::path::Path::new("assets/mods/base/derivations/item_assets.ron");
//            std::fs::write(&file_path, serialized).unwrap();
//        });
//        app.run();
//    }
//}
