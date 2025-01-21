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
#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;

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
    #[default]
    Paused,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .enable_state_scoped_entities::<GameState>()
            .add_sub_state::<InGameState>()
            .add_systems(OnEnter(GameState::Quit), (quit,))
            .add_plugins((
                LoadingPlugin,
                MenuPlugin,
                ActionsPlugin,
                InternalAudioPlugin,
                PlayerPlugin,
                levels::ChunkPlugin,
                levels::TransportPlugin,
                ui::UiPlugin,
            ));

        #[cfg(debug_assertions)]
        {
            app.add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()));
        }
    }
}
