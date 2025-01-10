use crate::GameState;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_kira_audio::AudioSource;

pub struct LoadingPlugin;

/// This plugin loads all assets using [`AssetLoader`] from a third party bevy plugin
/// Alternatively you can write the logic to load assets yourself
/// If interested, take a look at <https://bevy-cheatbook.github.io/features/assets.html>
impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Menu)
                .load_collection::<AudioAssets>()
                .load_collection::<FontAssets>()
                .load_collection::<UiAssets>()
                .load_collection::<TextureAssets>(),
        );
    }
}

// the following asset collections will be loaded during the State `GameState::Loading`
// when done loading, they will be inserted as resources (see <https://github.com/NiklasEi/bevy_asset_loader>)

#[derive(AssetCollection, Resource)]
pub struct AudioAssets {
    #[asset(path = "audio/flying.ogg")]
    pub flying: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
pub struct FontAssets {
    #[asset(path = "fonts/Jersey10-Regular.ttf")]
    pub jersey: Handle<Font>,
}

#[derive(AssetCollection, Resource)]
pub struct TextureAssets {
    #[asset(path = "textures/art-products.png")]
    pub products: Handle<Image>,
    #[asset(path = "textures/bevy.png")]
    pub bevy: Handle<Image>,
    #[asset(path = "textures/github.png")]
    pub github: Handle<Image>,
    #[asset(path = "textures/infiniteio32bit.png")]
    pub infinite_io: Handle<Image>,
    #[asset(path = "textures/isometric_floors.png")]
    pub isometric_floors: Handle<Image>,
    #[asset(path = "textures/isometric_countertop.png")]
    pub isometric_countertop: Handle<Image>,
    #[asset(path = "textures/isometric_belts.png")]
    pub isometric_belts: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct UiAssets {
    #[asset(path = "textures/ui/title.png")]
    pub title: Handle<Image>,
    #[asset(path = "textures/ui/button.png")]
    pub button: Handle<Image>,
    #[asset(path = "textures/ui/button--active.png")]
    pub button_active: Handle<Image>,
    #[asset(path = "textures/ui/support_me_on_kofi_badge_beige.png")]
    pub kofi_donation_link: Handle<Image>,
    #[asset(path = "textures/ui/support_me_on_kofi_badge_dark.png")]
    pub kofi_donation_link_dark: Handle<Image>,
}

