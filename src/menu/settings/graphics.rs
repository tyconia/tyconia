use bevy::prelude::*;

use crate::loading::*;
use crate::settings::{MsaaSetting, SettingsConfig, WindowModeSetting};
use crate::ui::*;

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

    cmd.entity(backdrop).with_children(|parent| {
        parent
            .spawn((
                StateScoped(super::SettingsTabsState::GeneralGraphics),
                Node {
                    flex_direction: FlexDirection::Column,
                    width: Val::Percent(100.),
                    row_gap: Val::Px(UI_SCALE),
                    ..default()
                },
            ))
            .with_children(|parent| {
                section_text("Display", parent, &fonts);

                let window_options = WindowModeSetting::ALL
                    .iter()
                    .map(|mode| mode.label().to_string())
                    .collect::<Vec<_>>();
                let window_selected = WindowModeSetting::ALL
                    .iter()
                    .position(|mode| *mode == settings.graphics.window_mode);
                setting_row(
                    parent,
                    "Window mode",
                    &ui,
                    &fonts,
                    &window_options,
                    window_selected,
                    WindowModeDropdown,
                );

                let msaa_options = MsaaSetting::ALL
                    .iter()
                    .map(|mode| mode.label().to_string())
                    .collect::<Vec<_>>();
                let msaa_selected = MsaaSetting::ALL
                    .iter()
                    .position(|mode| *mode == settings.graphics.msaa);
                setting_row(
                    parent,
                    "Anti-aliasing",
                    &ui,
                    &fonts,
                    &msaa_options,
                    msaa_selected,
                    MsaaDropdown,
                );

                checkbox_row(
                    parent,
                    "Vertical sync",
                    settings.graphics.vsync,
                    VsyncCheckbox,
                    &ui,
                    &fonts,
                );
            });
    });
}

fn setting_row<C: Component>(
    parent: &mut ChildBuilder,
    label: &str,
    ui: &Res<UiAssets>,
    fonts: &Res<FontAssets>,
    options: &[String],
    selected: Option<usize>,
    marker: C,
) {
    parent
        .spawn(Node {
            width: Val::Percent(100.),
            height: Val::Px(UI_SCALE * 6.),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|parent| {
            body_text(label, parent, fonts);
            parent
                .spawn(Node {
                    width: Val::Px(220.),
                    height: Val::Px(UI_SCALE * 5.),
                    ..default()
                })
                .with_children(|parent| {
                    spawn_dropdown(
                        parent,
                        ui,
                        fonts,
                        options,
                        "Select".into(),
                        selected,
                        marker,
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
pub struct WindowModeDropdown;

#[derive(Debug, Component)]
pub struct MsaaDropdown;

#[derive(Debug, Component)]
pub struct VsyncCheckbox;

pub fn configure_graphics(
    mut settings: ResMut<SettingsConfig>,
    window_mode: Query<&Dropdown, (With<WindowModeDropdown>, Changed<Dropdown>)>,
    msaa: Query<&Dropdown, (With<MsaaDropdown>, Changed<Dropdown>)>,
    vsync: Query<&CheckboxState, (With<VsyncCheckbox>, Changed<CheckboxState>)>,
) {
    for dropdown in window_mode.iter() {
        if let Some(mode) = dropdown
            .selected
            .and_then(|index| WindowModeSetting::ALL.get(index).copied())
        {
            if settings.graphics.window_mode != mode {
                settings.graphics.window_mode = mode;
            }
        }
    }

    for dropdown in msaa.iter() {
        if let Some(mode) = dropdown
            .selected
            .and_then(|index| MsaaSetting::ALL.get(index).copied())
        {
            if settings.graphics.msaa != mode {
                settings.graphics.msaa = mode;
            }
        }
    }

    for state in vsync.iter() {
        let vsync = state.is_active();
        if settings.graphics.vsync != vsync {
            settings.graphics.vsync = vsync;
        }
    }
}
