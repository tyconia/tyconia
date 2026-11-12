use bevy::prelude::*;
use serde::de::DeserializeSeed;
use std::fs;
use std::path::PathBuf;

mod runtime;
mod schema;

pub use runtime::*;
pub use schema::*;

#[derive(Resource, Default)]
pub struct StoryLoader {
    pub loaded: bool,
}

pub struct StoryPlugin;

impl Plugin for StoryPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ScriptDeclared>()
            .register_type::<ScriptAction>()
            .register_type::<Conversation>()
            .register_type::<ScenarioDeclared>()
            .init_resource::<HeadlessStoryReviewer>()
            .init_resource::<StoryLoader>()
            .init_resource::<StoryBeatRegistry>()
            .init_resource::<crate::levels::LevelRequests>()
            .add_systems(Startup, load_stories_temp)
            .add_systems(Update, (advance_headless_reviewer, resolve_story_jumps));
    }
}

fn load_stories_temp(
    mut reviewer: ResMut<HeadlessStoryReviewer>,
    mut loader: ResMut<StoryLoader>,
    type_registry: Res<AppTypeRegistry>,
) {
    if loader.loaded {
        return;
    }
    loader.loaded = true;

    // Try to load all .ron story files from assets/mods/*/stories/
    let mods_base = PathBuf::from("assets/mods");
    if !mods_base.exists() {
        info!("StoryLoader: no assets/mods directory found");
        return;
    }

    if let Ok(entries) = fs::read_dir(&mods_base) {
        for entry in entries.flatten() {
            let stories_dir = entry.path().join("stories");
            if !stories_dir.is_dir() {
                continue;
            }
            if let Ok(story_entries) = fs::read_dir(&stories_dir) {
                for story_entry in story_entries.flatten() {
                    let path = story_entry.path();
                    if path.extension().and_then(|s| s.to_str()) != Some("ron") {
                        continue;
                    }
                    info!("StoryLoader: loading story from {:?}", path);
                    if let Ok(content) = fs::read_to_string(&path) {
                        let registry = type_registry.read();
                        // Try to deserialize as Bevy Reflect RON
                        if let Ok(mut deserializer) = ron::de::Deserializer::from_str(&content) {
                            let reflect_deserializer =
                                bevy::reflect::serde::ReflectDeserializer::new(&registry);
                            match reflect_deserializer.deserialize(&mut deserializer) {
                                Ok(reflected) => {
                                    // Try to reflect into ScriptDeclared
                                    if let Some(script) = ScriptDeclared::from_reflect(&*reflected)
                                    {
                                        reviewer.active_scripts.push(ActiveScript {
                                            label: script.label.clone(),
                                            current_action_index: 0,
                                            status: ScriptStatus::Playing,
                                            actions: script.actions.clone(),
                                            elapsed_in_action: 0.0,
                                            step_count: 0,
                                            last_choice: None,
                                        });
                                        info!("StoryLoader: loaded script '{}'", script.label);
                                    }
                                }
                                Err(e) => {
                                    warn!("StoryLoader: failed to deserialize {:?}: {}", path, e);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
