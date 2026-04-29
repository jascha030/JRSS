//! Audio output device management.

use rodio::cpal;
use rodio::cpal::traits::{DeviceTrait, HostTrait};
use rodio::{DeviceSinkBuilder, MixerDeviceSink};

use super::events::OutputDeviceInfo;

pub fn list_output_devices() -> Vec<OutputDeviceInfo> {
    let host = cpal::default_host();

    let default_id: Option<String> = host
        .default_output_device()
        .and_then(|device| device.id().ok())
        .map(|id| id.to_string());

    let Ok(devices) = host.output_devices() else {
        return Vec::new();
    };

    devices
        .filter_map(|device| {
            let id = device.id().ok()?;
            let desc = device.description().ok()?;
            let default_config = device.default_output_config().ok()?;
            let id_str = id.to_string();

            Some(OutputDeviceInfo {
                id: id_str.clone(),
                name: desc.to_string(),
                is_default: default_id.as_ref() == Some(&id_str),
                channels: Some(default_config.channels()),
                sample_rate: Some(default_config.sample_rate()),
            })
        })
        .collect()
}

pub fn find_output_device(device_id: Option<&str>) -> Result<cpal::Device, String> {
    let host = cpal::default_host();

    match device_id {
        Some(target) => {
            let mut devices = host
                .output_devices()
                .map_err(|e| format!("Failed to enumerate output devices: {e}"))?;

            devices
                .find(|device| {
                    device
                        .id()
                        .map(|id| id.to_string() == target)
                        .unwrap_or(false)
                })
                .ok_or_else(|| format!("Output device not found: {target}"))
        }
        None => host
            .default_output_device()
            .ok_or_else(|| "No default output device".to_string()),
    }
}

pub fn open_output_sink(selected_device_id: Option<&str>) -> Result<MixerDeviceSink, String> {
    let device = find_output_device(selected_device_id)?;

    let default_config = device
        .default_output_config()
        .map_err(|e| format!("Failed to query default output config: {e}"))?;

    let channels = default_config
        .channels()
        .try_into()
        .map_err(|_| "Invalid channel count for output sink".to_string())?;

    let sample_rate = default_config
        .sample_rate()
        .try_into()
        .map_err(|_| "Invalid sample rate for output sink".to_string())?;

    let builder = DeviceSinkBuilder::from_device(device)
        .map_err(|e| format!("Failed to create output sink builder: {e}"))?
        .with_channels(channels)
        .with_sample_rate(sample_rate);

    builder
        .open_sink_or_fallback()
        .map_err(|e| format!("Failed to open output sink: {e}"))
}
