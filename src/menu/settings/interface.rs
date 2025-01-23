use bevy::prelude::*;

use crate::loading::*;
use crate::ui::*;

const MIN_UI_SCALE: f32 = 0.9;
const MAX_UI_SCALE: f32 = 1.4;

pub fn setup(
    mut cmd: Commands,
    backdrop: super::SettingsBackdropQuery,
    fonts: Res<FontAssets>,
    ui: Res<UiAssets>,

    ui_scale: Res<UiScale>,
) {
    let ui_scale_slider = Slider::new(MIN_UI_SCALE..MAX_UI_SCALE, ui_scale.0, usize::MAX);

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
    let _ = ui_scaler.get_single().map(|slider| {
        ui_scale.0 = slider.valued();
        info!("Scaling is {}", slider.valued());
    });
}
