use crate::{SystemProxyBackend, SystemProxyConfig, SystemProxyError, SystemProxyResult};

const WINDOWS_PROXY_REG_PATH: &str =
    r"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings";

pub(crate) fn create_backend() -> Box<dyn SystemProxyBackend> {
    Box::new(WindowsSystemProxyBackend)
}

#[derive(Debug, Default)]
struct WindowsSystemProxyBackend;

impl SystemProxyBackend for WindowsSystemProxyBackend {
    fn enable(&mut self, config: &SystemProxyConfig) -> SystemProxyResult<()> {
        let proxy_server = format!("{}:{}", config.host, config.port);
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

    fn disable(&mut self) -> SystemProxyResult<()> {
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
}

fn notify_windows_proxy_changed() {
    use std::ptr::null;
    use windows_sys::Win32::Networking::WinInet::{
        INTERNET_OPTION_REFRESH, INTERNET_OPTION_SETTINGS_CHANGED, InternetSetOptionW,
    };

    // SAFETY: null handle and null buffer are the documented way to broadcast per-user
    // internet option updates (settings changed + refresh).
    unsafe {
        let _ = InternetSetOptionW(null(), INTERNET_OPTION_SETTINGS_CHANGED, null(), 0);
        let _ = InternetSetOptionW(null(), INTERNET_OPTION_REFRESH, null(), 0);
    }
}

fn run_reg(args: &[&str]) -> SystemProxyResult<String> {
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
    Err(SystemProxyError::new(format!(
        "reg {} failed: {}",
        args.join(" "),
        reason
    )))
}

fn run_reg_command(args: &[&str]) -> SystemProxyResult<std::process::Output> {
    use std::process::Command;

    let mut command = Command::new("reg");
    configure_windows_hidden_command(&mut command);
    command
        .args(args)
        .output()
        .map_err(|error| SystemProxyError::new(error.to_string()))
}

fn configure_windows_hidden_command(command: &mut std::process::Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(CREATE_NO_WINDOW);
}
