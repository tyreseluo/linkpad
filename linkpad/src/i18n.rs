use crate::state::{Language, Page};

pub struct Strings {
    pub app_name: &'static str,
    pub menu_profiles: &'static str,
    pub menu_settings: &'static str,
    pub profiles_title: &'static str,
    pub profiles_desc: &'static str,
    pub settings_title: &'static str,
    pub settings_desc: &'static str,
    pub basic_setting_title: &'static str,
    pub language_label: &'static str,
    pub theme_label: &'static str,
}

const EN: Strings = Strings {
    app_name: "Linkpad",
    menu_profiles: "Profiles",
    menu_settings: "Settings",
    profiles_title: "Profiles",
    profiles_desc: "Manage subscription profiles, local configs, and sync sources.",
    settings_title: "Settings",
    settings_desc: "App preferences, network options, and system integration.",
    basic_setting_title: "Linkpad Basic Setting",
    language_label: "Language",
    theme_label: "Theme",
};

const ZH: Strings = Strings {
    app_name: "Linkpad",
    menu_profiles: "配置",
    menu_settings: "设置",
    profiles_title: "配置",
    profiles_desc: "管理订阅配置、本地配置与同步来源。",
    settings_title: "设置",
    settings_desc: "应用偏好、网络选项与系统集成。",
    basic_setting_title: "Linkpad 基础设置",
    language_label: "语言",
    theme_label: "主题",
};

pub fn strings(language: Language) -> &'static Strings {
    match language {
        Language::English => &EN,
        Language::SimplifiedChinese => &ZH,
    }
}

pub fn page_title(strings: &Strings, page: Page) -> &'static str {
    match page {
        Page::Profiles => strings.profiles_title,
        Page::Settings => strings.settings_title,
    }
}

pub fn page_description(strings: &Strings, page: Page) -> &'static str {
    match page {
        Page::Profiles => strings.profiles_desc,
        Page::Settings => strings.settings_desc,
    }
}

pub fn language_options(language: Language) -> Vec<String> {
    match language {
        Language::English => vec!["English".to_string(), "简体中文".to_string()],
        Language::SimplifiedChinese => vec!["英文".to_string(), "简体中文".to_string()],
    }
}

pub fn theme_options(language: Language) -> Vec<String> {
    match language {
        Language::English => vec![
            "Light".to_string(),
            "Dark".to_string(),
            "System".to_string(),
        ],
        Language::SimplifiedChinese => vec![
            "浅色".to_string(),
            "深色".to_string(),
            "跟随系统".to_string(),
        ],
    }
}
