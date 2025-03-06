pub mod actions;
mod audio;
pub mod hud;
mod levels;
mod loading;
mod menu;
mod player;
mod story;
pub mod ui;

pub use story::*;

mod mods;
pub use mods::*;

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

#[derive(Component)]
pub struct ChangeStates<T: States>(T);

// This example game uses States to separate logic
// See https://bevy-cheatbook.github.io/programming/states.html
// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash, Copy, Event, Reflect)]
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

#[derive(SubStates, Default, Clone, Eq, PartialEq, Debug, Hash, Event)]
#[source(GameState = GameState::Playing) ]
enum InGameState {
    #[default]
    Normal,
    // simulation paused. useful for saving
    Paused,
}

pub fn handle_game_state_events(
    mut next_game_state: ResMut<NextState<GameState>>,
    mut game_state_events: EventReader<GameState>,
) {
    for game_state in game_state_events.read() {
        next_game_state.set(*game_state);
    }
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ModProfilePlugin,))
            .register_type::<ModPack>();


        #[cfg(not(target_arch = "wasm32"))]
        app.add_plugins((ScriptingPlugin,));

        app.init_state::<GameState>()
            .register_type::<State<GameState>>()
            .init_state::<DeveloperMode>()
            .add_event::<GameState>()
            .add_systems(
                Update,
                handle_game_state_events.run_if(on_event::<GameState>),
            )
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

#[derive(Debug, Resource, Reflect, Hash, PartialEq, Eq, Clone)]
pub struct ModPack {
    pub mod_id: Meta,
    pub descriptor: MetaDescriptor,
    pub attributions: MetaAttributions,
}

mod tests {
    #[test]
    fn mod_pack_base_recipes() {
        fn serialize_mod_pack(type_registry: bevy::prelude::Res<bevy::prelude::AppTypeRegistry>) {
            let recipe_pack = crate::RecipePack(vec![
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
            ]);

            let type_registry = type_registry.read();

            let reflect_serializer =
                bevy::reflect::serde::ReflectSerializer::new(&recipe_pack, &type_registry);

            let serialized = ron::ser::to_string_pretty(
                &reflect_serializer,
                ron::ser::PrettyConfig::new().depth_limit(6),
            )
            .unwrap();

            let file_path = std::path::Path::new("assets/mods/tyconic/declarations/recipes.ron");
            std::fs::write(&file_path, serialized).unwrap();
        }
        let mut app = bevy::prelude::App::new();
        app.register_type::<super::ModPack>();
        app.add_systems(bevy::prelude::Startup, serialize_mod_pack);
        app.run();
    }

    #[test]
    fn mod_pack_base_research() {
        fn serialize_mod_pack(type_registry: bevy::prelude::Res<bevy::prelude::AppTypeRegistry>) {
            let research_pack = crate::ResearchPack(vec![
                crate::ResearchId("Lemon stand I".into()),
                crate::ResearchId("Lemon stand II".into()),
                crate::ResearchId("Diner crash I".into()),
                crate::ResearchId("Diner crash II".into()),
                crate::ResearchId("Diner crash III".into()),
            ]);

            let type_registry = type_registry.read();

            let reflect_serializer =
                bevy::reflect::serde::ReflectSerializer::new(&research_pack, &type_registry);

            let serialized = ron::ser::to_string_pretty(
                &reflect_serializer,
                ron::ser::PrettyConfig::new().depth_limit(6),
            )
            .unwrap();

            let file_path = std::path::Path::new("assets/mods/tyconic/declarations/research.ron");
            std::fs::write(&file_path, serialized).unwrap();
        }
        let mut app = bevy::prelude::App::new();
        app.register_type::<super::ModPack>();
        app.add_systems(bevy::prelude::Startup, serialize_mod_pack);
        app.run();
    }

    #[test]
    fn mod_pack_base_items() {
        fn serialize_mod_pack(type_registry: bevy::prelude::Res<bevy::prelude::AppTypeRegistry>) {
            let mod_pack = crate::ItemPack(vec![
                "pizza_slice".into(),
                "hamburger_hand".into(),
                "choco_cup".into(),
                "fries_medium".into(),
                "potato_medium".into(),
                "cheese_wheel".into(),
                "bread_loaf".into(),
                "pork_slab".into(),
            ]);
            let type_registry = type_registry.read();

            let reflect_serializer =
                bevy::reflect::serde::ReflectSerializer::new(&mod_pack, &type_registry);

            let serialized = ron::ser::to_string_pretty(
                &reflect_serializer,
                ron::ser::PrettyConfig::new().depth_limit(6),
            )
            .unwrap();

            let file_path = std::path::Path::new("assets/mods/tyconic/declarations/items.ron");
            std::fs::write(&file_path, serialized).unwrap();
        }
        let mut app = bevy::prelude::App::new();
        app.register_type::<super::ModPack>();
        app.add_systems(bevy::prelude::Startup, serialize_mod_pack);
        app.run();
    }

