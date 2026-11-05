use crate::GameState;
use crate::{
    ColliderDeclared, Level, LevelIsometric, LevelManager, ObstacleCollider, SceneDeclared,
};
use bevy::prelude::*;
use bevy::reflect::serde::{ReflectDeserializer, ReflectSerializer};
use serde::de::DeserializeSeed;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct LevelLoaderPlugin;

impl Plugin for LevelLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<LevelIsometric>()
            .register_type::<SceneDeclared>()
            .register_type::<ColliderDeclared>()
            .init_resource::<LevelPrefabCatalog>()
            .init_resource::<LevelRequests>()
            .init_resource::<CurrentLevel>()
            .init_resource::<PlayerSpawnRequest>()
            .add_systems(Startup, load_level_prefab_catalog)
            .add_systems(
                Update,
                apply_level_requests.run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnExit(GameState::Playing), clear_level_requests);
    }
}

#[derive(Resource, Default)]
pub struct LevelPrefabCatalog {
    pub loaded: bool,
    pub prefabs: HashMap<String, LevelPrefab>,
}

#[derive(Clone)]
pub struct LevelPrefab {
    pub path: PathBuf,
    pub level: LevelIsometric,
}

#[derive(Resource, Default)]
pub struct LevelRequests(pub Vec<LevelRequest>);

#[derive(Debug, Clone)]
pub enum LevelRequest {
    Load { level_label: String },
    Unload { level_label: String },
    Save { level_label: String },
    Delete { level_label: String },
    Regenerate { level_label: String },
}

#[derive(Resource, Default)]
pub struct CurrentLevel(pub Option<String>);

#[derive(Resource, Default)]
pub struct PlayerSpawnRequest(pub Option<Vec2>);

#[derive(Component)]
pub struct LevelInstance {
    pub prefab: String,
}

#[derive(Component)]
pub struct LevelScene;

fn clear_level_requests(
    mut requests: ResMut<LevelRequests>,
    mut current: ResMut<CurrentLevel>,
    mut spawn_request: ResMut<PlayerSpawnRequest>,
) {
    requests.0.clear();
    current.0 = None;
    spawn_request.0 = None;
}

fn load_level_prefab_catalog(
    mut catalog: ResMut<LevelPrefabCatalog>,
    type_registry: Res<AppTypeRegistry>,
) {
    if catalog.loaded {
        return;
    }
    catalog.loaded = true;

    let mods_dir = Path::new("assets/mods");
    let Ok(mod_entries) = fs::read_dir(mods_dir) else {
        warn!("LevelLoader: no {} directory found", mods_dir.display());
        return;
    };

    let registry = type_registry.read();
    for mod_entry in mod_entries.flatten() {
        let levels_dir = mod_entry.path().join("assets").join("levels");
        let mut level_files = Vec::new();
        collect_ron_files(&levels_dir, &mut level_files);

        for path in level_files {
            let Ok(content) = fs::read_to_string(&path) else {
                warn!("LevelLoader: failed to read {}", path.display());
                continue;
            };
            if !content.contains("tyconia::levels::LevelIsometric") {
                continue;
            }
            let Ok(mut deserializer) = ron::de::Deserializer::from_str(&content) else {
                warn!("LevelLoader: invalid RON at {}", path.display());
                continue;
            };
            let reflect_deserializer = ReflectDeserializer::new(&registry);
            let reflected = match reflect_deserializer.deserialize(&mut deserializer) {
                Ok(reflected) => reflected,
                Err(err) => {
                    warn!(
                        "LevelLoader: failed to deserialize {}: {}",
                        path.display(),
                        err
                    );
                    continue;
                }
            };
            let Some(level) = LevelIsometric::from_reflect(&*reflected) else {
                continue;
            };

            if level.label.is_empty() {
                warn!(
                    "LevelLoader: skipping unlabeled level at {}",
                    path.display()
                );
                continue;
            }

            info!(
                "LevelLoader: registered prefab '{}' from {}",
                level.label,
                path.display()
            );
            catalog
                .prefabs
                .insert(level.label.clone(), LevelPrefab { path, level });
        }
    }
}

fn collect_ron_files(dir: &Path, output: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_ron_files(&path, output);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("ron") {
            output.push(path);
        }
    }
}

