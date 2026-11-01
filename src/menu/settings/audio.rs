use crate::loading::*;
use crate::settings::{SettingsConfig, MAX_SAFE_DECIBELS};
use crate::ui::*;
use bevy::prelude::*;

/// range from dead silence to maximum safe volume
pub const SILENCE_TO_MAX_SAFE_DECIBELS: std::ops::Range<f32> = (0.)..MAX_SAFE_DECIBELS;

#[derive(Debug, Component)]
pub enum AudioChannelSlider {
    Master,
    Music,
    FX,
}

#[derive(Debug, Component)]
pub struct AudioMuteCheckbox;

pub fn setup_master(
    cmd: Commands,
    backdrop: super::SettingsBackdropQuery,
    fonts: Res<FontAssets>,
    ui: Res<UiAssets>,
    settings: Res<SettingsConfig>,
) {
    setup_channel(
        cmd,
        backdrop,
        fonts,
        ui,
        super::SettingsTabsState::AudioMaster,
        "Master",
        AudioChannelSlider::Master,
        Slider::new((0.)..(1.0), settings.audio.master, 101),
        Some(settings.audio.muted),
    );
}

pub fn setup_music(
    cmd: Commands,
    backdrop: super::SettingsBackdropQuery,
    fonts: Res<FontAssets>,
    ui: Res<UiAssets>,
    settings: Res<SettingsConfig>,
) {
    setup_channel(
        cmd,
        backdrop,
        fonts,
        ui,
        super::SettingsTabsState::AudioMusic,
        "Music",
        AudioChannelSlider::Music,
        Slider::new(SILENCE_TO_MAX_SAFE_DECIBELS, settings.audio.music, 81),
        None,
    );
}

pub fn setup_sfx(
    cmd: Commands,
    backdrop: super::SettingsBackdropQuery,
    fonts: Res<FontAssets>,
    ui: Res<UiAssets>,
    settings: Res<SettingsConfig>,
) {
    setup_channel(
        cmd,
        backdrop,
        fonts,
        ui,
        super::SettingsTabsState::AudioSfx,
        "Sound Effects",
        AudioChannelSlider::FX,
        Slider::new(SILENCE_TO_MAX_SAFE_DECIBELS, settings.audio.sfx, 81),
        None,
    );
}

fn setup_channel(
    mut cmd: Commands,
    backdrop: super::SettingsBackdropQuery,
    fonts: Res<FontAssets>,
    ui: Res<UiAssets>,
    state: super::SettingsTabsState,
    label: &str,
    channel: AudioChannelSlider,
    slider: Slider,
    muted: Option<bool>,
) {
    let Ok(backdrop) = backdrop.get_single() else {
        return;
    };

    cmd.entity(backdrop).with_children(|parent| {
        parent
            .spawn((
                StateScoped(state),
                Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
            ))
            .with_children(|parent| {
                section_text("Audio", parent, &fonts);
                labeled_slider(parent, label, channel, &ui, &fonts, slider);

                if let Some(muted) = muted {
                    parent
                        .spawn(Node {
                            width: Val::Percent(100.),
                            height: Val::Px(UI_SCALE * 5.),
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|parent| {
                            body_text("Mute all audio", parent, &fonts);
                            parent
                                .spawn(Node {
                                    height: Val::Px(UI_SCALE * 4.),
                                    ..default()
                                })
                                .with_children(|parent| {
                                    spawn_checkbox(
                                        if muted {
                                            CheckboxState::Active
                                        } else {
                                            CheckboxState::Inactive
                                        },
                                        AudioMuteCheckbox,
                                        parent,
                                        &fonts,
                                        &ui,
                                    );
                                });
                        });
                }
            });
    });
}

pub fn configure_master(
    mut settings: ResMut<SettingsConfig>,
    slider: Query<(&Slider, &AudioChannelSlider), Changed<Slider>>,
) {
    for (slider, channel) in slider.iter() {
        if matches!(channel, AudioChannelSlider::Master) {
            settings.audio.master = slider.valued().clamp(0.0, 1.0);
        }
    }
}

pub fn configure_music(
    mut settings: ResMut<SettingsConfig>,
    slider: Query<(&Slider, &AudioChannelSlider), Changed<Slider>>,
) {
    for (slider, channel) in slider.iter() {
        if matches!(channel, AudioChannelSlider::Music) {
            settings.audio.music = slider.valued().clamp(0.0, MAX_SAFE_DECIBELS);
        }
    }
}

pub fn configure_sfx(
    mut settings: ResMut<SettingsConfig>,
    slider: Query<(&Slider, &AudioChannelSlider), Changed<Slider>>,
) {
    for (slider, channel) in slider.iter() {
        if matches!(channel, AudioChannelSlider::FX) {
            settings.audio.sfx = slider.valued().clamp(0.0, MAX_SAFE_DECIBELS);
        }
    }
}

pub fn configure_mute(
    mut settings: ResMut<SettingsConfig>,
    checkbox: Query<&CheckboxState, (With<AudioMuteCheckbox>, Changed<CheckboxState>)>,
) {
    for state in checkbox.iter() {
        settings.audio.muted = state.is_active();
    }
}