    #[test]
    fn mod_pack_tyconic() {
        fn serialize_mod_pack(type_registry: bevy::prelude::Res<bevy::prelude::AppTypeRegistry>) {
            let mod_pack = crate::ModPack {
                mod_id: crate::Meta {
                    mod_name: "tyconic".into(),
                    version: (0, 0, 0).into(),
                },
                descriptor: crate::MetaDescriptor {
                    descripion:
                        "adds restaurants amenities and accomodations: NPCs, kitchen buildings etc"
                            .into(),
                    display_name: "Tyconic".into(),
                    thumbnail: None,
                    cover_art: None,
                    dependencies: vec![crate::MetaSource {
                        id: crate::MetaShorthand("base_0.1.0".into()),
                        sources: vec![],
                    }]
                    .into(),
                },
                attributions: crate::MetaAttributions {
                    authors: vec!["qarkdev+gh@gmail.com".into()],
                    licenses: vec!["EULA".into()],
                    credits: vec![],
                },
            };

            let type_registry = type_registry.read();

            let reflect_serializer =
                bevy::reflect::serde::ReflectSerializer::new(&mod_pack, &type_registry);

            let serialized = ron::ser::to_string_pretty(
                &reflect_serializer,
                ron::ser::PrettyConfig::new()
                    .depth_limit(6)
                    .indentor("  ".into()),
            )
            .unwrap();

            let file_path = std::path::Path::new("assets/mods/tyconic/meta.ron");
            std::fs::write(&file_path, serialized).unwrap();
        }
        let mut app = bevy::prelude::App::new();
        app.register_type::<super::ModPack>();
        app.add_systems(bevy::prelude::Startup, serialize_mod_pack);
        app.run();
    }
    #[test]
    fn mod_pack_base() {
        fn serialize_mod_pack(type_registry: bevy::prelude::Res<bevy::prelude::AppTypeRegistry>) {
            let mod_pack = crate::ModPack {
                mod_id: crate::Meta {
                    mod_name: "base".into(),
                    version: (0, 0, 0).into(),
                },
                descriptor: crate::MetaDescriptor {
                    descripion: "adds auto arms, mover belts and the infinite io building".into(),
                    display_name: "Tyconia base".into(),
                    thumbnail: None,
                    cover_art: None,
                    dependencies: vec![].into(),
                },
                attributions: crate::MetaAttributions {
                    authors: vec!["qarkdev+gh@gmail.com".into()],
                    licenses: vec!["EULA".into()],
                    credits: vec![],
                },
            };

            let type_registry = type_registry.read();

            let reflect_serializer =
                bevy::reflect::serde::ReflectSerializer::new(&mod_pack, &type_registry);

            let serialized = ron::ser::to_string_pretty(
                &reflect_serializer,
                ron::ser::PrettyConfig::new().depth_limit(6),
            )
            .unwrap();

            let file_path = std::path::Path::new("assets/mods/base/meta.ron");
            std::fs::write(&file_path, serialized).unwrap();
        }
        let mut app = bevy::prelude::App::new();
        app.register_type::<super::ModPack>();
        app.add_systems(bevy::prelude::Startup, serialize_mod_pack);
        app.run();
    }

    #[test]
    fn serialize_ron() {
        fn serialize(type_registry: bevy::prelude::Res<bevy::prelude::AppTypeRegistry>) {
            let item_pack = crate::Pack {
                description: "".into(),
                meta: crate::Meta {
                    mod_name: "Tyconic".to_string(),
                    version: (0, 1, 0).into(),
                },
                items: vec![
                    "pizza_slice".into(),
                    "hamburger".into(),
                    "french_fries".into(),
                    "potato_medium".into(),
                    "cheese_wheel".into(),
                    "bread_loaf".into(),
                    "beef_slab".into(),
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