fn apply_level_requests(
    mut commands: Commands,
    mut requests: ResMut<LevelRequests>,
    mut catalog: ResMut<LevelPrefabCatalog>,
    mut current: ResMut<CurrentLevel>,
    mut spawn_request: ResMut<PlayerSpawnRequest>,
    asset_server: Res<AssetServer>,
    type_registry: Res<AppTypeRegistry>,
    mut level: Query<(Entity, &mut LevelManager), With<Level>>,
    instances_query: Query<(Entity, &LevelInstance)>,
) {
    let Ok((level_entity, mut manager)) = level.get_single_mut() else {
        return;
    };

    let pending = std::mem::take(&mut requests.0);
    let mut instances: HashMap<String, Entity> = instances_query
        .iter()
        .map(|(entity, instance)| (instance.prefab.clone(), entity))
        .collect();

    for request in pending {
        match request {
            LevelRequest::Load { level_label } => {
                let Some(prefab) = catalog.prefabs.get(&level_label).cloned() else {
                    warn!("LevelLoader: requested unknown prefab '{}'", level_label);
                    continue;
                };
                load_prefab(
                    &mut commands,
                    &asset_server,
                    level_entity,
                    &level_label,
                    &prefab,
                    &mut instances,
                    &mut manager,
                    &mut current,
                    &mut spawn_request,
                );
            }
            LevelRequest::Unload { level_label } => {
                unload_instance(
                    &mut commands,
                    &level_label,
                    &mut instances,
                    &mut manager,
                    &mut current,
                );
            }
            LevelRequest::Save { level_label } => {
                save_prefab(&mut catalog, &manager, &level_label, &type_registry);
            }
            LevelRequest::Delete { level_label } => {
                unload_instance(
                    &mut commands,
                    &level_label,
                    &mut instances,
                    &mut manager,
                    &mut current,
                );
                catalog.prefabs.remove(&level_label);
                info!(
                    "LevelLoader: removed prefab '{}' from the catalog (asset file preserved)",
                    level_label
                );
            }
            LevelRequest::Regenerate { level_label } => {
                let Some(path) = catalog
                    .prefabs
                    .get(&level_label)
                    .map(|prefab| prefab.path.clone())
                else {
                    warn!(
                        "LevelLoader: cannot regenerate unknown prefab '{}'",
                        level_label
                    );
                    continue;
                };
                let level = manager.prefabs.get(&level_label).cloned().or_else(|| {
                    catalog
                        .prefabs
                        .get(&level_label)
                        .map(|prefab| prefab.level.clone())
                });
                unload_instance(
                    &mut commands,
                    &level_label,
                    &mut instances,
                    &mut manager,
                    &mut current,
                );
                let Some(level) = level else {
                    warn!("LevelLoader: no level data for '{}'", level_label);
                    continue;
                };
                let prefab = LevelPrefab { path, level };
                load_prefab(
                    &mut commands,
                    &asset_server,
                    level_entity,
                    &level_label,
                    &prefab,
                    &mut instances,
                    &mut manager,
                    &mut current,
                    &mut spawn_request,
                );
            }
        }
    }
}

fn load_prefab(
    commands: &mut Commands,
    asset_server: &AssetServer,
    level_entity: Entity,
    level_label: &str,
    prefab: &LevelPrefab,
    instances: &mut HashMap<String, Entity>,
    manager: &mut LevelManager,
    current: &mut CurrentLevel,
    spawn_request: &mut PlayerSpawnRequest,
) {
    if instances.contains_key(level_label) {
        info!("LevelLoader: prefab '{}' is already loaded", level_label);
        return;
    }

    let instance_entity = commands
        .spawn((
            LevelInstance {
                prefab: level_label.to_string(),
            },
            Transform::default(),
        ))
        .id();
    commands.entity(level_entity).add_child(instance_entity);

    for scene in prefab.level.scenes.iter() {
        let image = asset_server.load(scene.asset.clone());
        commands.entity(instance_entity).with_children(|parent| {
            parent.spawn((
                LevelScene,
                Sprite { image, ..default() },
                Transform::from_xyz(scene.position.x, scene.position.y, scene.z)
                    .with_scale(Vec3::splat(scene.scale)),
            ));
        });
    }

    for collider in prefab.level.colliders.iter() {
        let obstacle = if collider.screen_space {
            ObstacleCollider::rect(collider.half_extents)
        } else {
            ObstacleCollider::iso(collider.half_extents)
        };

        commands.entity(instance_entity).with_children(|parent| {
            parent.spawn((
                obstacle,
                Transform::from_xyz(collider.position.x, collider.position.y, 0.0).with_rotation(
                    Quat::from_rotation_z(collider.rotation_degrees.to_radians()),
                ),
            ));
        });
    }

    info!(
        "LevelLoader: loaded prefab '{}' from {} ({} scenes, {} colliders)",
        level_label,
        prefab.path.display(),
        prefab.level.scenes.len(),
        prefab.level.colliders.len()
    );

    manager
        .prefabs
        .insert(level_label.to_string(), prefab.level.clone());
    instances.insert(level_label.to_string(), instance_entity);
    current.0 = Some(level_label.to_string());
    spawn_request.0 = Some(prefab.level.spawn);
}

