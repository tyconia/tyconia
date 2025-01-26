use crate::actions::{movement::MovementAction, set_movement_actions};
use crate::loading::AudioAssets;
use crate::GameState;
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

#[derive(Debug, Resource)]
pub struct MasterAudio;

#[derive(Debug, Resource, Default)]
pub struct MasterAudioVolume(pub f32);

#[derive(Debug, Resource)]
pub struct SFXAudioChannel;

#[derive(Debug, Resource, Default)]
pub struct SFXAudioChannelVolume(pub Volume);

#[derive(Debug, Resource)]
pub struct MusicAudioChannel;

#[derive(Debug, Resource, Default)]
pub struct MusicAudioChannelVolume(pub Volume);

pub struct InternalAudioPlugin;

// This plugin is responsible to control the game audio
impl Plugin for InternalAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AudioPlugin)
            .add_audio_channel::<MasterAudio>()
            .init_resource::<MasterAudioVolume>()
            .add_audio_channel::<SFXAudioChannel>()
            .init_resource::<SFXAudioChannelVolume>()
            .add_audio_channel::<MusicAudioChannel>()
            .init_resource::<MusicAudioChannelVolume>()
            .add_systems(OnEnter(GameState::Playing), start_audio)
            .add_systems(
                Update,
                control_flying_sound
                    .after(set_movement_actions)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Resource)]
struct FlyingAudio(Handle<AudioInstance>);

fn start_audio(mut commands: Commands, audio_assets: Res<AudioAssets>, audio: Res<Audio>) {
    audio.pause();
    let handle = audio
        .play(audio_assets.flying.clone())
        .looped()
        .with_volume(0.3)
        .handle();
    commands.insert_resource(FlyingAudio(handle));
}

fn control_flying_sound(
    mut player_movement_action: EventReader<MovementAction>,
    audio: Res<FlyingAudio>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    let player_movement = player_movement_action.read().last();

    if let Some(instance) = audio_instances.get_mut(&audio.0) {
        match instance.state() {
            PlaybackState::Paused { .. } => {
                if player_movement.is_some() {
                    instance.resume(AudioTween::default());
                }
            }
            PlaybackState::Playing { .. } => {
                if player_movement.is_none() {
                    instance.pause(AudioTween::default());
                }
            }
            _ => {}
        }
    }
}
