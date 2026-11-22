use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Config {
    pub active_pack: String,
    pub sound_enabled: bool,
    pub muted: bool,
    pub output_device: String,
    pub autostart: Option<bool>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            active_pack: "classic-office".to_owned(),
            sound_enabled: true,
            muted: false,
            output_device: String::new(),
            autostart: None,
        }
    }
}

pub fn config_path() -> PathBuf {
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("Ticky").join("config.toml")
}

impl Config {
    pub fn load() -> Self {
        fs::read_to_string(config_path())
            .ok()
            .and_then(|text| toml::from_str(&text).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Could not create {parent:?}: {e}"))?;
        }
        let text = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        let tmp = path.with_extension("toml.tmp");
        fs::write(&tmp, text).map_err(|e| format!("Could not write {tmp:?}: {e}"))?;
        fs::rename(&tmp, &path).map_err(|e| format!("Could not save {path:?}: {e}"))?;
        Ok(())
    }
}
