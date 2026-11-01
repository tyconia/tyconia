use super::SettingsTabsState;
use crate::loading::{FontAssets, UiAssets};
use crate::settings::{LogLevelSetting, SettingsConfig};
use crate::ui::*;
use bevy::prelude::*;

#[derive(Debug, Component)]
pub struct ResetAllSettings;

#[derive(Debug, Component)]
pub struct LogLevelDropdown;

pub fn setup_developer(
    mut cmd: Commands,
    backdrop: super::SettingsBackdropQuery,
    fonts: Res<FontAssets>,
) {
    let Ok(backdrop) = backdrop.get_single() else {
        return;
    };

    cmd.entity(backdrop).with_children(|parent| {
        parent
            .spawn((
                StateScoped(SettingsTabsState::AdvancedDeveloper),
                Node {
                    flex_direction: FlexDirection::Column,
                    width: Val::Percent(100.),
                    row_gap: Val::Px(UI_SCALE),
                    ..default()
                },
            ))
            .with_children(|parent| {
                section_text("Developer", parent, &fonts);
                body_text(
                    "Enable developer mode by clicking the Mods tab 5 times.",
                    parent,
                    &fonts,
                );
                body_text(
                    "Developer mode unlocks editor tools and Lua scripting.",
                    parent,
                    &fonts,
                );
            });
    });
}

pub fn setup_logging(
    mut cmd: Commands,
    backdrop: super::SettingsBackdropQuery,
    fonts: Res<FontAssets>,
    ui: Res<UiAssets>,
    settings: Res<SettingsConfig>,
) {
    let Ok(backdrop) = backdrop.get_single() else {
        return;
    };

    let options = LogLevelSetting::ALL
        .iter()
        .map(|level| level.label().to_string())
        .collect::<Vec<_>>();
    let selected = LogLevelSetting::ALL
        .iter()
        .position(|level| *level == settings.developer.log_level);

    cmd.entity(backdrop).with_children(|parent| {
        parent
            .spawn((
                StateScoped(SettingsTabsState::AdvancedLogging),
                Node {
                    flex_direction: FlexDirection::Column,
                    width: Val::Percent(100.),
                    row_gap: Val::Px(UI_SCALE),
                    ..default()
                },
            ))
            .with_children(|parent| {
                section_text("Logging", parent, &fonts);
                body_text(
                    "Log level changes are stored for the next launch.",
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
                        body_text("Log level", parent, &fonts);
                        parent
                            .spawn(Node {
                                width: Val::Px(220.),
                                height: Val::Px(UI_SCALE * 5.),
                                ..default()
                            })
                            .with_children(|parent| {
                                spawn_dropdown(
                                    parent,
                                    &ui,
                                    &fonts,
                                    &options,
                                    "Select level".into(),
                                    selected,
                                    LogLevelDropdown,
                                );
                            });
                    });
            });
    });
}

pub fn configure_logging(
    mut settings: ResMut<SettingsConfig>,
    dropdowns: Query<&Dropdown, (With<LogLevelDropdown>, Changed<Dropdown>)>,
) {
    for dropdown in dropdowns.iter() {
        if let Some(level) = dropdown
            .selected
            .and_then(|index| LogLevelSetting::ALL.get(index).copied())
        {
            if settings.developer.log_level != level {
                settings.developer.log_level = level;
            }
        }
    }
}

pub fn setup_reset(
    mut cmd: Commands,
    backdrop: super::SettingsBackdropQuery,
    fonts: Res<FontAssets>,
    ui: Res<UiAssets>,
) {
    let Ok(backdrop) = backdrop.get_single() else {
        return;
    };

    cmd.entity(backdrop).with_children(|parent| {
        parent
            .spawn((
                StateScoped(SettingsTabsState::AdvancedReset),
                Node {
                    flex_direction: FlexDirection::Column,
                    width: Val::Percent(100.),
                    row_gap: Val::Px(UI_SCALE),
                    ..default()
                },
            ))
            .with_children(|parent| {
                section_text("Reset Settings", parent, &fonts);
                body_text(
                    "Restore every setting to its default value.",
                    parent,
                    &fonts,
                );
                parent
                    .spawn(Node {
                        width: Val::Percent(100.),
                        height: Val::Px(UI_SCALE * 8.),
                        ..default()
                    })
                    .with_children(|parent| {
                        spawn_button(
                            ButtonType::Text {
                                text: "Reset all settings".into(),
                                font_size: UI_SCALE * 2.4,
                            },
                            ResetAllSettings,
                            parent,
                            &fonts,
                            &ui,
                        );
                    });
            });
    });
}

pub fn reset_all_settings(
    mut settings: ResMut<SettingsConfig>,
    buttons: Query<&Interaction, (With<ResetAllSettings>, Changed<Interaction>)>,
) {
    if buttons
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed)
    {
        *settings = SettingsConfig::default();
    }
}
