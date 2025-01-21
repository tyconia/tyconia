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
                .load_collection::<FontAssets>()
                .load_collection::<AudioAssets>()
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
    #[asset(path = "fonts/Jersey10Charted-Regular.ttf")]
    pub jersey: Handle<Font>,
    #[asset(path = "fonts/Jersey25-Regular.ttf")]
    pub jersey_25: Handle<Font>,
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
    #[asset(path = "textures/isometric_kitchen_floors.png")]
    pub isometric_kitchen_floors: Handle<Image>,
    #[asset(path = "textures/isometric_countertop.png")]
    pub isometric_countertop: Handle<Image>,
    #[asset(path = "textures/isometric_belts.png")]
    pub isometric_belts: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct UiAssets {
    #[asset(path = "textures/ui/title.png")]
    pub title: Handle<Image>,

    //#[asset(path = "textures/ui/button_square.png")]
    //pub button_square: Handle<Image>,
    //#[asset(path = "textures/ui/button_square_active.png")]
    //pub button_square_active: Handle<Image>,
    #[asset(path = "textures/ui/button.png")]
    pub button: Handle<Image>,
    #[asset(path = "textures/ui/button--active.png")]
    pub button_active: Handle<Image>,
    #[asset(path = "textures/ui/support_me_on_kofi_badge_beige.png")]
    pub kofi_donation_link: Handle<Image>,
    #[asset(path = "textures/ui/support_me_on_kofi_badge_dark.png")]
    pub kofi_donation_link_dark: Handle<Image>,

    // buttons
    #[asset(path = "textures/ui/button_alpha.png")]
    pub button_alpha: Handle<Image>,
    #[asset(path = "textures/ui/button_alpha__active.png")]
    pub button_alpha_active: Handle<Image>,

    // range sliders
    #[asset(path = "textures/ui/range_slider_thumb.png")]
    pub range_slider_thumb: Handle<Image>,
    #[asset(path = "textures/ui/range_slider_thumb_active.png")]
    pub range_slider_thumb_active: Handle<Image>,
    #[asset(path = "textures/ui/range_slider_track.png")]
    pub range_slider_track: Handle<Image>,

    // icons
    #[asset(path = "textures/ui/check.png")]
    pub check: Handle<Image>,
    #[asset(path = "textures/ui/cross.png")]
    pub cross: Handle<Image>,

    #[asset(path = "textures/ui/back_ico.png")]
    pub back_ico: Handle<Image>,
    #[asset(path = "textures/ui/undo_ico.png")]
    pub undo_ico: Handle<Image>,

    // windows
    #[asset(path = "textures/ui/close_ico.png")]
    pub close_ico: Handle<Image>,
    #[asset(path = "textures/ui/close_active_ico.png")]
    pub close_active_ico: Handle<Image>,
    #[asset(path = "textures/ui/window_bar.png")]
    pub window_bar: Handle<Image>,
    #[asset(path = "textures/ui/window_content.png")]
    pub window_content: Handle<Image>,

    // settings icons
    #[asset(path = "textures/ui/monitor_ico.png")]
    pub monitor_ico: Handle<Image>,
    #[asset(path = "textures/ui/joystick_ico.png")]
    pub joystick_ico: Handle<Image>,
    #[asset(path = "textures/ui/speaker_ico.png")]
    pub speaker_ico: Handle<Image>,
    #[asset(path = "textures/ui/earth_ico.png")]
    pub earth_ico: Handle<Image>,
    #[asset(path = "textures/ui/magic_axe_ico.png")]
    pub magic_axe_ico: Handle<Image>,
    #[asset(path = "textures/ui/interface_ico.png")]
    pub interface_ico: Handle<Image>,
}
