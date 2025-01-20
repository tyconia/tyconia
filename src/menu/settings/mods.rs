use super::SettingsTabsState;
use crate::loading::FontAssets;
use bevy::prelude::*;

pub fn setup(mut cmd: Commands, backdrop: super::SettingsBackdropQuery, fonts: Res<FontAssets>) {
    cmd.entity(backdrop.single()).with_children(|parent| {
        parent
            .spawn((
                StateScoped(SettingsTabsState::Mods),
                Node {
                    width: Val::Px(100.),
                    height: Val::Px(100.),
                    align_content: AlignContent::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
            ))
            .with_children(|parent| {
                crate::ui::title_text("Mods coming soon", parent, &fonts);
            });
    });
}
