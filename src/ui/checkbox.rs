use crate::loading::*;
use crate::GameState;
use bevy::prelude::*;
use bevy::text::FontSmoothing;

use bevy::prelude::*;

/// Represents checkbox states
pub enum CheckboxState {
    Active,
    Inactive,
    Disabled,
}

/// Textured checkbox.
///
/// # Arguments
///
/// * `commands` - commands used to spawn the button
///
/// # Usage
/// just use it bro
pub fn spawn_checkbox<T: ChildBuild>(
    components: impl Bundle,
    cmd: &mut T,
    fonts: &Res<FontAssets>,
    ui: &Res<UiAssets>,
) {
    cmd.spawn(Node {
        height: Val::Px(super::UI_SCALE * 4.),
        aspect_ratio: Some(1.),
        ..default()
    })
    .with_children(|parent| {
        super::spawn_button(
            super::ButtonType::Icon {
                image: ui.cross.clone(),
                image_size: Val::Px(super::UI_SCALE * 4.),
            },
            components,
            parent,
            fonts,
            ui,
        );
    });
}
