use crate::state::{Language, ThemePreference};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct LoadedSettings {
    pub language: Language,
    pub theme: ThemePreference,
    pub system_proxy_enabled: bool,
    pub auto_launch_enabled: bool,
    pub silent_start_enabled: bool,
    pub clash_mixed_port: u16,
    pub proxy_group_selections: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedSettings {
    language: String,
    theme: String,
    #[serde(default)]
    system_proxy_enabled: bool,
    #[serde(default)]
    auto_launch_enabled: bool,
    #[serde(default)]
    silent_start_enabled: bool,
    #[serde(default = "default_mixed_port")]
    clash_mixed_port: u16,
    #[serde(default)]
    proxy_group_selections: HashMap<String, String>,
}

pub fn load() -> Option<LoadedSettings> {
    let path = settings_path()?;
    let content = fs::read_to_string(path).ok()?;
    let persisted: PersistedSettings = serde_json::from_str(&content).ok()?;

    let language = parse_language(&persisted.language).unwrap_or(Language::English);
    let theme = parse_theme(&persisted.theme).unwrap_or(ThemePreference::System);
    Some(LoadedSettings {
        language,
        theme,
        system_proxy_enabled: persisted.system_proxy_enabled,
        auto_launch_enabled: persisted.auto_launch_enabled,
        silent_start_enabled: persisted.silent_start_enabled,
        clash_mixed_port: normalize_port(persisted.clash_mixed_port),
        proxy_group_selections: persisted.proxy_group_selections,
    })
}

pub fn save(
    language: Language,
    theme: ThemePreference,
    system_proxy_enabled: bool,
    auto_launch_enabled: bool,
    silent_start_enabled: bool,
    clash_mixed_port: u16,
    proxy_group_selections: &HashMap<String, String>,
) -> std::io::Result<()> {
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
        system_proxy_enabled,
        auto_launch_enabled,
        silent_start_enabled,
        clash_mixed_port: normalize_port(clash_mixed_port),
        proxy_group_selections: proxy_group_selections.clone(),
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

fn default_mixed_port() -> u16 {
    7890
}

fn normalize_port(port: u16) -> u16 {
    if port == 0 {
        default_mixed_port()
    } else {
        port
    }
}
