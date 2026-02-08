use linkpad_core::Profile;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct PersistedProfiles {
    profiles: Vec<Profile>,
}

pub fn load() -> Vec<Profile> {
    let Some(path) = profiles_path() else {
        return Vec::new();
    };
    let Ok(content) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let Ok(persisted) = serde_json::from_str::<PersistedProfiles>(&content) else {
        return Vec::new();
    };
    persisted.profiles
}

pub fn save(profiles: &[Profile]) -> std::io::Result<()> {
    let Some(path) = profiles_path() else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let persisted = PersistedProfiles {
        profiles: profiles.to_vec(),
    };
    let json = serde_json::to_string_pretty(&persisted).unwrap_or_else(|_| "{}".to_string());
    fs::write(path, json)
}

fn profiles_path() -> Option<PathBuf> {
    let mut dir = dirs::config_dir()?;
    dir.push("linkpad");
    dir.push("profiles.json");
    Some(dir)
}
