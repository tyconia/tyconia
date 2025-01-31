use crate::audio::*;
use crate::loading::*;
use crate::ui::*;
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

/// maximum safe volume in decibel level according to google
pub const MAX_SAFE_DECIBELS: f32 = 40.;
/// range from dead silence to maximum safe volume
pub const SILENCE_TO_MAX_SAFE_DECIBELS: std::ops::Range<f32> = (0.)..MAX_SAFE_DECIBELS;

#[derive(Debug, Component)]
pub enum AudioChannelSlider {
    Master,
    Music,
    FX,
}

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
                    AudioChannelSlider::Master,
                    &ui,
                    &fonts,
                    Slider::new((0.)..(1.0), 0.7, usize::MIN),
                );

                labeled_slider(
                    parent,
                    "Music",
                    AudioChannelSlider::Music,
                    &ui,
                    &fonts,
                    Slider::new(SILENCE_TO_MAX_SAFE_DECIBELS, 20., usize::MIN),
                );

                labeled_slider(
                    parent,
                    "Sound Effects",
                    AudioChannelSlider::FX,
                    &ui,
                    &fonts,
                    Slider::new(SILENCE_TO_MAX_SAFE_DECIBELS, 20., usize::MIN),
                );

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

pub fn configure(
    mut master: ResMut<MasterAudioVolume>,
    mut music: ResMut<MusicAudioChannelVolume>,
    mut fx: ResMut<SFXAudioChannelVolume>,
    slider: Query<(&Slider, &AudioChannelSlider), Changed<Slider>>,
) {
    for (slider, channel) in slider.iter() {
        match channel {
            AudioChannelSlider::Master => {
                master.0 = slider.valued();
                info!("master volume set to {:?}", master.0);
            }
            AudioChannelSlider::Music => {
                music.0 = Volume::Decibels(slider.valued() as f64);
                info!("music volume set to {:?}", music.0);
            }
            AudioChannelSlider::FX => {
                fx.0 = Volume::Decibels(slider.valued() as f64);
                info!("fx volume set to {:?}", fx.0);
            }
        }
    }
}
