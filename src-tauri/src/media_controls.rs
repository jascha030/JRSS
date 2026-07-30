use crate::AudioState;
use crate::audio;
use crate::audio::events::PlaybackStateEvent;
use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, PlatformConfig};
use tauri::{AppHandle, Listener, Manager};

/// Create MPRIS media controls (Linux) and wire them into the audio command
/// queue and playback-state-changed events.
pub fn install(app: &AppHandle) -> Result<(), String> {
    let mut media_controls = MediaControls::new(PlatformConfig {
        dbus_name: "jrss",
        display_name: "JRSS",
        hwnd: None,
    })
    .map_err(|e| format!("Failed to create media controls: {e}"))?;

    let app_handle_for_media = app.handle().clone();
    media_controls
        .attach(move |event: MediaControlEvent| {
            let audio_state = app_handle_for_media.state::<AudioState>();
            let cmd = match event {
                MediaControlEvent::Play => Some(audio::commands::AudioCommand::Resume),
                MediaControlEvent::Pause => Some(audio::commands::AudioCommand::Pause),
                MediaControlEvent::Toggle => Some(audio::commands::AudioCommand::TogglePlayback),
                MediaControlEvent::Next => Some(audio::commands::AudioCommand::SkipForward {
                    delta_seconds: 15.0,
                }),
                MediaControlEvent::Previous => Some(audio::commands::AudioCommand::SkipBackward {
                    delta_seconds: 15.0,
                }),
                MediaControlEvent::Stop => Some(audio::commands::AudioCommand::Stop),
                _ => None,
            };
            if let Some(cmd) = cmd {
                let _ = audio_state.send(cmd);
            }
        })
        .map_err(|e| format!("Failed to attach media controls: {e}"))?;

    app.manage(std::sync::Mutex::new(media_controls));

    let app_handle_for_events = app.handle().clone();
    let last_media_item_id = std::sync::Mutex::new(None::<String>);
    app.handle().listen("playback-state-changed", move |event| {
        let Ok(payload) = serde_json::from_str::<PlaybackStateEvent>(event.payload()) else {
            return;
        };

        let controls = app_handle_for_events.state::<std::sync::Mutex<MediaControls>>();
        let Ok(mut controls) = controls.lock() else {
            return;
        };

        let Ok(mut last_id) = last_media_item_id.lock() else {
            return;
        };

        if *last_id != Some(payload.item_id.clone()) {
            *last_id = Some(payload.item_id.clone());
            let _ = controls.set_metadata(MediaMetadata {
                title: Some(&payload.title),
                artist: Some(&payload.artist),
                album: Some(&payload.artist),
                ..Default::default()
            });
        }

        let playback = if payload.is_playing {
            MediaPlayback::Playing { progress: None }
        } else {
            MediaPlayback::Paused { progress: None }
        };
        let _ = controls.set_playback(playback);
    });

    Ok(())
}
