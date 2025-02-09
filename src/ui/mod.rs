//! Contains all stylized UI elements

use bevy::prelude::*;
use bevy::text::FontSmoothing;

mod button;
mod checkbox;
mod dropdown;
mod notification;
mod prompt;
mod range_slider;
//mod selection;
mod system_cursor;
mod tooltip;
mod window;

pub use button::*;
pub use checkbox::*;
pub use notification::*;
pub use range_slider::*;
pub use system_cursor::*;
pub use window::*;

use crate::loading::FontAssets;

pub enum ZIndices {
    Menu = 1,
    Notification = 2,
    Window = 3,
    Prompt = 10,
    Selection = 11,
    Tooltip = 12,
}

impl From<ZIndices> for ZIndex {
    fn from(value: ZIndices) -> Self {
        Self(value as i32)
    }
}

/// default UI_SCALE
pub const UI_SCALE: f32 = 8.;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            button::ButtonPlugin,
            checkbox::CheckboxPlugin,
            range_slider::RangeSliderPlugin,
            window::WindowPlugin,
            notification::NotificationPlugin,
            #[cfg(not(target_arch = "wasm32"))]
            system_cursor::SystemCursorPlugin,
            //selection::SelectionPlugin,
        ));
    }
}

const LIGHT_FONT: TextColor = TextColor(Color::srgba(0.5, 0.5, 0.5, 1.0));
const GREY_1_FONT: TextColor = TextColor(Color::srgba(0.4, 0.4, 0.4, 1.0));
const DARK_FONT: TextColor = TextColor(Color::srgba(0.3, 0.3, 0.3, 1.0));

pub const SMALL_FONT: f32 = 12.;
pub const SMALL_MEDIUM_FONT: f32 = 22.;
pub const BUTTON_FONT: f32 = 24.;
pub const MEDIUM_FONT: f32 = 26.;
pub const LARGE_FONT: f32 = 36.;

pub const BUTTON_IMG_MODE_SLICED: bevy::ui::widget::NodeImageMode =
    bevy::ui::widget::NodeImageMode::Sliced(TextureSlicer {
        border: BorderRect {
            left: 7.,
            right: 7.,
            top: 7.,
            bottom: 7.,
        },
        center_scale_mode: SliceScaleMode::Tile {
            stretch_value: 1.75,
        },
        sides_scale_mode: SliceScaleMode::Tile {
            stretch_value: 1.75,
        },
        max_corner_scale: 3.,
    });

pub const NODE_IMG_MODE_SLICED: bevy::ui::widget::NodeImageMode =
    bevy::ui::widget::NodeImageMode::Sliced(TextureSlicer {
        border: BorderRect {
            left: 5.,
            right: 5.,
            top: 4.,
            bottom: 4.,
        },
        center_scale_mode: SliceScaleMode::Tile { stretch_value: 1.5 },
        sides_scale_mode: SliceScaleMode::Tile {
            stretch_value: 2.75,
        },
        max_corner_scale: 3.,
    });

pub fn build_text<'a, T: ChildBuild, F: FnMut(&'a mut T)>(
    text: &'a str,
    cmd: &'a mut T,
    fonts: &'a Res<FontAssets>,
    f: F,
) {
}

pub fn separator<'a, T: ChildBuild>(cmd: &'a mut T) {
    cmd.spawn(Node {
        width: Val::Percent(100.),
        height: Val::Px(16.),
        ..default()
    });
}

pub fn title_text<'a>(
    text: &'a str,
    cmd: &'a mut ChildBuilder,
    fonts: &'a Res<FontAssets>,
) -> EntityCommands<'a> {
    cmd.spawn((
        DARK_FONT,
        Text::new(text),
        TextFont {
            font: fonts.jersey_25.clone(),
            font_smoothing: FontSmoothing::AntiAliased,
            font_size: LARGE_FONT,
        },
    ))
}

pub fn section_text<'a>(
    text: &'a str,
    cmd: &'a mut ChildBuilder,
    fonts: &'a Res<FontAssets>,
) -> EntityCommands<'a> {
    cmd.spawn((
        GREY_1_FONT,
        Text::new(text),
        TextFont {
            font: fonts.jersey.clone(),
            font_smoothing: FontSmoothing::AntiAliased,
            font_size: SMALL_MEDIUM_FONT,
        },
    ))
}

pub fn body_text<'a>(
    text: &'a str,
    cmd: &'a mut ChildBuilder,
    fonts: &'a Res<FontAssets>,
) -> EntityCommands<'a> {
    cmd.spawn((
        DARK_FONT,
        Text::new(text),
        TextFont {
            font: fonts.jersey_25.clone(),
            font_smoothing: FontSmoothing::AntiAliased,
            font_size: SMALL_MEDIUM_FONT,
        },
    ))
}

#[derive(Debug, Component)]
pub struct Scrollable;
