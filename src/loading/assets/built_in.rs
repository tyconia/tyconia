use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_kira_audio::AudioSource;

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
    #[asset(path = "textures/isometric_inserters.png")]
    pub isometric_inserters: Handle<Image>,
    #[asset(path = "textures/isometric_table.png")]
    pub isometric_table: Handle<Image>,

    #[asset(path = "textures/burger.png")]
    pub burger: Handle<Image>,
    #[asset(path = "textures/pizza.png")]
    pub pizza: Handle<Image>,
    #[asset(path = "textures/fries.png")]
    pub fries: Handle<Image>,
    #[asset(path = "textures/choco.png")]
    pub choco: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct UiAssets {
    #[asset(path = "textures/ui/title.png")]
    pub title: Handle<Image>,

    #[asset(path = "textures/ui/ui_selection.png")]
    pub selection: Handle<Image>,

    #[asset(path = "textures/ui/button.png")]
    pub button: Handle<Image>,
    #[asset(path = "textures/ui/button--active.png")]
    pub button_active: Handle<Image>,
    #[asset(path = "textures/ui/support_me_on_kofi_badge_beige.png")]
    pub kofi_donation_link: Handle<Image>,
    #[asset(path = "textures/ui/support_me_on_kofi_badge_dark.png")]
    pub kofi_donation_link_dark: Handle<Image>,
    #[asset(path = "textures/ui/support_me_on_kofi_badge_red.png")]
    pub kofi_donation_link_red: Handle<Image>,

    // buttons
    #[asset(path = "textures/ui/button_alpha.png")]
    pub button_alpha: Handle<Image>,
    #[asset(path = "textures/ui/button_alpha__active.png")]
    pub button_alpha_active: Handle<Image>,
    #[asset(path = "textures/ui/button_alpha__hover.png")]
    pub button_alpha_hover: Handle<Image>,

    // range sliders
    #[asset(path = "textures/ui/range_slider_thumb.png")]
    pub range_slider_thumb: Handle<Image>,
    #[asset(path = "textures/ui/range_slider_thumb_active.png")]
    pub range_slider_thumb_active: Handle<Image>,
    #[asset(path = "textures/ui/range_slider_thumb_hover.png")]
    pub range_slider_thumb_hover: Handle<Image>,
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
    #[asset(path = "textures/ui/magic_axe_real_ico.png")]
    pub magic_axe_real_ico: Handle<Image>,
    #[asset(path = "textures/ui/interface_ico.png")]
    pub interface_ico: Handle<Image>,

    // log levels
    #[asset(path = "textures/ui/log_level_common_ico.png")]
    pub log_level_common_ico: Handle<Image>,
    #[asset(path = "textures/ui/log_level_info_ico.png")]
    pub log_level_info_ico: Handle<Image>,
    #[asset(path = "textures/ui/log_level_warning_ico.png")]
    pub log_level_warning_ico: Handle<Image>,
    #[asset(path = "textures/ui/log_level_error_ico.png")]
    pub log_level_error_ico: Handle<Image>,
    #[asset(path = "textures/ui/log_level_critical_ico.png")]
    pub log_level_critical_ico: Handle<Image>,

    #[asset(path = "textures/ui/inventory_slot.png")]
    pub inventory_slot: Handle<Image>,
    #[asset(path = "textures/ui/inventory_slot__hover.png")]
    pub inventory_slot_hover: Handle<Image>,
    #[asset(path = "textures/ui/inventory_slot__active.png")]
    pub inventory_slot_active: Handle<Image>,

    #[asset(path = "textures/ui/inventory_backdrop.png")]
    pub inventory_backdrop: Handle<Image>,
}
