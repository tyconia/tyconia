use bevy::prelude::*;

use crate::loading::*;
use crate::ui::*;

const MIN_UI_SCALE: f32 = 0.2;
const MAX_UI_SCALE: f32 = 2.;

pub fn setup(
    mut cmd: Commands,
    backdrop: super::SettingsBackdropQuery,
    fonts: Res<FontAssets>,
    ui: Res<UiAssets>,

    ui_scale: Res<UiScale>,
) {
    let ui_scale_slider = Slider {
        range: (MIN_UI_SCALE..MAX_UI_SCALE),
        steps: 16,
        value: ui_scale.0,
    };

    cmd.entity(backdrop.single()).with_children(|parent| {
        parent
            .spawn((
                StateScoped(super::SettingsTabsState::Interface),
                Node {
                    flex_direction: FlexDirection::Column,
                    width: Val::Percent(100.),
                    ..default()
                },
            ))
            .with_children(|parent| {
                section_text("General", parent, &fonts);
                labeled_slider(parent, "UI scale", UiScaler, &ui, &fonts, ui_scale_slider);
            });
    });
}

#[derive(Debug, Component)]
pub struct UiScaler;

pub fn ui_scaling(
    mut ui_scale: ResMut<UiScale>,
    ui_scaler: Query<&Slider, (With<UiScaler>, Changed<Slider>)>,
) {
    const MIN_UI_SCALE: f32 = 1.4;
    const MAX_UI_SCALE: f32 = 1.;

    let _ = ui_scaler.get_single().map(|slider| {
        //ui_scale.0 = slider.value;
        info!("Scaling is {}", slider.value);
        //ui_scale.0 += 0.01;
    });
}
