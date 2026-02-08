use crate::{CoreError, CoreResult};
use std::fs;
use std::path::{Path, PathBuf};

const MACOS_LAUNCH_AGENT_LABEL: &str = "com.linkpad.desktop";
const MACOS_LAUNCH_AGENT_FILE: &str = "com.linkpad.desktop.plist";
const SILENT_START_ARG: &str = "--silent-start";

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

        #[cfg(not(target_os = "macos"))]
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

        #[cfg(not(target_os = "macos"))]
        {
            Ok(StartupStatus::default())
        }
    }

    #[cfg(target_os = "macos")]
    fn install_macos_launch_agent(&self, silent_start: bool) -> CoreResult<()> {
        let path = self.launch_agent_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| CoreError::InvalidConfig(error.to_string()))?;
        }

        let executable = std::env::current_exe()
            .map_err(|error| CoreError::InvalidConfig(format!("failed to resolve current executable: {error}")))?;
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

        let content =
            fs::read_to_string(path).map_err(|error| CoreError::InvalidConfig(error.to_string()))?;
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
