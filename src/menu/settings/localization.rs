use super::SettingsTabsState;
use crate::loading::{FontAssets, UiAssets};
use crate::settings::{Language, SettingsConfig};
use crate::ui::*;
use bevy::prelude::*;

pub fn setup(
    mut cmd: Commands,
    backdrop: super::SettingsBackdropQuery,
    fonts: Res<FontAssets>,
    ui: Res<UiAssets>,
    settings: Res<SettingsConfig>,
) {
    let Ok(backdrop) = backdrop.get_single() else {
        return;
    };

    let options = Language::ALL
        .iter()
        .map(|language| language.label().to_string())
        .collect::<Vec<_>>();
    let selected = Language::ALL
        .iter()
        .position(|language| *language == settings.language);

    cmd.entity(backdrop).with_children(|parent| {
        parent
            .spawn((
                StateScoped(SettingsTabsState::GeneralLanguage),
                Node {
                    flex_direction: FlexDirection::Column,
                    width: Val::Percent(100.),
                    row_gap: Val::Px(UI_SCALE),
                    ..default()
                },
            ))
            .with_children(|parent| {
                section_text("Language", parent, &fonts);
                body_text(
                    "The selected language is applied the next time the menu is opened.",
                    parent,
                    &fonts,
                );
                parent
                    .spawn(Node {
                        width: Val::Percent(100.),
                        height: Val::Px(UI_SCALE * 6.),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|parent| {
                        body_text("Menu language", parent, &fonts);
                        parent
                            .spawn(Node {
                                width: Val::Px(260.),
                                height: Val::Px(UI_SCALE * 5.),
                                ..default()
                            })
                            .with_children(|parent| {
                                spawn_dropdown(
                                    parent,
                                    &ui,
                                    &fonts,
                                    &options,
                                    "Select language".into(),
                                    selected,
                                    LanguageDropdown,
                                );
                            });
                    });
            });
    });
}

#[derive(Debug, Component)]
pub struct LanguageDropdown;

pub fn configure_localization(
    mut settings: ResMut<SettingsConfig>,
    dropdowns: Query<&Dropdown, (With<LanguageDropdown>, Changed<Dropdown>)>,
) {
    for dropdown in dropdowns.iter() {
        if let Some(language) = dropdown
            .selected
            .and_then(|index| Language::ALL.get(index).copied())
        {
            if settings.language != language {
                settings.language = language;
            }
        }
    }
}
