use base64::Engine as _;
use base64::engine::general_purpose;
use chrono::Local;
use percent_encoding::percent_decode_str;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub type CoreResult<T> = Result<T, CoreError>;

#[derive(Debug)]
pub enum CoreError {
    AlreadyRunning,
    NotRunning,
    ProfileNotFound,
    InvalidConfig(String),
    InvalidProfile(String),
    Network(String),
    Parse(String),
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreError::AlreadyRunning => write!(f, "core already running"),
            CoreError::NotRunning => write!(f, "core not running"),
            CoreError::ProfileNotFound => write!(f, "profile not found"),
            CoreError::InvalidConfig(msg) => write!(f, "invalid config: {msg}"),
            CoreError::InvalidProfile(msg) => write!(f, "invalid profile: {msg}"),
            CoreError::Network(msg) => write!(f, "network error: {msg}"),
            CoreError::Parse(msg) => write!(f, "parse error: {msg}"),
        }
    }
}

impl std::error::Error for CoreError {}

#[derive(Clone, Debug)]
pub struct Core {
    inner: Arc<Mutex<CoreState>>,
}

#[derive(Clone, Debug, Default)]
struct CoreState {
    running: bool,
    config: Config,
    profiles: Vec<Profile>,
}

impl Default for Core {
    fn default() -> Self {
        Self::new()
    }
}

