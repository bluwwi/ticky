use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Clone, Debug, Default)]
pub struct Sounds {
    #[serde(default)]
    pub key_down: Vec<String>,
    #[serde(default)]
    pub key_up: Vec<String>,
    #[serde(default)]
    pub enter: Vec<String>,
    #[serde(default)]
    pub backspace: Vec<String>,
    #[serde(default)]
    pub mouse_left: Vec<String>,
    #[serde(default)]
    pub mouse_right: Vec<String>,
    #[serde(default)]
    pub mouse_middle: Vec<String>,
    #[serde(default)]
    pub mouse_back: Vec<String>,
    #[serde(default)]
    pub mouse_forward: Vec<String>,
    #[serde(default)]
    pub mouse_wheel_up: Vec<String>,
    #[serde(default)]
    pub mouse_wheel_down: Vec<String>,
    #[serde(default)]
    pub mouse_wheel_left: Vec<String>,
    #[serde(default)]
    pub mouse_wheel_right: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct PackManifest {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub sounds: Sounds,
}

#[derive(Clone, Debug)]
pub struct Pack {
    pub manifest: PackManifest,
    pub dir: PathBuf,
}

pub fn packs_dir() -> PathBuf {
    let mut candidates = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("assets").join("packs"));
            candidates.push(dir.join("packs"));
        }
    }
    candidates.push(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("packs"),
    );

    candidates
        .into_iter()
        .find(|path| path.is_dir())
        .unwrap_or_else(|| PathBuf::from("assets").join("packs"))
}

pub fn load_packs() -> Vec<Pack> {
    let mut packs = Vec::new();
    let Ok(entries) = std::fs::read_dir(packs_dir()) else {
        return packs;
    };

    for entry in entries.flatten() {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        let manifest_path = dir.join("pack.toml");
        let Ok(text) = std::fs::read_to_string(&manifest_path) else {
            continue;
        };
        match toml::from_str::<PackManifest>(&text) {
            Ok(manifest) => packs.push(Pack { manifest, dir }),
            Err(err) => eprintln!("Ignoring {}: {err}", manifest_path.display()),
        }
    }

    packs.sort_by(|a, b| a.manifest.name.to_lowercase().cmp(&b.manifest.name.to_lowercase()));
    packs
}
