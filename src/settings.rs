use std::fs;
use std::path::PathBuf;

use bevy::prelude::*;
use bevy::window::{MonitorSelection, PresentMode, PrimaryWindow, WindowMode};
use bevy_kira_audio::prelude::*;
use serde::{Deserialize, Serialize};

use crate::audio::{
    MasterAudio, MasterAudioVolume, MusicAudioChannel, MusicAudioChannelVolume, SFXAudioChannel,
    SFXAudioChannelVolume,
};

pub const MIN_UI_SCALE: f32 = 0.75;
pub const MAX_UI_SCALE: f32 = 1.6;
pub const MAX_SAFE_DECIBELS: f32 = 40.0;

#[derive(Resource, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct SettingsConfig {
    pub audio: AudioSettings,
    pub graphics: GraphicsSettings,
    pub interface: InterfaceSettings,
    pub accessibility: AccessibilitySettings,
    pub language: Language,
    pub mods: ModSettings,
    pub developer: DeveloperSettings,
}

impl Default for SettingsConfig {
    fn default() -> Self {
        Self {
            audio: AudioSettings::default(),
            graphics: GraphicsSettings::default(),
            interface: InterfaceSettings::default(),
            accessibility: AccessibilitySettings::default(),
            language: Language::default(),
            mods: ModSettings::default(),
            developer: DeveloperSettings::default(),
        }
    }
}

impl SettingsConfig {
    pub fn load() -> Self {
        let Some(path) = settings_path() else {
            return Self::default();
        };
        let Ok(contents) = fs::read_to_string(&path) else {
            return Self::default();
        };
        ron::from_str(&contents).unwrap_or_else(|error| {
            warn!(
                "Failed to parse settings at {}: {}. Using defaults.",
                path.display(),
                error
            );
            Self::default()
        })
    }

    pub fn save(&self) -> Result<(), String> {
        let path = settings_path().ok_or_else(|| "No configuration directory".to_string())?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let ron = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .map_err(|error| error.to_string())?;
        fs::write(&path, ron).map_err(|error| error.to_string())
    }
}

pub fn settings_path() -> Option<PathBuf> {
    directories::ProjectDirs::from(
        env!("PROJECT_QUALIFIER"),
        env!("PROJECT_ORGANIZATION"),
        env!("PROJECT_APPLICATION"),
    )
    .map(|dirs| dirs.config_dir().join("settings.ron"))
}

pub fn mods_dir() -> Option<PathBuf> {
    directories::ProjectDirs::from(
        env!("PROJECT_QUALIFIER"),
        env!("PROJECT_ORGANIZATION"),
        env!("PROJECT_APPLICATION"),
    )
    .map(|dirs| dirs.config_dir().join("mods"))
}

pub fn saves_dir() -> Option<PathBuf> {
    directories::ProjectDirs::from(
        env!("PROJECT_QUALIFIER"),
        env!("PROJECT_ORGANIZATION"),
        env!("PROJECT_APPLICATION"),
    )
    .map(|dirs| dirs.config_dir().join("saves"))
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct AudioSettings {
    pub master: f32,
    pub music: f32,
    pub sfx: f32,
    pub muted: bool,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master: 0.7,
            music: 20.0,
            sfx: 20.0,
            muted: false,
        }
    }
}

