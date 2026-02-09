use linkpad_core::Profile;
use robius_directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct PersistedProfiles {
    profiles: Vec<Profile>,
}

pub fn load() -> Vec<Profile> {
    migrate_legacy_profiles_file();

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
    let mut dir = super::app_config_dir()?;
    dir.push("profiles.json");
    Some(dir)
}

fn migrate_legacy_profiles_file() {
    let Some(new_path) = profiles_path() else {
        return;
    };

    if new_path.exists() {
        return;
    }

    let Some(legacy_path) = legacy_profiles_path(&new_path) else {
        return;
    };

    if let Some(parent) = new_path.parent() {
        if fs::create_dir_all(parent).is_err() {
            return;
        }
    }

    let _ = fs::copy(legacy_path, new_path);
}

fn legacy_profiles_path(new_path: &PathBuf) -> Option<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(base_dirs) = BaseDirs::new() {
        let mut legacy = base_dirs.config_dir().to_path_buf();
        legacy.push("linkpad");
        legacy.push("profiles.json");
        candidates.push(legacy);
    }

    if let Some(mut legacy) = super::app_config_dir() {
        legacy.push("linkpad");
        legacy.push("profiles.json");
        candidates.push(legacy);
    }

    candidates.sort();
    candidates.dedup();
    candidates
        .into_iter()
        .find(|path| path != new_path && path.is_file())
}