fn unload_instance(
    commands: &mut Commands,
    level_label: &str,
    instances: &mut HashMap<String, Entity>,
    manager: &mut LevelManager,
    current: &mut CurrentLevel,
) {
    let Some(instance_entity) = instances.remove(level_label) else {
        warn!("LevelLoader: cannot unload unknown level '{}'", level_label);
        return;
    };

    commands.entity(instance_entity).despawn_recursive();
    manager.prefabs.remove(level_label);
    if current.0.as_deref() == Some(level_label) {
        current.0 = None;
    }
    info!("LevelLoader: unloaded '{}'", level_label);
}

fn save_prefab(
    catalog: &mut LevelPrefabCatalog,
    manager: &LevelManager,
    level_label: &str,
    type_registry: &AppTypeRegistry,
) {
    let Some(prefab) = catalog.prefabs.get(level_label) else {
        warn!("LevelLoader: cannot save unknown prefab '{}'", level_label);
        return;
    };
    let path = prefab.path.clone();
    let level = manager
        .prefabs
        .get(level_label)
        .cloned()
        .unwrap_or_else(|| prefab.level.clone());

    let registry = type_registry.read();
    let reflect_serializer = ReflectSerializer::new(&level, &registry);
    let serialized = match ron::ser::to_string_pretty(
        &reflect_serializer,
        ron::ser::PrettyConfig::new()
            .depth_limit(8)
            .indentor("  ".into()),
    ) {
        Ok(serialized) => serialized,
        Err(err) => {
            warn!(
                "LevelLoader: failed to serialize '{}': {}",
                level_label, err
            );
            return;
        }
    };
    let serialized = super::post_processing::level(&serialized);

    match fs::write(&path, serialized) {
        Ok(()) => {
            info!(
                "LevelLoader: saved prefab '{}' to {}",
                level_label,
                path.display()
            );
            if let Some(prefab) = catalog.prefabs.get_mut(level_label) {
                prefab.level = level;
            }
        }
        Err(err) => warn!(
            "LevelLoader: failed to save '{}' to {}: {}",
            level_label,
            path.display(),
            err
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_loads_levels_and_scenes() {
        let mut app = App::new();
        app.register_type::<LevelIsometric>()
            .register_type::<SceneDeclared>()
            .register_type::<ColliderDeclared>()
            .init_resource::<LevelPrefabCatalog>()
            .add_systems(Startup, load_level_prefab_catalog);
        app.update();

        let catalog = app.world().resource::<LevelPrefabCatalog>();
        let prefab = catalog
            .prefabs
            .get("prologue_restaurant_0")
            .expect("prologue_restaurant_0 should be registered");

        assert!(!prefab.level.scenes.is_empty());
        assert!(!prefab.level.colliders.is_empty());
        assert!(prefab
            .level
            .scenes
            .iter()
            .all(|scene| !scene.asset.contains("burger")));
    }
}

#[cfg(test)]
mod load_tests {
    use super::*;
    use bevy::asset::AssetPlugin;
    use bevy::transform::TransformPlugin;

    fn setup() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default(), TransformPlugin))
            .register_type::<LevelIsometric>()
            .register_type::<SceneDeclared>()
            .register_type::<ColliderDeclared>()
            .init_asset::<Image>()
            .init_resource::<LevelPrefabCatalog>()
            .init_resource::<LevelRequests>()
            .init_resource::<CurrentLevel>()
            .init_resource::<PlayerSpawnRequest>()
            .add_systems(Startup, load_level_prefab_catalog)
            .add_systems(Update, apply_level_requests);

        app.world_mut().spawn((
            Level::default(),
            LevelManager::default(),
            Transform::default(),
            GlobalTransform::default(),
        ));
        app
    }

    fn request(app: &mut App, request: LevelRequest) {
        app.world_mut()
            .resource_mut::<LevelRequests>()
            .0
            .push(request);
        app.update();
    }

    fn instance_count(app: &mut App) -> usize {
        app.world_mut()
            .query_filtered::<Entity, With<LevelInstance>>()
            .iter(app.world())
            .count()
    }

    fn collider_count(app: &mut App) -> usize {
        app.world_mut()
            .query_filtered::<Entity, With<ObstacleCollider>>()
            .iter(app.world())
            .count()
    }

    #[test]
    fn save_prefab_writes_manager_edits_and_refreshes_catalog() {
        let mut app = setup();
        app.update();
        let source = app
            .world()
            .resource::<LevelPrefabCatalog>()
            .prefabs
            .get("prologue_restaurant_0")
            .expect("prologue_restaurant_0 should be registered")
            .level
            .clone();

        let path = std::env::temp_dir().join(format!(
            "tyconia-prefab-save-test-{}.ron",
            std::process::id()
        ));
        let added = ColliderDeclared {
            position: Vec2::new(321.0, -123.0),
            half_extents: Vec2::new(24.0, 12.0),
            rotation_degrees: 0.0,
            screen_space: false,
        };

        let mut saved_level = source.clone();
        saved_level.colliders.push(added.clone());

        let mut catalog = LevelPrefabCatalog::default();
        catalog.prefabs.insert(
            "save_test".into(),
            LevelPrefab {
                path: path.clone(),
                level: source.clone(),
            },
        );
        let mut manager = LevelManager::default();
        manager.prefabs.insert("save_test".into(), saved_level);

        let registry = app.world().resource::<AppTypeRegistry>();
        save_prefab(&mut catalog, &manager, "save_test", registry);

        let written = std::fs::read_to_string(&path).expect("saved prefab should exist");
        assert!(written.contains("321.0"));
        assert!(written.contains("-123.0"));
        let saved_collider = catalog.prefabs["save_test"]
            .level
            .colliders
            .last()
            .expect("catalog should include the saved collider");
        assert_eq!(saved_collider.position, added.position);
        assert_eq!(saved_collider.half_extents, added.half_extents);

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn regenerate_uses_edited_manager_prefab() {
        let mut app = setup();

        request(
            &mut app,
            LevelRequest::Load {
                level_label: "prologue_restaurant_0".into(),
            },
        );
        let baseline = collider_count(&mut app);

        {
            let mut query = app
                .world_mut()
                .query_filtered::<&mut LevelManager, With<Level>>();
            let mut manager = query.single_mut(app.world_mut());
            manager
                .prefabs
                .get_mut("prologue_restaurant_0")
                .expect("level should be in manager")
                .colliders
                .push(ColliderDeclared {
                    position: Vec2::new(123.0, -45.0),
                    half_extents: Vec2::new(20.0, 10.0),
                    rotation_degrees: 0.0,
                    screen_space: false,
                });
        }

        request(
            &mut app,
            LevelRequest::Regenerate {
                level_label: "prologue_restaurant_0".into(),
            },
        );

        assert_eq!(collider_count(&mut app), baseline + 1);
        assert!(app
            .world_mut()
            .query::<(&GlobalTransform, &ObstacleCollider)>()
            .iter(app.world())
            .any(|(transform, _)| {
                transform
                    .translation()
                    .xy()
                    .distance(Vec2::new(123.0, -45.0))
                    < 0.01
            }));
    }

    #[test]
    fn load_unload_regenerate_and_delete_level_instance() {
        let mut app = setup();

        request(
            &mut app,
            LevelRequest::Load {
                level_label: "prologue_restaurant_0".into(),
            },
        );
        assert_eq!(instance_count(&mut app), 1);
        assert!(collider_count(&mut app) > 0);
        assert!(app.world().resource::<PlayerSpawnRequest>().0.is_some());

        app.update();
        let collider_positions: Vec<Vec2> = app
            .world_mut()
            .query::<(&GlobalTransform, &ObstacleCollider)>()
            .iter(app.world())
            .map(|(transform, _)| transform.translation().xy())
            .collect();
        assert!(
            collider_positions
                .iter()
                .any(|position| position.distance(Vec2::new(-480.0, -30.0)) < 0.01),
            "loaded colliders did not inherit level transforms: {collider_positions:?}"
        );

        request(
            &mut app,
            LevelRequest::Unload {
                level_label: "prologue_restaurant_0".into(),
            },
        );
        assert_eq!(instance_count(&mut app), 0);

        request(
            &mut app,
            LevelRequest::Regenerate {
                level_label: "prologue_restaurant_0".into(),
            },
        );
        assert_eq!(instance_count(&mut app), 1);

        request(
            &mut app,
            LevelRequest::Delete {
                level_label: "prologue_restaurant_0".into(),
            },
        );
        assert_eq!(instance_count(&mut app), 0);
        assert!(!app
            .world()
            .resource::<LevelPrefabCatalog>()
            .prefabs
            .contains_key("prologue_restaurant_0"));
    }
}
