use crate::state::{Language, Page};

mod en;
mod zh;

use en::EN;
use zh::ZH;

pub struct Strings {
    pub app_name: &'static str,
    pub menu_profiles: &'static str,
    pub menu_proxy_groups: &'static str,
    pub menu_rules: &'static str,
    pub menu_settings: &'static str,
    pub profiles_title: &'static str,
    pub profiles_desc: &'static str,
    pub profiles_import_title: &'static str,
    pub profiles_import_hint: &'static str,
    pub profiles_import_url_label: &'static str,
    pub profiles_import_url_placeholder: &'static str,
    pub profiles_import_button: &'static str,
    pub profiles_current_title: &'static str,
    pub profiles_current_name: &'static str,
    pub profiles_current_source: &'static str,
    pub profiles_current_updated: &'static str,
    pub profiles_current_stats: &'static str,
    pub profiles_current_empty: &'static str,
    pub profiles_list_title: &'static str,
    pub profiles_list_empty: &'static str,
    pub profiles_action_activate: &'static str,
    pub profiles_action_refresh: &'static str,
    pub profiles_action_delete: &'static str,
    pub profiles_status_active: &'static str,
    pub profiles_status_inactive: &'static str,
    pub profiles_import_ready: &'static str,
    pub profiles_import_loading: &'static str,
    pub profiles_import_success: &'static str,
    pub profiles_import_error: &'static str,
    pub proxy_groups_title: &'static str,
    pub proxy_groups_desc: &'static str,
    pub proxy_groups_empty: &'static str,
    pub proxy_groups_count_prefix: &'static str,
    pub proxy_groups_expand: &'static str,
    pub proxy_groups_collapse: &'static str,
    pub proxy_groups_members_prefix: &'static str,
    pub proxy_groups_selected_prefix: &'static str,
    pub proxy_groups_protocol_prefix: &'static str,
    pub proxy_groups_protocol_unknown: &'static str,
    pub proxy_groups_udp_tag: &'static str,
    pub proxy_groups_open: &'static str,
    pub proxy_groups_opened: &'static str,
    pub proxy_groups_active_group_prefix: &'static str,
    pub proxy_groups_active_group_empty: &'static str,
    pub proxy_groups_proxy_empty: &'static str,
    pub proxy_groups_proxy_use: &'static str,
    pub proxy_groups_proxy_selected: &'static str,
    pub proxy_groups_proxy_latency_suffix: &'static str,
    pub proxy_groups_proxy_overflow_prefix: &'static str,
    pub rules_title: &'static str,
    pub rules_desc: &'static str,
    pub rules_empty: &'static str,
    pub rules_count_prefix: &'static str,
    pub rules_search_placeholder: &'static str,
    pub rules_filter_all: &'static str,
    pub rules_filter_domain: &'static str,
    pub rules_filter_ip_cidr: &'static str,
    pub rules_filter_process_name: &'static str,
    pub settings_title: &'static str,
    pub settings_desc: &'static str,
    pub basic_setting_title: &'static str,
    pub system_setting_title: &'static str,
    pub clash_setting_title: &'static str,
    pub language_label: &'static str,
    pub theme_label: &'static str,
    pub system_proxy_label: &'static str,
    pub auto_launch_label: &'static str,
    pub silent_start_label: &'static str,
    pub clash_port_label: &'static str,
    pub clash_core_version_label: &'static str,
    pub clash_core_path_label: &'static str,
    pub clash_port_save_button: &'static str,
    pub clash_port_update_success: &'static str,
    pub clash_port_update_invalid: &'static str,
    pub clash_port_update_failed_prefix: &'static str,
    pub system_proxy_enable_success: &'static str,
    pub system_proxy_disable_success: &'static str,
    pub system_proxy_enable_failed_prefix: &'static str,
    pub system_proxy_disable_failed_prefix: &'static str,
}

pub fn strings(language: Language) -> &'static Strings {
    match language {
        Language::English => &EN,
        Language::SimplifiedChinese => &ZH,
    }
}

pub fn page_title(strings: &Strings, page: Page) -> &'static str {
    match page {
        Page::Profiles => strings.profiles_title,
        Page::ProxyGroups => strings.proxy_groups_title,
        Page::Rules => strings.rules_title,
        Page::Settings => strings.settings_title,
    }
}

pub fn page_description(strings: &Strings, page: Page) -> &'static str {
    match page {
        Page::Profiles => strings.profiles_desc,
        Page::ProxyGroups => strings.proxy_groups_desc,
        Page::Rules => strings.rules_desc,
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