impl Core {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(CoreState::default())),
        }
    }

    pub fn start(&self) -> CoreResult<()> {
        let mut state = self.inner.lock().expect("core state poisoned");
        if state.running {
            return Err(CoreError::AlreadyRunning);
        }
        state.running = true;
        Ok(())
    }

    pub fn stop(&self) -> CoreResult<()> {
        let mut state = self.inner.lock().expect("core state poisoned");
        if !state.running {
            return Err(CoreError::NotRunning);
        }
        state.running = false;
        Ok(())
    }

    pub fn restart(&self) -> CoreResult<()> {
        let _ = self.stop();
        self.start()
    }

    pub fn is_running(&self) -> bool {
        let state = self.inner.lock().expect("core state poisoned");
        state.running
    }

    pub fn config(&self) -> Config {
        let state = self.inner.lock().expect("core state poisoned");
        state.config.clone()
    }

    pub fn update_config(&self, config: Config) -> CoreResult<()> {
        let mut state = self.inner.lock().expect("core state poisoned");
        state.config = config;
        Ok(())
    }

    pub fn import_profile_url(&self, source_url: &str, set_active: bool) -> CoreResult<Profile> {
        let content = fetch_profile_content(source_url)?;
        let parsed = parse_profile_yaml(source_url, &content)?;

        let mut state = self.inner.lock().expect("core state poisoned");
        let mut profile = Profile {
            id: build_profile_id(source_url),
            name: parsed.name,
            source_url: source_url.to_string(),
            updated_at: current_local_timestamp(),
            node_count: parsed.node_count,
            group_count: parsed.group_count,
            rule_count: parsed.rule_count,
            active: set_active,
            proxy_groups: parsed.proxy_groups,
            proxy_nodes: parsed.proxy_nodes,
            rules: parsed.rules,
            raw_yaml: content,
        };

        if let Some(index) = state
            .profiles
            .iter()
            .position(|item| item.source_url == source_url)
        {
            profile.id = state.profiles[index].id.clone();
            state.profiles[index] = profile.clone();
        } else {
            state.profiles.insert(0, profile.clone());
        }

        if set_active || state.profiles.iter().all(|item| !item.active) {
            for item in &mut state.profiles {
                item.active = item.id == profile.id;
            }
        }

        state
            .profiles
            .iter()
            .find(|item| item.id == profile.id)
            .cloned()
            .ok_or(CoreError::ProfileNotFound)
    }

    pub fn refresh_profile(&self, id: &str) -> CoreResult<Profile> {
        let existing = {
            let state = self.inner.lock().expect("core state poisoned");
            state
                .profiles
                .iter()
                .find(|profile| profile.id == id)
                .cloned()
                .ok_or(CoreError::ProfileNotFound)?
        };

        let content = fetch_profile_content(&existing.source_url)?;
        let parsed = parse_profile_yaml(&existing.source_url, &content)?;

        let mut state = self.inner.lock().expect("core state poisoned");
        let index = state
            .profiles
            .iter()
            .position(|profile| profile.id == id)
            .ok_or(CoreError::ProfileNotFound)?;

        state.profiles[index] = Profile {
            id: existing.id.clone(),
            name: parsed.name,
            source_url: existing.source_url.clone(),
            updated_at: current_local_timestamp(),
            node_count: parsed.node_count,
            group_count: parsed.group_count,
            rule_count: parsed.rule_count,
            active: existing.active,
            proxy_groups: parsed.proxy_groups,
            proxy_nodes: parsed.proxy_nodes,
            rules: parsed.rules,
            raw_yaml: content,
        };

        Ok(state.profiles[index].clone())
    }

    pub fn delete_profile(&self, id: &str) -> CoreResult<()> {
        let mut state = self.inner.lock().expect("core state poisoned");
        let index = state
            .profiles
            .iter()
            .position(|profile| profile.id == id)
            .ok_or(CoreError::ProfileNotFound)?;

        let removed_active = state.profiles[index].active;
        state.profiles.remove(index);
        if removed_active {
            if let Some(first) = state.profiles.first_mut() {
                first.active = true;
            }
        }
        Ok(())
    }

    pub fn profiles(&self) -> Vec<Profile> {
        let state = self.inner.lock().expect("core state poisoned");
        state.profiles.clone()
    }

    pub fn active_profile(&self) -> Option<Profile> {
        let state = self.inner.lock().expect("core state poisoned");
        state
            .profiles
            .iter()
            .find(|profile| profile.active)
            .cloned()
    }

    pub fn active_proxy_groups(&self) -> Vec<ProxyGroup> {
        self.active_profile()
            .map(|profile| profile.proxy_groups)
            .unwrap_or_default()
    }

    pub fn active_proxy_nodes(&self) -> Vec<ProxyNode> {
        self.active_profile()
            .map(|profile| profile.proxy_nodes)
            .unwrap_or_default()
    }

    pub fn active_rules(&self) -> Vec<String> {
        self.active_profile()
            .map(|profile| profile.rules)
            .unwrap_or_default()
    }

    pub fn set_active_profile(&self, id: &str) -> CoreResult<()> {
        let mut state = self.inner.lock().expect("core state poisoned");
        let mut found = false;
        for profile in &mut state.profiles {
            if profile.id == id {
                profile.active = true;
                found = true;
            } else {
                profile.active = false;
            }
        }
        if found {
            Ok(())
        } else {
            Err(CoreError::ProfileNotFound)
        }
    }

    pub fn replace_profiles(&self, mut profiles: Vec<Profile>) {
        if profiles.is_empty() {
            let mut state = self.inner.lock().expect("core state poisoned");
            state.profiles.clear();
            return;
        }

        let mut found_active = false;
        for profile in &mut profiles {
            if profile.active {
                if found_active {
                    profile.active = false;
                } else {
                    found_active = true;
                }
            }
        }
        if !found_active {
            if let Some(first) = profiles.first_mut() {
                first.active = true;
            }
        }

        let mut state = self.inner.lock().expect("core state poisoned");
        state.profiles = profiles;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub mode: ProxyMode,
    pub mixed_port: u16,
    pub allow_lan: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: ProxyMode::Rule,
            mixed_port: 7890,
            allow_lan: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProxyMode {
    Rule,
    Global,
    Direct,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub source_url: String,
    pub updated_at: String,
    pub node_count: usize,
    pub group_count: usize,
    pub rule_count: usize,
    pub active: bool,
    #[serde(default)]
    pub proxy_groups: Vec<ProxyGroup>,
    #[serde(default)]
    pub proxy_nodes: Vec<ProxyNode>,
    #[serde(default)]
    pub rules: Vec<String>,
    #[serde(default, skip_serializing)]
    pub raw_yaml: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProxyGroup {
    pub name: String,
    pub kind: String,
    pub size: usize,
    pub proxies: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProxyNode {
    pub name: String,
    pub kind: String,
    pub udp: bool,
}

#[derive(Debug)]
struct ParsedProfile {
    name: String,
    node_count: usize,
    group_count: usize,
    rule_count: usize,
    proxy_groups: Vec<ProxyGroup>,
    proxy_nodes: Vec<ProxyNode>,
    rules: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RawProfileDoc {
    #[serde(default)]
    proxies: Vec<RawProxy>,
    #[serde(default, rename = "proxy-groups")]
    proxy_groups: Vec<RawProxyGroup>,
    #[serde(default)]
    rules: Vec<serde_yaml::Value>,
}

#[derive(Debug, Deserialize)]
struct RawProxy {
    #[serde(default)]
    name: String,
    #[serde(default, rename = "type")]
    kind: String,
    #[serde(default)]
    udp: bool,
}

#[derive(Debug, Deserialize)]
struct RawProxyGroup {
    #[serde(default)]
    name: String,
    #[serde(default, rename = "type")]
    kind: String,
    #[serde(default)]
    proxies: Vec<String>,
}

#[derive(Debug, Clone)]
struct SubscriptionNode {
    name: String,
    kind: String,
    udp: bool,
}

fn fetch_profile_content(source_url: &str) -> CoreResult<String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|error| CoreError::Network(error.to_string()))?;

    let mut last_body: Option<String> = None;
    let mut last_error: Option<CoreError> = None;

    for user_agent in ["linkpad/0.1.0", "clash-verge/2.4.0"] {
        match fetch_profile_content_once(&client, source_url, user_agent) {
            Ok(body) => {
                if looks_like_clash_yaml(&body) {
                    return Ok(body);
                }
                last_body = Some(body);
            }
            Err(error) => {
                last_error = Some(error);
            }
        }
    }

    if let Some(body) = last_body {
        Ok(body)
    } else {
        Err(last_error.unwrap_or_else(|| CoreError::Network("failed to fetch profile".to_string())))
    }
}

fn fetch_profile_content_once(
    client: &reqwest::blocking::Client,
    source_url: &str,
    user_agent: &str,
) -> CoreResult<String> {
    let response = client
        .get(source_url)
        .header(reqwest::header::USER_AGENT, user_agent)
        .header(
            reqwest::header::ACCEPT,
            "application/yaml,text/yaml,text/plain,*/*",
        )
        .send()
        .map_err(|error| CoreError::Network(error.to_string()))?;

    let status = response.status();
    if !status.is_success() {
        return Err(CoreError::Network(format!(
            "http status {}",
            status.as_u16()
        )));
    }

    response
        .text()
        .map_err(|error| CoreError::Network(error.to_string()))
}

fn looks_like_clash_yaml(content: &str) -> bool {
    let trimmed = content.trim_start_matches('\u{feff}').trim_start();
    trimmed.contains("proxies:")
        || trimmed.contains("proxy-groups:")
        || trimmed.starts_with("mixed-port:")
        || trimmed.starts_with("port:")
        || trimmed.starts_with("mode:")
}

fn parse_profile_yaml(source_url: &str, content: &str) -> CoreResult<ParsedProfile> {
    if let Ok(parsed) = parse_clash_yaml_profile(source_url, content) {
        return Ok(parsed);
    }

    parse_subscription_profile(source_url, content)
}

fn parse_clash_yaml_profile(source_url: &str, content: &str) -> CoreResult<ParsedProfile> {
    let parsed: RawProfileDoc =
        serde_yaml::from_str(content).map_err(|error| CoreError::Parse(error.to_string()))?;

    if parsed.proxies.is_empty() {
        return Err(CoreError::InvalidProfile(
            "missing `proxies` section or empty proxies".to_string(),
        ));
    }

    let proxy_nodes = collect_proxy_nodes(&parsed.proxies);
    let all_proxy_names = proxy_nodes
        .iter()
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();

    let mut groups: Vec<ProxyGroup> = parsed
        .proxy_groups
        .iter()
        .filter(|group| !group.name.trim().is_empty())
        .map(|group| ProxyGroup {
            name: group.name.clone(),
            kind: if group.kind.trim().is_empty() {
                "select".to_string()
            } else {
                group.kind.clone()
            },
            size: group.proxies.len(),
            proxies: group.proxies.clone(),
        })
        .collect();

    if groups.is_empty() {
        groups.push(ProxyGroup {
            name: "default".to_string(),
            kind: "select".to_string(),
            size: parsed.proxies.len(),
            proxies: all_proxy_names,
        });
    }

    let name = profile_name_from_source(source_url, &parsed);

    let rules = collect_rules(&parsed.rules);

    Ok(ParsedProfile {
        name,
        node_count: parsed.proxies.len(),
        group_count: groups.len(),
        rule_count: rules.len(),
        proxy_groups: groups,
        proxy_nodes,
        rules,
    })
}

fn parse_subscription_profile(source_url: &str, content: &str) -> CoreResult<ParsedProfile> {
    let mut candidates = vec![content.to_string()];
    if let Some(decoded) = decode_subscription_payload(content) {
        if decoded.trim() != content.trim() {
            candidates.push(decoded);
        }
    }

    for candidate in candidates {
        if let Some(parsed) = parse_subscription_text(source_url, &candidate) {
            return Ok(parsed);
        }
    }

    Err(CoreError::InvalidProfile(
        "unsupported profile format".to_string(),
    ))
}

fn parse_subscription_text(source_url: &str, text: &str) -> Option<ParsedProfile> {
    let nodes = extract_subscription_nodes(text);
    if nodes.is_empty() {
        return None;
    }

    let mut grouped: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for node in &nodes {
        grouped
            .entry(node.kind.clone())
            .or_default()
            .push(node.name.clone());
    }

    let all_proxies = nodes
        .iter()
        .map(|node| node.name.clone())
        .collect::<Vec<_>>();
    let proxy_nodes = nodes
        .iter()
        .map(|node| ProxyNode {
            name: node.name.clone(),
            kind: normalize_kind(&node.kind),
            udp: node.udp || kind_supports_udp(&node.kind),
        })
        .collect::<Vec<_>>();

    let mut groups = Vec::with_capacity(grouped.len() + 1);
    groups.push(ProxyGroup {
        name: "All Proxies".to_string(),
        kind: "select".to_string(),
        size: nodes.len(),
        proxies: all_proxies,
    });
    for (kind, proxies) in grouped {
        groups.push(ProxyGroup {
            name: format!("{} Nodes", kind.to_uppercase()),
            kind: kind.clone(),
            size: proxies.len(),
            proxies,
        });
    }

    Some(ParsedProfile {
        name: profile_name_from_subscription(source_url, &nodes),
        node_count: nodes.len(),
        group_count: groups.len(),
        rule_count: 0,
        proxy_groups: groups,
        proxy_nodes,
        rules: Vec::new(),
    })
}

fn extract_subscription_nodes(text: &str) -> Vec<SubscriptionNode> {
    let mut nodes = Vec::new();
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') || !looks_like_proxy_uri(line) {
            continue;
        }

        let kind = line
            .split("://")
            .next()
            .unwrap_or("proxy")
            .to_ascii_lowercase();
        let name = parse_subscription_node_name(line)
            .unwrap_or_else(|| format!("{}-{}", kind.to_uppercase(), nodes.len() + 1));
        let udp = kind_supports_udp(&kind);
        nodes.push(SubscriptionNode { name, kind, udp });
    }
    nodes
}

