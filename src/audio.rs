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
    pub fn spawn() -> Self {
        let (tx, rx) = mpsc::channel();
        let _ = thread::Builder::new()
            .name("ticky-audio".into())
            .spawn(move || audio_loop(rx));
        Self { tx }
    }

    pub fn play(&self, bytes: Arc<[u8]>, volume: f32) {
        let _ = self.tx.send(AudioCommand::Play { bytes, volume });
    }

    pub fn shutdown(&self) {
        let _ = self.tx.send(AudioCommand::Shutdown);
    }
}

fn audio_loop(rx: Receiver<AudioCommand>) {
    let (stream, handle) = match rodio::OutputStream::try_default() {
        Ok(pair) => pair,
        Err(err) => {
            eprintln!("ticky: audio output unavailable: {err}");
            return;
        }
    };
    eprintln!("ticky: audio output ready");

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

        let device = rodio::cpal::default_host()
            .default_output_device()
            .expect("no default output device");
        println!("device: {:?}", device.name());
        println!("config: {:?}", device.default_output_config());
    }
}
