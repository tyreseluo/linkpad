use crate::{CoreError, CoreResult};
#[cfg(target_os = "macos")]
use std::fs;
use std::path::Path;
#[cfg(target_os = "macos")]
use std::path::PathBuf;

#[cfg(target_os = "macos")]
const MACOS_LAUNCH_AGENT_LABEL: &str = "com.linkpad.desktop";
#[cfg(target_os = "macos")]
const MACOS_LAUNCH_AGENT_FILE: &str = "com.linkpad.desktop.plist";
const SILENT_START_ARG: &str = "--silent-start";
#[cfg(target_os = "windows")]
const WINDOWS_RUN_REG_PATH: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
#[cfg(target_os = "windows")]
const WINDOWS_RUN_VALUE_NAME: &str = "Linkpad";

#[derive(Clone, Copy, Debug, Default)]
pub struct StartupStatus {
    pub auto_launch: bool,
    pub silent_start: bool,
}

#[derive(Debug, Default)]
pub struct StartupManager;

impl StartupManager {
    pub fn configure(&self, auto_launch: bool, silent_start: bool) -> CoreResult<()> {
        #[cfg(target_os = "macos")]
        {
            if auto_launch {
                self.install_macos_launch_agent(silent_start)
            } else {
                self.remove_macos_launch_agent()
            }
        }

        #[cfg(target_os = "windows")]
        {
            if auto_launch {
                self.install_windows_run_entry(silent_start)
            } else {
                self.remove_windows_run_entry()
            }
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            let _ = auto_launch;
            let _ = silent_start;
            Ok(())
        }
    }

    pub fn status(&self) -> CoreResult<StartupStatus> {
        #[cfg(target_os = "macos")]
        {
            self.read_macos_launch_agent_status()
        }

        #[cfg(target_os = "windows")]
        {
            self.read_windows_run_entry_status()
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            Ok(StartupStatus::default())
        }
    }

    #[cfg(target_os = "macos")]
    fn install_macos_launch_agent(&self, silent_start: bool) -> CoreResult<()> {
        let path = self.launch_agent_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| CoreError::InvalidConfig(error.to_string()))?;
        }

        let executable = std::env::current_exe().map_err(|error| {
            CoreError::InvalidConfig(format!("failed to resolve current executable: {error}"))
        })?;
        let plist = build_launch_agent_plist(&executable, silent_start);
        fs::write(&path, plist).map_err(|error| CoreError::InvalidConfig(error.to_string()))?;
        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn remove_macos_launch_agent(&self) -> CoreResult<()> {
        let path = self.launch_agent_path()?;
        if path.exists() {
            fs::remove_file(path).map_err(|error| CoreError::InvalidConfig(error.to_string()))?;
        }
        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn read_macos_launch_agent_status(&self) -> CoreResult<StartupStatus> {
        let path = self.launch_agent_path()?;
        if !path.exists() {
            return Ok(StartupStatus::default());
        }

        let content = fs::read_to_string(path)
            .map_err(|error| CoreError::InvalidConfig(error.to_string()))?;
        let silent_start = content.contains(&format!("<string>{SILENT_START_ARG}</string>"));
        Ok(StartupStatus {
            auto_launch: true,
            silent_start,
        })
    }

    #[cfg(target_os = "macos")]
    fn launch_agent_path(&self) -> CoreResult<PathBuf> {
        let mut home_dir = dirs::home_dir().ok_or_else(|| {
            CoreError::InvalidConfig("failed to locate user home directory".to_string())
        })?;
        home_dir.push("Library");
        home_dir.push("LaunchAgents");
        home_dir.push(MACOS_LAUNCH_AGENT_FILE);
        Ok(home_dir)
    }

    #[cfg(target_os = "windows")]
    fn install_windows_run_entry(&self, silent_start: bool) -> CoreResult<()> {
        let executable = std::env::current_exe().map_err(|error| {
            CoreError::InvalidConfig(format!("failed to resolve current executable: {error}"))
        })?;
        let run_command = build_windows_run_command(&executable, silent_start);
        run_reg(&[
            "add",
            WINDOWS_RUN_REG_PATH,
            "/v",
            WINDOWS_RUN_VALUE_NAME,
            "/t",
            "REG_SZ",
            "/d",
            run_command.as_str(),
            "/f",
        ])?;
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn remove_windows_run_entry(&self) -> CoreResult<()> {
        let output = run_reg_command(&[
            "delete",
            WINDOWS_RUN_REG_PATH,
            "/v",
            WINDOWS_RUN_VALUE_NAME,
            "/f",
        ])?;
        if output.status.success() || output.status.code() == Some(1) {
            return Ok(());
        }
        Err(reg_command_error(
            &[
                "delete",
                WINDOWS_RUN_REG_PATH,
                "/v",
                WINDOWS_RUN_VALUE_NAME,
                "/f",
            ],
            &output,
        ))
    }

    #[cfg(target_os = "windows")]
    fn read_windows_run_entry_status(&self) -> CoreResult<StartupStatus> {
        let value = query_windows_run_value()?;
        let Some(command_line) = value else {
            return Ok(StartupStatus::default());
        };
        Ok(StartupStatus {
            auto_launch: true,
            silent_start: command_line
                .split_whitespace()
                .any(|arg| arg.eq_ignore_ascii_case(SILENT_START_ARG)),
        })
    }
}

#[cfg(target_os = "macos")]
fn build_launch_agent_plist(executable: &Path, silent_start: bool) -> String {
    let mut args = vec![format!(
        "<string>{}</string>",
        escape_xml(&executable.to_string_lossy())
    )];
    if silent_start {
        args.push(format!("<string>{SILENT_START_ARG}</string>"));
    }
    let arguments_block = args.join("\n        ");

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{label}</string>
    <key>ProgramArguments</key>
    <array>
        {arguments}
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <false/>
</dict>
</plist>
"#,
        label = MACOS_LAUNCH_AGENT_LABEL,
        arguments = arguments_block,
    )
}

