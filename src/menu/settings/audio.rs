use crate::audio::*;
use crate::loading::*;
use crate::ui::*;
use bevy::prelude::*;

/// maximum safe volume in decibel level according to google
pub const MAX_SAFE_DECIBELS: f32 = 70.;
/// range from dead silence to maximum safe volume
pub const SILENCE_TO_MAX_SAFE_DECIBELS: std::ops::Range<f32> = (0.)..MAX_SAFE_DECIBELS;

pub fn setup(
    mut cmd: Commands,
    backdrop: super::SettingsBackdropQuery,
    fonts: Res<FontAssets>,
    ui: Res<UiAssets>,
) {
    cmd.entity(backdrop.single()).with_children(|parent| {
        parent
            .spawn((
                StateScoped(super::SettingsTabsState::Audio),
                Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
            ))
            .with_children(|parent| {
                section_text("General", parent, &fonts);
                labeled_slider(
                    parent,
                    "Master",
                    (),
                    &ui,
                    &fonts,
                    Slider::new(SILENCE_TO_MAX_SAFE_DECIBELS, 10., usize::MIN),
                );
                spawn_checkbox((), parent, &fonts, &ui);
                //labeled_slider(parent, "Music", (), &ui, &fonts, 50.);
                //labeled_slider(parent, "Sound Effects", (), &ui, &fonts, 50.);
                separator(parent);
                //labeled_slider(parent, "Indoor Ambience", (), &ui, &fonts, 50.);
                //labeled_slider(parent, "City Ambience", (), &ui, &fonts, 50.);

                separator(parent);

                section_text("Misc", parent, &fonts);
                //labeled_slider(parent, "Interface", (), &ui, &fonts, 50.);
                //labeled_slider(parent, "Notifications", (), &ui, &fonts, 50.);

                separator(parent);
                section_text("Mods", parent, &fonts);
            });
    });
}

pub fn configure() {}
