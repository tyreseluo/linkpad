use crate::{SILENT_START_ARG, StartupBackend, StartupError, StartupResult, StartupStatus};
use std::path::Path;

const WINDOWS_RUN_REG_PATH: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
const WINDOWS_RUN_VALUE_NAME: &str = "Linkpad";

pub(crate) fn create_backend() -> Box<dyn StartupBackend> {
    Box::new(WindowsStartupBackend)
}

#[derive(Debug, Default)]
struct WindowsStartupBackend;

impl StartupBackend for WindowsStartupBackend {
    fn configure(&self, auto_launch: bool, silent_start: bool) -> StartupResult<()> {
        if auto_launch {
            install_windows_run_entry(silent_start)
        } else {
            remove_windows_run_entry()
        }
    }

    fn status(&self) -> StartupResult<StartupStatus> {
        read_windows_run_entry_status()
    }
}

fn install_windows_run_entry(silent_start: bool) -> StartupResult<()> {
    let executable = std::env::current_exe().map_err(|error| {
        StartupError::new(format!("failed to resolve current executable: {error}"))
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

fn remove_windows_run_entry() -> StartupResult<()> {
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

fn read_windows_run_entry_status() -> StartupResult<StartupStatus> {
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

fn build_windows_run_command(executable: &Path, silent_start: bool) -> String {
    let escaped = executable.to_string_lossy().replace('"', "\\\"");
    let mut command_line = format!("\"{escaped}\"");
    if silent_start {
        command_line.push(' ');
        command_line.push_str(SILENT_START_ARG);
    }
    command_line
}

fn query_windows_run_value() -> StartupResult<Option<String>> {
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

fn run_reg(args: &[&str]) -> StartupResult<String> {
    let output = run_reg_command(args)?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
    }
    Err(reg_command_error(args, &output))
}

fn run_reg_command(args: &[&str]) -> StartupResult<std::process::Output> {
    use std::process::Command;

    let mut command = Command::new("reg");
    configure_windows_hidden_command(&mut command);
    command
        .args(args)
        .output()
        .map_err(|error| StartupError::new(error.to_string()))
}

fn reg_command_error(args: &[&str], output: &std::process::Output) -> StartupError {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let reason = if !stderr.trim().is_empty() {
        stderr.trim().to_string()
    } else {
        stdout.trim().to_string()
    };
    StartupError::new(format!("reg {} failed: {}", args.join(" "), reason))
}

fn configure_windows_hidden_command(command: &mut std::process::Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(CREATE_NO_WINDOW);
}