#[cfg(target_os = "macos")]
fn escape_xml(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(target_os = "windows")]
fn build_windows_run_command(executable: &Path, silent_start: bool) -> String {
    let escaped = executable.to_string_lossy().replace('"', "\\\"");
    let mut command_line = format!("\"{escaped}\"");
    if silent_start {
        command_line.push(' ');
        command_line.push_str(SILENT_START_ARG);
    }
    command_line
}

#[cfg(target_os = "windows")]
fn query_windows_run_value() -> CoreResult<Option<String>> {
    let args = ["query", WINDOWS_RUN_REG_PATH, "/v", WINDOWS_RUN_VALUE_NAME];
    let output = run_reg_command(&args)?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Ok(parse_reg_query_value(&stdout, WINDOWS_RUN_VALUE_NAME));
    }
    if output.status.code() == Some(1) {
        return Ok(None);
    }
    Err(reg_command_error(&args, &output))
}

#[cfg(target_os = "windows")]
fn parse_reg_query_value(output: &str, value_name: &str) -> Option<String> {
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let mut parts = trimmed.split_whitespace();
        let Some(current_name) = parts.next() else {
            continue;
        };
        if !current_name.eq_ignore_ascii_case(value_name) {
            continue;
        }

        if let Some((_, value)) = trimmed.split_once("REG_SZ") {
            return Some(value.trim().to_string());
        }
        if let Some((_, value)) = trimmed.split_once("REG_EXPAND_SZ") {
            return Some(value.trim().to_string());
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn run_reg(args: &[&str]) -> CoreResult<String> {
    let output = run_reg_command(args)?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
    }
    Err(reg_command_error(args, &output))
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
fn reg_command_error(args: &[&str], output: &std::process::Output) -> CoreError {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let reason = if !stderr.trim().is_empty() {
        stderr.trim().to_string()
    } else {
        stdout.trim().to_string()
    };
    CoreError::InvalidConfig(format!("reg {} failed: {}", args.join(" "), reason))
}

#[cfg(target_os = "windows")]
fn configure_windows_hidden_command(command: &mut std::process::Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(CREATE_NO_WINDOW);
}
