use crate::GameState;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
//use iyes_progress::prelude::*;

mod assets;

pub use assets::*;

pub struct LoadingPlugin;

/// This plugin loads all assets using [`AssetLoader`] from a third party bevy plugin
/// Alternatively you can write the logic to load assets yourself
/// If interested, take a look at <https://bevy-cheatbook.github.io/features/assets.html>
impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        //#[cfg(not(target_arch = "wasm32"))]
        //app.add_plugins((ItemTextureMapPlugin,));

        app.add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Menu)
                .load_collection::<FontAssets>()
                .load_collection::<AudioAssets>()
                .load_collection::<UiAssets>()
                .load_collection::<TextureAssets>(),
        );
    }
}

//pub fn report_progress(progress_tracker: ProgressEntry<GameState>) {
//    info!("progress is {:?}", progress_tracker.get_progress());
//}

// the following asset collections will be loaded during the State `GameState::Loading`
// when done loading, they will be inserted as resources (see <https://github.com/NiklasEi/bevy_asset_loader>)
