#![allow(clippy::type_complexity)]
#![feature(trait_upcasting)]
#![feature(impl_trait_in_bindings)]

pub mod actions;
mod audio;
pub mod hud;
mod levels;
mod loading;
mod menu;
mod player;
pub mod ui;

use crate::actions::ActionsPlugin;
use crate::audio::InternalAudioPlugin;
use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;
use crate::player::PlayerPlugin;

use bevy::app::App;
pub use levels::*;

#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;

#[derive(States, Clone, Eq, PartialEq, Debug, Hash, Copy)]
pub struct DeveloperMode(pub bool);

impl Default for DeveloperMode {
    fn default() -> Self {
        Self(false)
    }
}

// This example game uses States to separate logic
// See https://bevy-cheatbook.github.io/programming/states.html
// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash, Copy)]
pub enum GameState {
    /// Initial asset loading
    #[default]
    Loading,
    /// Gameworld loaded and optionally simulated
    Playing,
    /// Main menu drawn and waiting for interactions
    Menu,
    /// Signal to quit window
    Quit,
}

fn quit(mut event_writer: EventWriter<AppExit>) {
    event_writer.send(AppExit::Success);
}

#[derive(SubStates, Default, Clone, Eq, PartialEq, Debug, Hash)]
#[source(GameState = GameState::Playing) ]
enum InGameState {
    Normal,
    // simulation paused. useful for saving
    #[default]
    Paused,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .init_state::<DeveloperMode>()
            .enable_state_scoped_entities::<GameState>()
            .enable_state_scoped_entities::<DeveloperMode>()
            .add_sub_state::<InGameState>()
            .add_systems(OnEnter(GameState::Quit), (quit,))
            .add_plugins((
                LoadingPlugin,
                MenuPlugin,
                hud::HUDPlugin,
                ActionsPlugin,
                InternalAudioPlugin,
                PlayerPlugin,
                levels::LevelsPlugin,
                ui::UiPlugin,
            ));

        #[cfg(debug_assertions)]
        {
            app.add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()));
        }
    }
}

mod tests {
    #[test]
    fn serialize_ron() {
        fn serialize(type_registry: bevy::prelude::Res<bevy::prelude::AppTypeRegistry>) {
            let item_pack = crate::Pack {
                meta: crate::Meta {
                    mod_name: "Tyconic".to_string(),
                    namespace: crate::Namespace::Vanilla,
                    version: (0, 1, 0).into(),
                },
                items: vec![
                    crate::ItemId("pizza_slice".into()),
                    crate::ItemId("hamburger".into()),
                    //crate::ItemId::from(("Choco", "choco_cup")),
                    //crate::ItemId::from(("Fries", "french_fries")),
                    //crate::ItemId::from(("Potato", "potato_medium")),
                    crate::ItemId("cheese_wheel".into()),
                    crate::ItemId("bread_loaf".into()),
                    crate::ItemId("pork_slab".into()),
                ],
                research: vec![
                    crate::ResearchId("Lemon stand I".into()),
                    crate::ResearchId("Lemon stand II".into()),
                    crate::ResearchId("Diner crash I".into()),
                    crate::ResearchId("Diner crash II".into()),
                    crate::ResearchId("Diner crash III".into()),
                ],
                recipes: vec![
                    crate::RecipeId("presumptive_pizza".into()),
                    crate::RecipeId("folk_pizza".into()),
                    crate::RecipeId("rustic_pizza".into()),
                    crate::RecipeId("standard_pizza".into()),
                    crate::RecipeId("artisanal_pizza".into()),
                    crate::RecipeId("gilded_pizza".into()),
                    crate::RecipeId("presumptive_burger".into()),
                    crate::RecipeId("folk_burger".into()),
                    crate::RecipeId("rustic_burger".into()),
                    crate::RecipeId("standard_burger".into()),
                    crate::RecipeId("artisanal_burger".into()),
                    crate::RecipeId("gilded_burger".into()),
                    crate::RecipeId("presumptive_fries".into()),
                    crate::RecipeId("folk_fries".into()),
                    crate::RecipeId("rustic_fries".into()),
                    crate::RecipeId("standard_fries".into()),
                    crate::RecipeId("artisanal_fries".into()),
                    crate::RecipeId("gilded_fries".into()),
                ],
            };
            let type_registry = type_registry.read();

            let reflect_serializer =
                bevy::reflect::serde::ReflectSerializer::new(&item_pack, &type_registry);

            let serialized = ron::ser::to_string_pretty(
                &reflect_serializer,
                ron::ser::PrettyConfig::new().depth_limit(6),
            )
            .unwrap();

            let file_path = std::path::Path::new("item_pack.ron");
            std::fs::write(&file_path, serialized).unwrap();
        }
        let mut app = bevy::prelude::App::new();
        app.register_type::<super::Pack>();
        app.add_systems(bevy::prelude::Startup, serialize);
        app.run();
    }
}
