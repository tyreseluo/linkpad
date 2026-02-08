use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct AppState {
    pub active_page: Page,
    pub language: Language,
    pub theme: ThemePreference,
    pub profile_url_input: String,
    pub import_status: ImportStatus,
    pub profiles: Vec<ProfileSummary>,
    pub proxy_groups: Vec<ProxyGroupSummary>,
    pub proxy_nodes: Vec<ProxyNodeSummary>,
    pub rules: Vec<String>,
    pub rules_query: String,
    pub rules_filter: RuleFilter,
    pub active_proxy_group: Option<String>,
    pub proxy_group_selected: HashMap<String, usize>,
    pub system_proxy_enabled: bool,
    pub auto_launch_enabled: bool,
    pub silent_start_enabled: bool,
    pub clash_mixed_port: u16,
    pub clash_port_input: String,
    pub clash_core_version: String,
    pub clash_core_path: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Profiles,
    ProxyGroups,
    Rules,
    Settings,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    English,
    SimplifiedChinese,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemePreference {
    Light,
    Dark,
    System,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuleFilter {
    All,
    Domain,
    IpCidr,
    ProcessName,
}

#[derive(Clone, Debug)]
pub struct ImportStatus {
    pub message: String,
    pub is_error: bool,
}

#[derive(Clone, Debug)]
pub struct ProfileSummary {
    pub id: String,
    pub name: String,
    pub source: String,
    pub updated_at: String,
    pub node_count: usize,
    pub group_count: usize,
    pub rule_count: usize,
    pub active: bool,
}

#[derive(Clone, Debug)]
pub struct ProxyGroupSummary {
    pub name: String,
    pub kind: String,
    pub size: usize,
    pub proxies: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct ProxyNodeSummary {
    pub name: String,
    pub kind: String,
    pub udp: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            active_page: Page::Profiles,
            language: Language::English,
            theme: ThemePreference::System,
            profile_url_input: String::new(),
            import_status: ImportStatus {
                message: "Ready to import profile URL.".to_string(),
                is_error: false,
            },
            profiles: Vec::new(),
            proxy_groups: Vec::new(),
            proxy_nodes: Vec::new(),
            rules: Vec::new(),
            rules_query: String::new(),
            rules_filter: RuleFilter::All,
            active_proxy_group: None,
            proxy_group_selected: HashMap::new(),
            system_proxy_enabled: false,
            auto_launch_enabled: false,
            silent_start_enabled: false,
            clash_mixed_port: 7890,
            clash_port_input: "7890".to_string(),
            clash_core_version: "Unknown".to_string(),
            clash_core_path: "-".to_string(),
        }
    }
}

impl AppState {
    pub fn active_profile(&self) -> Option<&ProfileSummary> {
        self.profiles.iter().find(|profile| profile.active)
    }
}

impl Language {
    pub fn from_index(index: usize) -> Self {
        match index {
            1 => Self::SimplifiedChinese,
            _ => Self::English,
        }
    }

    pub fn as_index(self) -> usize {
        match self {
            Self::English => 0,
            Self::SimplifiedChinese => 1,
        }
    }
}

impl ThemePreference {
    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Self::Light,
            1 => Self::Dark,
            _ => Self::System,
        }
    }

    pub fn as_index(self) -> usize {
        match self {
            Self::Light => 0,
            Self::Dark => 1,
            Self::System => 2,
        }
    }
}