fn parse_subscription_node_name(line: &str) -> Option<String> {
    if let Ok(url) = url::Url::parse(line) {
        if let Some(fragment) = url.fragment() {
            let name = percent_decode_str(fragment).decode_utf8_lossy();
            let trimmed = name.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    if let Some((_, fragment)) = line.split_once('#') {
        let name = percent_decode_str(fragment).decode_utf8_lossy();
        let trimmed = name.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    None
}

fn looks_like_proxy_uri(line: &str) -> bool {
    let Some((scheme, _)) = line.split_once("://") else {
        return false;
    };
    matches!(
        scheme.to_ascii_lowercase().as_str(),
        "ss" | "ssr"
            | "trojan"
            | "vmess"
            | "vless"
            | "tuic"
            | "hysteria"
            | "hysteria2"
            | "hy2"
            | "anytls"
            | "http"
            | "https"
            | "socks5"
            | "wireguard"
    )
}

fn decode_subscription_payload(content: &str) -> Option<String> {
    let compact: String = content.lines().map(str::trim).collect();
    let compact = compact.trim();
    if compact.is_empty() || !looks_like_base64_payload(compact) {
        return None;
    }

    for engine in [
        &general_purpose::STANDARD,
        &general_purpose::STANDARD_NO_PAD,
        &general_purpose::URL_SAFE,
        &general_purpose::URL_SAFE_NO_PAD,
    ] {
        let Ok(bytes) = engine.decode(compact) else {
            continue;
        };
        let Ok(decoded) = String::from_utf8(bytes) else {
            continue;
        };
        if decoded.contains("://") {
            return Some(decoded);
        }
    }
    None
}

fn looks_like_base64_payload(text: &str) -> bool {
    if text.len() < 24 || text.contains("://") {
        return false;
    }
    text.bytes().all(|byte| {
        byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/' | b'=' | b'-' | b'_')
    })
}

fn profile_name_from_source(source_url: &str, parsed: &RawProfileDoc) -> String {
    if let Some(group) = parsed.proxy_groups.first() {
        let trimmed = group.name.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    if let Some(proxy) = parsed.proxies.first() {
        let trimmed = proxy.name.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    if let Ok(url) = url::Url::parse(source_url) {
        if let Some(host) = url.host_str() {
            return host.to_string();
        }
    }

    "imported-profile".to_string()
}

fn collect_proxy_nodes(proxies: &[RawProxy]) -> Vec<ProxyNode> {
    proxies
        .iter()
        .enumerate()
        .map(|(index, proxy)| {
            let trimmed = proxy.name.trim();
            let name = if trimmed.is_empty() {
                format!("Proxy {}", index + 1)
            } else {
                trimmed.to_string()
            };
            let kind = normalize_kind(&proxy.kind);
            let udp = proxy.udp || kind_supports_udp(&proxy.kind);
            ProxyNode { name, kind, udp }
        })
        .collect()
}

fn collect_rules(values: &[serde_yaml::Value]) -> Vec<String> {
    values
        .iter()
        .filter_map(|value| {
            if let Some(rule) = value.as_str() {
                let trimmed = rule.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            } else {
                let rendered = serde_yaml::to_string(value).ok()?;
                let trimmed = rendered.trim().replace('\n', " ");
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                }
            }
        })
        .collect()
}

fn normalize_kind(kind: &str) -> String {
    let trimmed = kind.trim();
    if trimmed.is_empty() {
        "proxy".to_string()
    } else {
        trimmed.to_ascii_lowercase()
    }
}

fn kind_supports_udp(kind: &str) -> bool {
    matches!(
        kind.to_ascii_lowercase().as_str(),
        "tuic" | "hysteria" | "hysteria2" | "hy2" | "wireguard"
    )
}

fn profile_name_from_subscription(source_url: &str, nodes: &[SubscriptionNode]) -> String {
    if let Ok(url) = url::Url::parse(source_url) {
        if let Some(host) = url.host_str() {
            return host.to_string();
        }
    }

    if let Some(first) = nodes.first() {
        let trimmed = first.name.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    "imported-subscription".to_string()
}

fn build_profile_id(source_url: &str) -> String {
    let seconds = now_unix_seconds();
    let mut hasher = DefaultHasher::new();
    source_url.hash(&mut hasher);
    format!("p-{seconds}-{:x}", hasher.finish() & 0xffff_ffff)
}

fn now_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_secs())
        .unwrap_or(0)
}

fn current_local_timestamp() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_clash_yaml_profile() {
        let content = r#"
mixed-port: 7890
mode: rule
proxies:
  - { name: "node-1", type: ss, server: "example.com", port: 443, cipher: aes-128-gcm, password: "pwd" }
proxy-groups:
  - { name: "auto", type: select, proxies: ["node-1"] }
rules:
  - MATCH,auto
"#;

        let parsed = parse_profile_yaml("https://example.com/sub.yaml", content)
            .expect("should parse clash yaml");
        assert_eq!(parsed.node_count, 1);
        assert_eq!(parsed.group_count, 1);
        assert_eq!(parsed.rule_count, 1);
        assert_eq!(parsed.proxy_groups[0].proxies, vec!["node-1".to_string()]);
        assert_eq!(parsed.rules, vec!["MATCH,auto".to_string()]);
    }

    #[test]
    fn parses_base64_subscription_profile() {
        let plain = "ss://YWVzLTEyOC1nY206cGFzcw@example.com:443#Node%201\ntrojan://pass@example.com:443#Node%202\n";
        let encoded = general_purpose::STANDARD.encode(plain);

        let parsed = parse_profile_yaml("https://example.com/sub", &encoded)
            .expect("should parse base64 subscription");
        assert_eq!(parsed.node_count, 2);
        assert!(parsed.group_count >= 1);
        assert_eq!(parsed.rule_count, 0);
        assert_eq!(parsed.proxy_groups[0].proxies.len(), 2);
        assert!(parsed.rules.is_empty());
    }
}
