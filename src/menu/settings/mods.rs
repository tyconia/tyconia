use super::SettingsTabsState;
use crate::loading::FontAssets;
use bevy::prelude::*;

pub fn setup(mut cmd: Commands, backdrop: super::SettingsBackdropQuery, fonts: Res<FontAssets>) {
    cmd.entity(backdrop.single()).with_children(|parent| {
        parent
            .spawn((
                StateScoped(SettingsTabsState::Mods),
                Node {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                BackgroundColor(Color::WHITE),
            ))
            .with_children(|parent| {
                crate::ui::title_text("Mods coming soon", parent, &fonts);
            });
    });
}
