use bevy::prelude::*;

use crate::loading::*;
use crate::settings::{
    SettingsConfig, MAX_UI_SCALE as SETTINGS_MAX_UI_SCALE, MIN_UI_SCALE as SETTINGS_MIN_UI_SCALE,
};
use crate::ui::*;

const MIN_UI_SCALE: f32 = SETTINGS_MIN_UI_SCALE;
const MAX_UI_SCALE: f32 = SETTINGS_MAX_UI_SCALE;

pub fn setup(
    mut cmd: Commands,
    backdrop: super::SettingsBackdropQuery,
    fonts: Res<FontAssets>,
    ui: Res<UiAssets>,
    settings: Res<SettingsConfig>,
) {
    let ui_scale_slider = Slider::new(MIN_UI_SCALE..MAX_UI_SCALE, settings.interface.ui_scale, 26);

    let Ok(backdrop) = backdrop.get_single() else {
        return;
    };

    cmd.entity(backdrop).with_children(|parent| {
        parent
            .spawn((
                StateScoped(super::SettingsTabsState::GeneralInterface),
                Node {
                    flex_direction: FlexDirection::Column,
                    width: Val::Percent(100.),
                    ..default()
                },
            ))
            .with_children(|parent| {
                section_text("General", parent, &fonts);
                labeled_slider(parent, "UI scale", UiScaler, &ui, &fonts, ui_scale_slider);
                separator(parent);
                section_text("Interface behavior", parent, &fonts);
                checkbox_row(
                    parent,
                    "Reduce motion",
                    settings.interface.reduced_motion,
                    ReducedMotionCheckbox,
                    &ui,
                    &fonts,
                );
                checkbox_row(
                    parent,
                    "High contrast focus",
                    settings.interface.high_contrast,
                    HighContrastCheckbox,
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
pub struct UiScaler;

#[derive(Debug, Component)]
pub struct ReducedMotionCheckbox;

#[derive(Debug, Component)]
pub struct HighContrastCheckbox;

pub fn ui_scaling(
    mut ui_scale: ResMut<UiScale>,
    mut settings: ResMut<SettingsConfig>,
    ui_scaler: Query<&Slider, (With<UiScaler>, Changed<Slider>)>,
) {
    if let Ok(slider) = ui_scaler.get_single() {
        let scale = slider.valued().clamp(MIN_UI_SCALE, MAX_UI_SCALE);
        ui_scale.0 = scale;
        settings.interface.ui_scale = scale;
    }
}

pub fn configure_interface_checkboxes(
    mut settings: ResMut<SettingsConfig>,
    reduced: Query<&CheckboxState, (With<ReducedMotionCheckbox>, Changed<CheckboxState>)>,
    contrast: Query<&CheckboxState, (With<HighContrastCheckbox>, Changed<CheckboxState>)>,
) {
    for state in reduced.iter() {
        settings.interface.reduced_motion = state.is_active();
    }
    for state in contrast.iter() {
        settings.interface.high_contrast = state.is_active();
    }
}
