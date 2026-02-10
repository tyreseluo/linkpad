use crate::{SILENT_START_ARG, StartupBackend, StartupError, StartupResult, StartupStatus};
use std::path::{Path, PathBuf};

const MACOS_LAUNCH_AGENT_LABEL: &str = "com.linkpad.desktop";
const MACOS_LAUNCH_AGENT_FILE: &str = "com.linkpad.desktop.plist";

pub(crate) fn create_backend() -> Box<dyn StartupBackend> {
    Box::new(MacosStartupBackend)
}

#[derive(Debug, Default)]
struct MacosStartupBackend;

impl StartupBackend for MacosStartupBackend {
    fn configure(&self, auto_launch: bool, silent_start: bool) -> StartupResult<()> {
        if auto_launch {
            install_macos_launch_agent(silent_start)
        } else {
            remove_macos_launch_agent()
        }
    }

    fn status(&self) -> StartupResult<StartupStatus> {
        read_macos_launch_agent_status()
    }
}

fn install_macos_launch_agent(silent_start: bool) -> StartupResult<()> {
    let path = launch_agent_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| StartupError::new(error.to_string()))?;
    }

    let executable = std::env::current_exe().map_err(|error| {
        StartupError::new(format!("failed to resolve current executable: {error}"))
    })?;
    let plist = build_launch_agent_plist(&executable, silent_start);
    std::fs::write(path, plist).map_err(|error| StartupError::new(error.to_string()))?;
    Ok(())
}

fn remove_macos_launch_agent() -> StartupResult<()> {
    let path = launch_agent_path()?;
    if path.exists() {
        std::fs::remove_file(path).map_err(|error| StartupError::new(error.to_string()))?;
    }
    Ok(())
}

fn read_macos_launch_agent_status() -> StartupResult<StartupStatus> {
    let path = launch_agent_path()?;
    if !path.exists() {
        return Ok(StartupStatus::default());
    }

    let content =
        std::fs::read_to_string(path).map_err(|error| StartupError::new(error.to_string()))?;
    let silent_start = content.contains(&format!("<string>{SILENT_START_ARG}</string>"));
    Ok(StartupStatus {
        auto_launch: true,
        silent_start,
    })
}

fn launch_agent_path() -> StartupResult<PathBuf> {
    let mut home_dir = dirs::home_dir()
        .ok_or_else(|| StartupError::new("failed to locate user home directory"))?;
    home_dir.push("Library");
    home_dir.push("LaunchAgents");
    home_dir.push(MACOS_LAUNCH_AGENT_FILE);
    Ok(home_dir)
}

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

fn escape_xml(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
