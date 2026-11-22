use std::io::Cursor;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread;

enum AudioCommand {
    Play { bytes: Arc<[u8]>, volume: f32 },
    Shutdown,
}

#[derive(Clone)]
pub struct AudioHandle {
    tx: Sender<AudioCommand>,
}

impl AudioHandle {
    pub fn spawn(device: Option<String>) -> Self {
        let (tx, rx) = mpsc::channel();
        let _ = thread::Builder::new()
            .name("ticky-audio".into())
            .spawn(move || audio_loop(rx, device));
        Self { tx }
    }

    pub fn play(&self, bytes: Arc<[u8]>, volume: f32) {
        let _ = self.tx.send(AudioCommand::Play { bytes, volume });
    }

    pub fn shutdown(&self) {
        let _ = self.tx.send(AudioCommand::Shutdown);
    }
}

fn audio_loop(rx: Receiver<AudioCommand>, device: Option<String>) {
    let (stream, handle) = match open_output(device.as_deref()) {
        Some(pair) => pair,
        None => {
            eprintln!("ticky: audio output unavailable");
            return;
        }
    };
    eprintln!("ticky: audio output ready");

    while let Ok(command) = rx.try_recv() {
        if matches!(command, AudioCommand::Shutdown) {
            drop(stream);
            return;
        }
    }

    while let Ok(command) = rx.recv() {
        match command {
            AudioCommand::Play { bytes, volume } => {
                let reader = Cursor::new(bytes.to_vec());
                let decoder = match rodio::Decoder::new(reader) {
                    Ok(decoder) => decoder,
                    Err(err) => {
                        eprintln!("ticky: decode failed: {err}");
                        continue;
                    }
                };
                let sink = match rodio::Sink::try_new(&handle) {
                    Ok(sink) => sink,
                    Err(err) => {
                        eprintln!("ticky: sink failed: {err}");
                        continue;
                    }
                };
                sink.set_volume(volume);
                sink.append(decoder);
                sink.detach();
            }
            AudioCommand::Shutdown => break,
        }
    }

    drop(stream);
}

pub fn output_devices() -> Vec<String> {
    use rodio::cpal::traits::{DeviceTrait, HostTrait};

    let mut names: Vec<String> = rodio::cpal::default_host()
        .output_devices()
        .map(|devices| {
            devices
                .filter_map(|device| device.name().ok())
                .filter(|name| {
                    let lower = name.to_ascii_lowercase();
                    !lower.contains("steam")
                        && !lower.contains("voicemod")
                        && !lower.contains("virtual")
                        && !lower.contains("dummy")
                })
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names.dedup();
    names
}

fn open_output(selected: Option<&str>) -> Option<(rodio::OutputStream, rodio::OutputStreamHandle)> {
    use rodio::cpal::traits::{DeviceTrait, HostTrait};

    if let Some(selected) = selected {
        if let Ok(devices) = rodio::cpal::default_host().output_devices() {
            for device in devices {
                if device.name().ok().as_deref() == Some(selected) {
                    if let Some(pair) = open_device(&device) {
                        return Some(pair);
                    }
                }
            }
        }
    }

    for device in ranked_devices() {
        if let Some(pair) = open_device(&device) {
            return Some(pair);
        }
    }

    rodio::OutputStream::try_default().ok()
}

pub fn preferred_output_name() -> Option<String> {
    use rodio::cpal::traits::DeviceTrait;

    ranked_devices()
        .first()
        .and_then(|device| device.name().ok())
}

fn ranked_devices() -> Vec<rodio::cpal::Device> {
    use rodio::cpal::traits::{DeviceTrait, HostTrait};

    let host = rodio::cpal::default_host();
    let default_name = host
        .default_output_device()
        .and_then(|device| device.name().ok())
        .unwrap_or_default();
    let key = default_name
        .split('(')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();

    let mut candidates: Vec<(i32, rodio::cpal::Device)> = Vec::new();
    if let Ok(devices) = host.output_devices() {
        for device in devices {
            let name = device.name().unwrap_or_default();
            let lower = name.to_ascii_lowercase();
            if lower.contains("steam") || lower.contains("voicemod") || lower.contains("virtual") {
                continue;
            }

            let channels = device
                .default_output_config()
                .map(|config| config.channels())
                .unwrap_or(0);

            let mut score = 0;
            if name == default_name {
                score += 100;
            }
            if channels <= 2 {
                score += 50;
            }
            if channels > 2 {
                score -= 80;
            }
            if !key.is_empty() && lower.contains(&key) {
                score += 20;
            }
            if lower.contains("realtek") {
                score += 10;
            }
            if lower.contains("headphone") {
                score += 120;
            }

            candidates.push((score, device));
        }
    }
    candidates.sort_by(|a, b| b.0.cmp(&a.0));
    candidates.into_iter().map(|(_, device)| device).collect()
}

fn open_device(
    device: &rodio::cpal::Device,
) -> Option<(rodio::OutputStream, rodio::OutputStreamHandle)> {
    use rodio::cpal::traits::DeviceTrait;

    if let Ok(ranges) = device.supported_output_configs() {
        let ranges: Vec<_> = ranges.collect();
        for channels in [2u16, 1u16] {
            for format in [
                rodio::cpal::SampleFormat::F32,
                rodio::cpal::SampleFormat::I16,
                rodio::cpal::SampleFormat::I32,
                rodio::cpal::SampleFormat::U8,
            ] {
                let range = match ranges
                    .iter()
                    .find(|range| range.channels() == channels && range.sample_format() == format)
                {
                    Some(range) => range.clone(),
                    None => continue,
                };
                let rate = if range.min_sample_rate().0 <= 48_000
                    && 48_000 <= range.max_sample_rate().0
                {
                    48_000
                } else {
                    range.max_sample_rate().0
                };
                let config = range.with_sample_rate(rodio::cpal::SampleRate(rate));
                if let Ok(pair) = rodio::OutputStream::try_from_device_config(device, config) {
                    eprintln!(
                        "ticky: output {:?} {} ch {} Hz {:?}",
                        device.name(),
                        channels,
                        rate,
                        format
                    );
                    return Some(pair);
                }
            }
        }
    }

    match rodio::OutputStream::try_from_device(device) {
        Ok(pair) => {
            eprintln!("ticky: output {:?} default config", device.name());
            Some(pair)
        }
        Err(err) => {
            eprintln!("ticky: output {:?} failed: {err}", device.name());
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn pack_wav_decodes_to_samples() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets/packs/classic-office/samples/down-01.wav");
        let bytes = std::fs::read(&path).unwrap();
        let samples: Vec<i16> = rodio::Decoder::new(Cursor::new(bytes))
            .unwrap()
            .collect();
        let peak = samples
            .iter()
            .fold(0_i16, |peak, sample| peak.max(sample.abs()));
        assert!(!samples.is_empty());
        assert!(peak > 300, "peak was {peak}");
    }

    #[test]
    fn default_output_device() {
        use rodio::cpal::traits::{DeviceTrait, HostTrait};

        let host = rodio::cpal::default_host();
        if let Ok(devices) = host.output_devices() {
            for device in devices {
                println!("--- device: {:?}", device.name());
                println!("    default: {:?}", device.default_output_config());
                if let Ok(ranges) = device.supported_output_configs() {
                    for range in ranges {
                        println!(
                            "    range: {} ch {:?} {}..{}",
                            range.channels(),
                            range.sample_format(),
                            range.min_sample_rate().0,
                            range.max_sample_rate().0
                        );
                    }
                }
            }
        }
    }
}
