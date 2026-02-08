use crate::state::{Language, ThemePreference};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedSettings {
    language: String,
    theme: String,
}

pub fn load() -> Option<(Language, ThemePreference)> {
    let path = settings_path()?;
    let content = fs::read_to_string(path).ok()?;
    let persisted: PersistedSettings = serde_json::from_str(&content).ok()?;

    let language = parse_language(&persisted.language).unwrap_or(Language::English);
    let theme = parse_theme(&persisted.theme).unwrap_or(ThemePreference::System);
    Some((language, theme))
}

pub fn save(language: Language, theme: ThemePreference) -> std::io::Result<()> {
    let path = match settings_path() {
        Some(path) => path,
        None => return Ok(()),
    };

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let persisted = PersistedSettings {
        language: serialize_language(language).to_string(),
        theme: serialize_theme(theme).to_string(),
    };

    let json = serde_json::to_string_pretty(&persisted).unwrap_or_else(|_| "{}".to_string());
    fs::write(path, json)
}

fn settings_path() -> Option<PathBuf> {
    let mut dir = dirs::config_dir()?;
    dir.push("linkpad");
    dir.push("settings.json");
    Some(dir)
}

fn parse_language(raw: &str) -> Option<Language> {
    match raw {
        "en" | "english" => Some(Language::English),
        "zh-CN" | "zh" | "zh-cn" | "simplified_chinese" => Some(Language::SimplifiedChinese),
        _ => None,
    }
}

fn parse_theme(raw: &str) -> Option<ThemePreference> {
    match raw {
        "light" => Some(ThemePreference::Light),
        "dark" => Some(ThemePreference::Dark),
        "system" => Some(ThemePreference::System),
        _ => None,
    }
}

fn serialize_language(language: Language) -> &'static str {
    match language {
        Language::English => "en",
        Language::SimplifiedChinese => "zh-CN",
    }
}

fn serialize_theme(theme: ThemePreference) -> &'static str {
    match theme {
        ThemePreference::Light => "light",
        ThemePreference::Dark => "dark",
        ThemePreference::System => "system",
    }
}
