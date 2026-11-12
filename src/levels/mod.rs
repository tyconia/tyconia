mod collision;
mod config;
mod declared;
mod editor;
mod geometry_visuals;
mod isometric;
mod level_loader;
mod logistics;
mod mini_game;
mod pack;
pub mod post_processing;
mod vertical;

use crate::{DeveloperMode, GameState};
use bevy::prelude::*;
use bevy::utils::HashMap;
use std::time::*;

pub use collision::*;
pub use config::*;
pub use declared::*;
pub use editor::*;
pub use geometry_visuals::*;
pub use isometric::*;
pub use level_loader::*;
pub use logistics::*;
pub use mini_game::*;
pub use pack::*;
pub use vertical::*;

pub struct LevelsPlugin;

impl Plugin for LevelsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_level)
            .add_plugins((
                LevelLoaderPlugin,
                LevelGeometryVisualsPlugin,
                LevelEditorPlugin,
                TransportPlugin,
                ResearchEditorPlugin,
            ));

        #[cfg(debug_assertions)]
        app.add_systems(
            Update,
            draw_obstacle_collider_gizmos
                .run_if(in_state(GameState::Playing))
                .run_if(in_state(DeveloperMode(true))),
        );

        #[cfg(not(target_arch = "wasm32"))]
        app.register_type::<LevelManager>()
            .register_type::<ItemId>();
    }
}

#[derive(Component, Reflect, Default)]
pub struct LevelManager {
    pub prefabs: HashMap<String, LevelIsometric>,
}

#[derive(Component, Reflect)]
pub struct Level {
    //pub created_at: Instant,
    pub total_play_time: Duration,
}

#[derive(Component)]
pub enum LevelLoadStatus {
    AssetsLoading,
    AssetsLoaded,
}

impl Default for Level {
    fn default() -> Self {
        Self {
            //created_at: Instant::now(),
            total_play_time: Duration::from_secs(1),
        }
    }
}

pub fn spawn_level(mut cmd: Commands) {
    cmd.spawn((
        Level::default(),
        LevelManager::default(),
        LevelLoadStatus::AssetsLoading,
        Transform::default(),
        GlobalTransform::default(),
        Visibility::default(),
        StateScoped(GameState::Playing),
    ));
}

#[cfg(test)]
mod tests;
