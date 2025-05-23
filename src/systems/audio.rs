use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use std::time::Duration;

use crate::screens::Screen;

pub fn plugin(app: &mut App) {
    app.init_resource::<AudioResources>()
        .add_systems(Startup, setup_audio)
        .add_systems(Update, (handle_screen_transitions, instance_control));
}

/// Control instance playback - currently this allows pausing/resuming with mouse click
fn instance_control(
    audio_resources: Res<AudioResources>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    // Get the current music instance if available
    if let Some(handle) = &audio_resources.current_music {
        if let Some(instance) = audio_instances.get_mut(handle) {
            if input.just_pressed(KeyCode::KeyM) {
                match instance.state() {
                    PlaybackState::Paused { .. } => {
                        instance.resume(AudioTween::default());
                    }
                    PlaybackState::Playing { .. } => {
                        instance.pause(AudioTween::default());
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Resource to track audio state and handles
#[derive(Resource, Default)]
struct AudioResources {
    current_music: Option<Handle<AudioInstance>>,
    main_menu_track: Option<Handle<AudioSource>>,
}

/// Setup audio resources and load assets
fn setup_audio(
    // mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut audio_resources: ResMut<AudioResources>,
) {
    // Preload the main menu audio track
    audio_resources.main_menu_track = Some(asset_server.load("audio/main_menu.ogg"));
}

/// System to handle screen transitions and play appropriate music
fn handle_screen_transitions(
    audio: Res<Audio>,
    mut audio_resources: ResMut<AudioResources>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    current_state: Res<State<Screen>>,
    mut prev_state: Local<Screen>,
) {
    // Only process if the state has changed
    if *prev_state == *current_state.get() {
        return;
    }

    // Update previous state for next frame
    *prev_state = current_state.get().clone();

    // Stop any currently playing music with a fade out
    if let Some(handle) = &audio_resources.current_music {
        if let Some(instance) = audio_instances.get_mut(handle) {
            instance.stop(AudioTween::new(
                Duration::from_secs(1),
                AudioEasing::OutPowf(2.0),
            ));
        }
        audio_resources.current_music = None;
    }

    // Play appropriate music based on the current screen
    match current_state.get() {
        Screen::MainMenu => {
            if let Some(track) = &audio_resources.main_menu_track {
                let handle = audio
                    .play(track.clone())
                    .looped()
                    .fade_in(AudioTween::new(
                        Duration::new(2, 0),
                        AudioEasing::OutPowf(2.0),
                    ))
                    .handle();
                audio_resources.current_music = Some(handle);
            }
        }
        Screen::NewGame => {
            if let Some(track) = &audio_resources.main_menu_track {
                let handle = audio
                    .play(track.clone())
                    .looped()
                    .fade_in(AudioTween::new(
                        Duration::new(2, 0),
                        AudioEasing::OutPowf(2.0),
                    ))
                    .handle();
                audio_resources.current_music = Some(handle);
            }
        }
        _ => {
            // No music for other screens like gameplay
        }
    }
}
