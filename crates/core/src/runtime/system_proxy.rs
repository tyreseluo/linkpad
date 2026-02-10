use crate::{CoreError, CoreResult};

#[derive(Debug, Default)]
pub struct SystemProxyManager {
    #[cfg(target_os = "macos")]
    snapshot: Option<SystemProxySnapshot>,
}

impl SystemProxyManager {
    pub fn enable(&mut self, host: &str, port: u16) -> CoreResult<()> {
        #[cfg(target_os = "macos")]
        {
            if self.snapshot.is_none() {
                self.snapshot = Some(capture_snapshot()?);
            }

            if let Err(error) = apply_proxy_for_all_services(host, port) {
                if let Some(snapshot) = self.snapshot.as_ref() {
                    let _ = restore_snapshot(snapshot);
                }
                return Err(error);
            }
            return Ok(());
        }

        #[cfg(target_os = "windows")]
        {
            return enable_windows_proxy(host, port);
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            let _ = host;
            let _ = port;
            Err(CoreError::InvalidConfig(
                "system proxy manager is not implemented for this platform".to_string(),
            ))
        }
    }

    pub fn disable(&mut self) -> CoreResult<()> {
        #[cfg(target_os = "macos")]
        {
            disable_proxy_for_all_services()?;
            self.snapshot = None;
            return Ok(());
        }

        #[cfg(target_os = "windows")]
        {
            return disable_windows_proxy();
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            Err(CoreError::InvalidConfig(
                "system proxy manager is not implemented for this platform".to_string(),
            ))
        }
    }
}

#[cfg(target_os = "windows")]
const WINDOWS_PROXY_REG_PATH: &str =
    r"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings";

#[cfg(target_os = "windows")]
fn enable_windows_proxy(host: &str, port: u16) -> CoreResult<()> {
    let proxy_server = format!("{host}:{port}");
    run_reg(&[
        "add",
        WINDOWS_PROXY_REG_PATH,
        "/v",
        "ProxyServer",
        "/t",
        "REG_SZ",
        "/d",
        proxy_server.as_str(),
        "/f",
    ])?;
    run_reg(&[
        "add",
        WINDOWS_PROXY_REG_PATH,
        "/v",
        "ProxyEnable",
        "/t",
        "REG_DWORD",
        "/d",
        "1",
        "/f",
    ])?;
    notify_windows_proxy_changed();
    Ok(())
}

#[cfg(target_os = "windows")]
fn disable_windows_proxy() -> CoreResult<()> {
    run_reg(&[
        "add",
        WINDOWS_PROXY_REG_PATH,
        "/v",
        "ProxyEnable",
        "/t",
        "REG_DWORD",
        "/d",
        "0",
        "/f",
    ])?;
    notify_windows_proxy_changed();
    Ok(())
}

#[cfg(target_os = "windows")]
fn notify_windows_proxy_changed() {
    use std::ptr::null;
    use windows_sys::Win32::Networking::WinInet::{
        INTERNET_OPTION_REFRESH, INTERNET_OPTION_SETTINGS_CHANGED, InternetSetOptionW,
    };

    unsafe {
        let _ = InternetSetOptionW(null(), INTERNET_OPTION_SETTINGS_CHANGED, null(), 0);
        let _ = InternetSetOptionW(null(), INTERNET_OPTION_REFRESH, null(), 0);
    }
}

#[cfg(target_os = "windows")]
fn run_reg(args: &[&str]) -> CoreResult<String> {
    let output = run_reg_command(args)?;
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
    Err(CoreError::InvalidConfig(format!(
        "reg {} failed: {}",
        args.join(" "),
        reason
    )))
}

#[cfg(target_os = "windows")]
fn run_reg_command(args: &[&str]) -> CoreResult<std::process::Output> {
    use std::process::Command;

    let mut command = Command::new("reg");
    configure_windows_hidden_command(&mut command);
    command
        .args(args)
        .output()
        .map_err(|error| CoreError::InvalidConfig(error.to_string()))
}

#[cfg(target_os = "windows")]
fn configure_windows_hidden_command(command: &mut std::process::Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(target_os = "macos")]
#[derive(Clone, Debug)]
struct SystemProxySnapshot {
    services: Vec<ServiceProxyState>,
}

#[cfg(target_os = "macos")]
#[derive(Clone, Debug)]
struct ServiceProxyState {
    name: String,
    web: ProxyState,
    secure_web: ProxyState,
    socks: ProxyState,
}

#[cfg(target_os = "macos")]
#[derive(Clone, Debug, Default)]
struct ProxyState {
    enabled: bool,
    server: String,
    port: u16,
}

#[cfg(target_os = "macos")]
#[derive(Clone, Copy, Debug)]
enum ProxyProtocol {
    Web,
    SecureWeb,
    Socks,
}

#[cfg(target_os = "macos")]
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

#[cfg(target_os = "macos")]
fn capture_snapshot() -> CoreResult<SystemProxySnapshot> {
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

#[cfg(target_os = "macos")]
fn restore_snapshot(snapshot: &SystemProxySnapshot) -> CoreResult<()> {
    for service in &snapshot.services {
        restore_protocol_state(&service.name, ProxyProtocol::Web, &service.web)?;
        restore_protocol_state(&service.name, ProxyProtocol::SecureWeb, &service.secure_web)?;
        restore_protocol_state(&service.name, ProxyProtocol::Socks, &service.socks)?;
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn restore_protocol_state(
    service: &str,
    protocol: ProxyProtocol,
    state: &ProxyState,
) -> CoreResult<()> {
    if state.enabled && !state.server.is_empty() && state.port > 0 {
        set_proxy(service, protocol, &state.server, state.port)?;
        set_proxy_state(service, protocol, true)?;
    } else {
        set_proxy_state(service, protocol, false)?;
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn apply_proxy_for_all_services(host: &str, port: u16) -> CoreResult<()> {
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

#[cfg(target_os = "macos")]
fn disable_proxy_for_all_services() -> CoreResult<()> {
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

#[cfg(target_os = "macos")]
fn list_active_services() -> CoreResult<Vec<String>> {
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

#[cfg(target_os = "macos")]
fn get_proxy_state(service: &str, protocol: ProxyProtocol) -> CoreResult<ProxyState> {
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

#[cfg(target_os = "macos")]
fn set_proxy(service: &str, protocol: ProxyProtocol, host: &str, port: u16) -> CoreResult<()> {
    let port_value = port.to_string();
    let _ = run_networksetup([protocol.set_cmd(), service, host, &port_value])?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn set_proxy_state(service: &str, protocol: ProxyProtocol, enabled: bool) -> CoreResult<()> {
    let state = if enabled { "on" } else { "off" };
    let _ = run_networksetup([protocol.set_state_cmd(), service, state])?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn run_networksetup<'a>(args: impl IntoIterator<Item = &'a str>) -> CoreResult<String> {
    use std::process::Command;

    let args_vec = args
        .into_iter()
        .map(|arg| arg.to_string())
        .collect::<Vec<_>>();
    let output = Command::new("networksetup")
        .args(&args_vec)
        .output()
        .map_err(|error| CoreError::InvalidConfig(error.to_string()))?;

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
    Err(CoreError::InvalidConfig(format!(
        "networksetup {} failed: {}",
        args_vec.join(" "),
        reason
    )))
}
