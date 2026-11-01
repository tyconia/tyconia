use std::path::PathBuf;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::settings::saves_dir;

#[derive(Resource, Default, Clone, Debug)]
pub struct SaveCatalog(pub Vec<SaveEntry>);

impl SaveCatalog {
    pub fn latest(&self) -> Option<&SaveEntry> {
        self.0.first()
    }

    pub fn get(&self, index: usize) -> Option<&SaveEntry> {
        self.0.get(index)
    }
}

#[derive(Clone, Debug)]
pub struct SaveEntry {
    pub path: PathBuf,
    pub metadata: SaveMetadata,
}

#[derive(Resource, Default, Clone, Debug)]
pub struct SelectedSave(pub Option<usize>);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(default)]
pub struct SaveMetadata {
    pub name: String,
    pub world: String,
    pub seed: u64,
    pub playtime_seconds: u64,
    pub last_played: String,
    pub game_version: String,
    pub mod_profile: Vec<String>,
    pub screenshot: Option<PathBuf>,
}

impl Default for SaveMetadata {
    fn default() -> Self {
        Self {
            name: "Unnamed save".into(),
            world: "Tyconic".into(),
            seed: 0,
            playtime_seconds: 0,
            last_played: "Unknown".into(),
            game_version: env!("CARGO_PKG_VERSION").into(),
            mod_profile: Vec::new(),
            screenshot: None,
        }
    }
}

impl SaveMetadata {
    pub fn playtime_label(&self) -> String {
        let hours = self.playtime_seconds / 3600;
        let minutes = (self.playtime_seconds % 3600) / 60;
        format!("{hours}h {minutes}m")
    }
}

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SaveCatalog>()
            .init_resource::<SelectedSave>()
            .add_systems(Startup, load_save_catalog)
            .add_systems(OnEnter(crate::GameState::Menu), load_save_catalog);
    }
}

pub fn load_save_catalog(mut catalog: ResMut<SaveCatalog>) {
    catalog.0 = scan_saves();
}

#[cfg(not(target_arch = "wasm32"))]
fn scan_saves() -> Vec<SaveEntry> {
    let Some(directory) = saves_dir() else {
        return Vec::new();
    };
    let Ok(entries) = std::fs::read_dir(directory) else {
        return Vec::new();
    };

    let mut saves = entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "ron"))
        .filter_map(|entry| {
            let path = entry.path();
            let contents = std::fs::read_to_string(&path).ok()?;
            let metadata = ron::from_str::<SaveMetadata>(&contents).ok()?;
            Some(SaveEntry { path, metadata })
        })
        .collect::<Vec<_>>();

    saves.sort_by(|a, b| b.metadata.last_played.cmp(&a.metadata.last_played));
    saves
}

#[cfg(target_arch = "wasm32")]
fn scan_saves() -> Vec<SaveEntry> {
    Vec::new()
}
