use crate::{SystemProxyBackend, SystemProxyConfig, SystemProxyError, SystemProxyResult};

pub(crate) fn create_backend() -> Box<dyn SystemProxyBackend> {
    Box::new(MacosSystemProxyBackend::default())
}

#[derive(Debug, Default)]
struct MacosSystemProxyBackend {
    snapshot: Option<SystemProxySnapshot>,
}

impl SystemProxyBackend for MacosSystemProxyBackend {
    fn enable(&mut self, config: &SystemProxyConfig) -> SystemProxyResult<()> {
        if self.snapshot.is_none() {
            self.snapshot = Some(capture_snapshot()?);
        }

        if let Err(error) = apply_proxy_for_all_services(&config.host, config.port) {
            if let Some(snapshot) = self.snapshot.as_ref() {
                let _ = restore_snapshot(snapshot);
            }
            return Err(error);
        }
        Ok(())
    }

    fn disable(&mut self) -> SystemProxyResult<()> {
        disable_proxy_for_all_services()?;
        self.snapshot = None;
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct SystemProxySnapshot {
    services: Vec<ServiceProxyState>,
}

#[derive(Clone, Debug)]
struct ServiceProxyState {
    name: String,
    web: ProxyState,
    secure_web: ProxyState,
    socks: ProxyState,
}

#[derive(Clone, Debug, Default)]
struct ProxyState {
    enabled: bool,
    server: String,
    port: u16,
}

#[derive(Clone, Copy, Debug)]
enum ProxyProtocol {
    Web,
    SecureWeb,
    Socks,
}

impl ProxyProtocol {
    fn get_cmd(self) -> &'static str {
        match self {
            Self::Web => "-getwebproxy",
            Self::SecureWeb => "-getsecurewebproxy",
            Self::Socks => "-getsocksfirewallproxy",
        }
    }

    fn set_cmd(self) -> &'static str {
        match self {
            Self::Web => "-setwebproxy",
            Self::SecureWeb => "-setsecurewebproxy",
            Self::Socks => "-setsocksfirewallproxy",
        }
    }

    fn set_state_cmd(self) -> &'static str {
        match self {
            Self::Web => "-setwebproxystate",
            Self::SecureWeb => "-setsecurewebproxystate",
            Self::Socks => "-setsocksfirewallproxystate",
        }
    }
}

fn capture_snapshot() -> SystemProxyResult<SystemProxySnapshot> {
    let mut services = Vec::new();
    for service in list_active_services()? {
        services.push(ServiceProxyState {
            name: service.clone(),
            web: get_proxy_state(&service, ProxyProtocol::Web)?,
            secure_web: get_proxy_state(&service, ProxyProtocol::SecureWeb)?,
            socks: get_proxy_state(&service, ProxyProtocol::Socks)?,
        });
    }
    Ok(SystemProxySnapshot { services })
}

fn restore_snapshot(snapshot: &SystemProxySnapshot) -> SystemProxyResult<()> {
    for service in &snapshot.services {
        restore_protocol_state(&service.name, ProxyProtocol::Web, &service.web)?;
        restore_protocol_state(&service.name, ProxyProtocol::SecureWeb, &service.secure_web)?;
        restore_protocol_state(&service.name, ProxyProtocol::Socks, &service.socks)?;
    }
    Ok(())
}

fn restore_protocol_state(
    service: &str,
    protocol: ProxyProtocol,
    state: &ProxyState,
) -> SystemProxyResult<()> {
    if state.enabled && !state.server.is_empty() && state.port > 0 {
        set_proxy(service, protocol, &state.server, state.port)?;
        set_proxy_state(service, protocol, true)?;
    } else {
        set_proxy_state(service, protocol, false)?;
    }
    Ok(())
}

fn apply_proxy_for_all_services(host: &str, port: u16) -> SystemProxyResult<()> {
    for service in list_active_services()? {
        for protocol in [
            ProxyProtocol::Web,
            ProxyProtocol::SecureWeb,
            ProxyProtocol::Socks,
        ] {
            set_proxy(&service, protocol, host, port)?;
            set_proxy_state(&service, protocol, true)?;
        }
    }
    Ok(())
}

fn disable_proxy_for_all_services() -> SystemProxyResult<()> {
    for service in list_active_services()? {
        for protocol in [
            ProxyProtocol::Web,
            ProxyProtocol::SecureWeb,
            ProxyProtocol::Socks,
        ] {
            set_proxy_state(&service, protocol, false)?;
        }
    }
    Ok(())
}

fn list_active_services() -> SystemProxyResult<Vec<String>> {
    let output = run_networksetup(["-listallnetworkservices"])?;
    let mut services = Vec::new();
    for line in output.lines().skip(1) {
        let service = line.trim();
        if service.is_empty() || service.starts_with('*') {
            continue;
        }
        services.push(service.to_string());
    }
    Ok(services)
}

fn get_proxy_state(service: &str, protocol: ProxyProtocol) -> SystemProxyResult<ProxyState> {
    let output = run_networksetup([protocol.get_cmd(), service])?;
    let mut state = ProxyState::default();

    for line in output.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("Enabled:") {
            state.enabled = value.trim().eq_ignore_ascii_case("Yes");
            continue;
        }
        if let Some(value) = line.strip_prefix("Server:") {
            state.server = value.trim().to_string();
            continue;
        }
        if let Some(value) = line.strip_prefix("Port:") {
            state.port = value.trim().parse::<u16>().unwrap_or(0);
        }
    }
    Ok(state)
}

fn set_proxy(
    service: &str,
    protocol: ProxyProtocol,
    host: &str,
    port: u16,
) -> SystemProxyResult<()> {
    let port_value = port.to_string();
    let _ = run_networksetup([protocol.set_cmd(), service, host, &port_value])?;
    Ok(())
}

fn set_proxy_state(service: &str, protocol: ProxyProtocol, enabled: bool) -> SystemProxyResult<()> {
    let state = if enabled { "on" } else { "off" };
    let _ = run_networksetup([protocol.set_state_cmd(), service, state])?;
    Ok(())
}

fn run_networksetup<'a>(args: impl IntoIterator<Item = &'a str>) -> SystemProxyResult<String> {
    use std::process::Command;

    let args_vec = args
        .into_iter()
        .map(|arg| arg.to_string())
        .collect::<Vec<_>>();
    let output = Command::new("networksetup")
        .args(&args_vec)
        .output()
        .map_err(|error| SystemProxyError::new(error.to_string()))?;

    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let reason = if !stderr.trim().is_empty() {
        stderr.trim().to_string()
    } else {
        stdout.trim().to_string()
    };
    Err(SystemProxyError::new(format!(
        "networksetup {} failed: {}",
        args_vec.join(" "),
        reason
    )))
}