impl AudioSettings {
    pub fn effective_master(&self) -> f32 {
        if self.muted {
            0.0
        } else {
            self.master.clamp(0.0, 1.0)
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct GraphicsSettings {
    pub window_mode: WindowModeSetting,
    pub msaa: MsaaSetting,
    pub vsync: bool,
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            window_mode: WindowModeSetting::Windowed,
            msaa: MsaaSetting::Sample4,
            vsync: true,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowModeSetting {
    Windowed,
    BorderlessFullscreen,
    Fullscreen,
}

impl WindowModeSetting {
    pub const ALL: [Self; 3] = [Self::Windowed, Self::BorderlessFullscreen, Self::Fullscreen];

    pub fn label(self) -> &'static str {
        match self {
            Self::Windowed => "Windowed",
            Self::BorderlessFullscreen => "Borderless Fullscreen",
            Self::Fullscreen => "Fullscreen",
        }
    }

    pub fn to_bevy(self) -> WindowMode {
        match self {
            Self::Windowed => WindowMode::Windowed,
            Self::BorderlessFullscreen => {
                WindowMode::BorderlessFullscreen(MonitorSelection::Primary)
            }
            Self::Fullscreen => WindowMode::Fullscreen(MonitorSelection::Primary),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum MsaaSetting {
    Off,
    Sample2,
    Sample4,
    Sample8,
}

impl MsaaSetting {
    pub const ALL: [Self; 4] = [Self::Off, Self::Sample2, Self::Sample4, Self::Sample8];

    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Sample2 => "2x",
            Self::Sample4 => "4x",
            Self::Sample8 => "8x",
        }
    }

    pub fn to_bevy(self) -> Msaa {
        match self {
            Self::Off => Msaa::Off,
            Self::Sample2 => Msaa::Sample2,
            Self::Sample4 => Msaa::Sample4,
            Self::Sample8 => Msaa::Sample8,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct InterfaceSettings {
    pub ui_scale: f32,
    pub reduced_motion: bool,
    pub high_contrast: bool,
}

impl Default for InterfaceSettings {
    fn default() -> Self {
        Self {
            ui_scale: 1.0,
            reduced_motion: false,
            high_contrast: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct AccessibilitySettings {
    pub colorblind_mode: ColorblindMode,
    pub visual_audio_cues: bool,
    pub subtitles: bool,
    pub screen_shake: bool,
}

impl Default for AccessibilitySettings {
    fn default() -> Self {
        Self {
            colorblind_mode: ColorblindMode::None,
            visual_audio_cues: true,
            subtitles: false,
            screen_shake: true,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorblindMode {
    None,
    Protanopia,
    Deuteranopia,
    Tritanopia,
}

impl ColorblindMode {
    pub const ALL: [Self; 4] = [
        Self::None,
        Self::Protanopia,
        Self::Deuteranopia,
        Self::Tritanopia,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Protanopia => "Protanopia",
            Self::Deuteranopia => "Deuteranopia",
            Self::Tritanopia => "Tritanopia",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Language {
    #[default]
    English,
    German,
    French,
    Spanish,
}

impl Language {
    pub const ALL: [Self; 4] = [Self::English, Self::German, Self::French, Self::Spanish];

    pub fn label(self) -> &'static str {
        match self {
            Self::English => "English",
            Self::German => "Deutsch",
            Self::French => "Francais",
            Self::Spanish => "Espanol",
        }
    }

    pub fn text(self, key: &str) -> &'static str {
        match (self, key) {
            (Self::German, "continue") => "Fortsetzen",
            (Self::German, "new_game") => "Neues Spiel",
            (Self::German, "load_game") => "Spiel laden",
            (Self::German, "multiplayer") => "Mehrspieler",
            (Self::German, "settings") => "Einstellungen",
            (Self::German, "credits") => "Mitwirkende",
            (Self::German, "quit") => "Beenden",
            (Self::French, "continue") => "Continuer",
            (Self::French, "new_game") => "Nouvelle partie",
            (Self::French, "load_game") => "Charger",
            (Self::French, "multiplayer") => "Multijoueur",
            (Self::French, "settings") => "Parametres",
            (Self::French, "credits") => "Credits",
            (Self::French, "quit") => "Quitter",
            (Self::Spanish, "continue") => "Continuar",
            (Self::Spanish, "new_game") => "Nueva partida",
            (Self::Spanish, "load_game") => "Cargar partida",
            (Self::Spanish, "multiplayer") => "Multijugador",
            (Self::Spanish, "settings") => "Ajustes",
            (Self::Spanish, "credits") => "Creditos",
            (Self::Spanish, "quit") => "Salir",
            (_, "continue") => "Continue",
            (_, "new_game") => "New Game",
            (_, "load_game") => "Load Game",
            (_, "multiplayer") => "Multiplayer",
            (_, "settings") => "Settings",
            (_, "credits") => "Credits",
            (_, "quit") => "Quit",
            (_, _) => "",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(default)]
pub struct ModSettings {
    pub enabled: Vec<String>,
}

impl Default for ModSettings {
    fn default() -> Self {
        Self {
            enabled: vec!["tyconic_0.0.0-dev".into(), "base_0.0.0-dev".into()],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(default)]
pub struct DeveloperSettings {
    pub log_level: LogLevelSetting,
}

impl Default for DeveloperSettings {
    fn default() -> Self {
        Self {
            log_level: LogLevelSetting::Info,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevelSetting {
    Error,
    Warning,
    Info,
    Debug,
    Trace,
}

impl LogLevelSetting {
    pub const ALL: [Self; 5] = [
        Self::Error,
        Self::Warning,
        Self::Info,
        Self::Debug,
        Self::Trace,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Error => "Error",
            Self::Warning => "Warning",
            Self::Info => "Info",
            Self::Debug => "Debug",
            Self::Trace => "Trace",
        }
    }
}

pub struct GameSettingsPlugin;

impl Plugin for GameSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SettingsConfig>()
            .add_systems(PreStartup, load_settings)
            .add_systems(
                Update,
                (
                    apply_audio_settings.run_if(resource_changed::<SettingsConfig>),
                    apply_graphics_settings.run_if(resource_changed::<SettingsConfig>),
                    apply_interface_settings.run_if(resource_changed::<SettingsConfig>),
                    persist_settings,
                ),
            );
    }
}

fn load_settings(mut settings: ResMut<SettingsConfig>) {
    *settings = SettingsConfig::load();
}

fn persist_settings(settings: Res<SettingsConfig>, mut last: Local<Option<SettingsConfig>>) {
    if last.as_ref() == Some(&*settings) {
        return;
    }
    if let Err(error) = settings.save() {
        warn!("Failed to save settings: {error}");
    }
    *last = Some(settings.clone());
}

fn apply_audio_settings(
    settings: Res<SettingsConfig>,
    mut master_volume: ResMut<MasterAudioVolume>,
    mut music_volume: ResMut<MusicAudioChannelVolume>,
    mut sfx_volume: ResMut<SFXAudioChannelVolume>,
    music_channel: Res<AudioChannel<MusicAudioChannel>>,
    sfx_channel: Res<AudioChannel<SFXAudioChannel>>,
    master_channel: Res<AudioChannel<MasterAudio>>,
) {
    let master = settings.audio.effective_master();
    master_volume.0 = master;
    master_channel.set_volume(master as f64);

    let music = combined_volume(settings.audio.music, master);
    let sfx = combined_volume(settings.audio.sfx, master);

    music_volume.0 = music;
    sfx_volume.0 = sfx;
    music_channel.set_volume(music);
    sfx_channel.set_volume(sfx);
}

fn combined_volume(decibels: f32, master: f32) -> Volume {
    if master <= f32::EPSILON || decibels <= -80.0 {
        return Volume::Amplitude(0.0);
    }
    let amplitude = 10f32.powf(decibels / 20.0) * master;
    Volume::Amplitude(amplitude as f64)
}

fn apply_graphics_settings(
    settings: Res<SettingsConfig>,
    mut last: Local<Option<GraphicsSettings>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut cameras: Query<&mut Msaa>,
) {
    if last.as_ref() == Some(&settings.graphics) {
        return;
    }

    let mode = settings.graphics.window_mode.to_bevy();
    let present_mode = if settings.graphics.vsync {
        PresentMode::AutoVsync
    } else {
        PresentMode::AutoNoVsync
    };

    for mut window in windows.iter_mut() {
        window.mode = mode;
        window.present_mode = present_mode;
    }

    let msaa = settings.graphics.msaa.to_bevy();
    for mut camera_msaa in cameras.iter_mut() {
        *camera_msaa = msaa;
    }

    *last = Some(settings.graphics.clone());
}

fn apply_interface_settings(settings: Res<SettingsConfig>, mut ui_scale: ResMut<UiScale>) {
    let scale = settings
        .interface
        .ui_scale
        .clamp(MIN_UI_SCALE, MAX_UI_SCALE);
    if ui_scale.0 != scale {
        ui_scale.0 = scale;
    }
}
