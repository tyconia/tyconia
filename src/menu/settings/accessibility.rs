use super::SettingsTabsState;
use crate::loading::{FontAssets, UiAssets};
use crate::settings::{ColorblindMode, SettingsConfig};
use crate::ui::*;
use bevy::prelude::*;

pub fn setup_visual(
    mut cmd: Commands,
    backdrop: super::SettingsBackdropQuery,
    fonts: Res<FontAssets>,
    ui: Res<UiAssets>,
    settings: Res<SettingsConfig>,
) {
    let Ok(backdrop) = backdrop.get_single() else {
        return;
    };

    let options = ColorblindMode::ALL
        .iter()
        .map(|mode| mode.label().to_string())
        .collect::<Vec<_>>();
    let selected = ColorblindMode::ALL
        .iter()
        .position(|mode| *mode == settings.accessibility.colorblind_mode);

    cmd.entity(backdrop).with_children(|parent| {
        parent
            .spawn((
                StateScoped(SettingsTabsState::AccessibilityVisual),
                Node {
                    flex_direction: FlexDirection::Column,
                    width: Val::Percent(100.),
                    row_gap: Val::Px(UI_SCALE),
                    ..default()
                },
            ))
            .with_children(|parent| {
                section_text("Visual Accessibility", parent, &fonts);
                parent
                    .spawn(Node {
                        width: Val::Percent(100.),
                        height: Val::Px(UI_SCALE * 6.),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|parent| {
                        body_text("Colorblind mode", parent, &fonts);
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
                                    "Select mode".into(),
                                    selected,
                                    ColorblindDropdown,
                                );
                            });
                    });

                checkbox_row(
                    parent,
                    "Subtitles",
                    settings.accessibility.subtitles,
                    SubtitlesCheckbox,
                    &ui,
                    &fonts,
                );
                checkbox_row(
                    parent,
                    "Reduce screen shake",
                    !settings.accessibility.screen_shake,
                    ScreenShakeCheckbox,
                    &ui,
                    &fonts,
                );
            });
    });
}

pub fn setup_audio(
    mut cmd: Commands,
    backdrop: super::SettingsBackdropQuery,
    fonts: Res<FontAssets>,
    ui: Res<UiAssets>,
    settings: Res<SettingsConfig>,
) {
    let Ok(backdrop) = backdrop.get_single() else {
        return;
    };

    cmd.entity(backdrop).with_children(|parent| {
        parent
            .spawn((
                StateScoped(SettingsTabsState::AccessibilityAudio),
                Node {
                    flex_direction: FlexDirection::Column,
                    width: Val::Percent(100.),
                    row_gap: Val::Px(UI_SCALE),
                    ..default()
                },
            ))
            .with_children(|parent| {
                section_text("Audio Accessibility", parent, &fonts);
                checkbox_row(
                    parent,
                    "Visual notifications for audio cues",
                    settings.accessibility.visual_audio_cues,
                    VisualCuesCheckbox,
                    &ui,
                    &fonts,
                );
            });
    });
}

fn checkbox_row<C: Component>(
    parent: &mut ChildBuilder,
    label: &str,
    active: bool,
    marker: C,
    ui: &Res<UiAssets>,
    fonts: &Res<FontAssets>,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.),
            height: Val::Px(UI_SCALE * 5.),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|parent| {
            body_text(label, parent, fonts);
            parent
                .spawn(Node {
                    height: Val::Px(UI_SCALE * 4.),
                    ..default()
                })
                .with_children(|parent| {
                    spawn_checkbox(
                        if active {
                            CheckboxState::Active
                        } else {
                            CheckboxState::Inactive
                        },
                        marker,
                        parent,
                        fonts,
                        ui,
                    );
                });
        });
}

#[derive(Debug, Component)]
pub struct ColorblindDropdown;

#[derive(Debug, Component)]
pub struct SubtitlesCheckbox;

#[derive(Debug, Component)]
pub struct ScreenShakeCheckbox;

#[derive(Debug, Component)]
pub struct VisualCuesCheckbox;

pub fn configure_accessibility(
    mut settings: ResMut<SettingsConfig>,
    colorblind: Query<&Dropdown, (With<ColorblindDropdown>, Changed<Dropdown>)>,
    subtitles: Query<&CheckboxState, (With<SubtitlesCheckbox>, Changed<CheckboxState>)>,
    screen_shake: Query<&CheckboxState, (With<ScreenShakeCheckbox>, Changed<CheckboxState>)>,
    visual_cues: Query<&CheckboxState, (With<VisualCuesCheckbox>, Changed<CheckboxState>)>,
) {
    for dropdown in colorblind.iter() {
        if let Some(mode) = dropdown
            .selected
            .and_then(|index| ColorblindMode::ALL.get(index).copied())
        {
            if settings.accessibility.colorblind_mode != mode {
                settings.accessibility.colorblind_mode = mode;
            }
        }
    }
    for state in subtitles.iter() {
        settings.accessibility.subtitles = state.is_active();
    }
    for state in screen_shake.iter() {
        settings.accessibility.screen_shake = !state.is_active();
    }
    for state in visual_cues.iter() {
        settings.accessibility.visual_audio_cues = state.is_active();
    }
}
