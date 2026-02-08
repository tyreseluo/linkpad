use crate::{CoreError, CoreResult};
use std::fs::{self, File, OpenOptions};
use std::path::Path;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

const DEFAULT_KERNEL_BINARY: &str = "mihomo";
const RUNTIME_CONFIG_FILE: &str = "runtime.yaml";
const RUNTIME_LOG_FILE: &str = "mihomo.log";

#[derive(Clone, Debug)]
pub struct KernelInfo {
    pub binary_path: Option<String>,
    pub version: Option<String>,
    pub suggested_path: String,
    pub status: String,
}

#[derive(Debug)]
pub struct KernelRuntime {
    child: Option<Child>,
    runtime_dir: PathBuf,
    kernel_binary: String,
}

impl Default for KernelRuntime {
    fn default() -> Self {
        let mut runtime_dir = dirs::config_dir().unwrap_or_else(std::env::temp_dir);
        runtime_dir.push("linkpad");
        runtime_dir.push("runtime");
        Self {
            child: None,
            runtime_dir,
            kernel_binary: DEFAULT_KERNEL_BINARY.to_string(),
        }
    }
}

impl KernelRuntime {
    pub fn start(&mut self, config_yaml: &str) -> CoreResult<()> {
        if self.is_running() {
            return Err(CoreError::AlreadyRunning);
        }

        self.ensure_runtime_dir()?;
        let config_path = self.config_path();
        fs::write(&config_path, config_yaml)
            .map_err(|error| CoreError::InvalidConfig(error.to_string()))?;

        let log_file = self.open_log_file()?;
        let kernel_binary = self.resolve_kernel_binary()?;
        let mut command = Command::new(&kernel_binary);
        command
            .arg("-f")
            .arg(&config_path)
            .arg("-d")
            .arg(&self.runtime_dir)
            .stdout(Stdio::from(
                log_file
                    .try_clone()
                    .map_err(|error| CoreError::InvalidConfig(error.to_string()))?,
            ))
            .stderr(Stdio::from(log_file));

        let mut child = command.spawn().map_err(|error| {
            CoreError::InvalidConfig(format!(
                "failed to start `{}`: {error}",
                kernel_binary.display()
            ))
        })?;

        thread::sleep(Duration::from_millis(400));
        if let Some(status) = child
            .try_wait()
            .map_err(|error| CoreError::InvalidConfig(error.to_string()))?
        {
            return Err(CoreError::InvalidConfig(format!(
                "mihomo exited early with status {status}; check {}",
                self.log_path().display()
            )));
        }

        self.child = Some(child);
        Ok(())
    }

    pub fn stop(&mut self) -> CoreResult<()> {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        Ok(())
    }

    pub fn is_running(&mut self) -> bool {
        let Some(child) = self.child.as_mut() else {
            return false;
        };

        match child.try_wait() {
            Ok(Some(_)) => {
                self.child = None;
                false
            }
            Ok(None) => true,
            Err(_) => {
                self.child = None;
                false
            }
        }
    }

    pub fn kernel_info(&self) -> KernelInfo {
        let suggested_path = self.default_install_path().display().to_string();
        match self.resolve_kernel_binary() {
            Ok(path) => KernelInfo {
                binary_path: Some(path.display().to_string()),
                version: detect_kernel_version(&path).or_else(|| {
                    path.file_name()
                        .and_then(|name| name.to_str())
                        .and_then(extract_version_from_text)
                }),
                suggested_path,
                status: "ok".to_string(),
            },
            Err(error) => KernelInfo {
                binary_path: None,
                version: None,
                suggested_path,
                status: error.to_string(),
            },
        }
    }

    fn ensure_runtime_dir(&self) -> CoreResult<()> {
        fs::create_dir_all(&self.runtime_dir)
            .map_err(|error| CoreError::InvalidConfig(error.to_string()))
    }

    fn config_path(&self) -> PathBuf {
        self.runtime_dir.join(RUNTIME_CONFIG_FILE)
    }

    fn log_path(&self) -> PathBuf {
        self.runtime_dir.join(RUNTIME_LOG_FILE)
    }

    fn open_log_file(&self) -> CoreResult<File> {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.log_path())
            .map_err(|error| CoreError::InvalidConfig(error.to_string()))
    }

    fn resolve_kernel_binary(&self) -> CoreResult<PathBuf> {
        let mut checked_paths: Vec<PathBuf> = Vec::new();
        let mut non_executable_paths: Vec<PathBuf> = Vec::new();

        if let Some(from_env) = std::env::var_os("LINKPAD_MIHOMO_PATH") {
            let path = PathBuf::from(from_env);
            if let Some(resolved) = resolve_kernel_candidate_path(&path, &mut non_executable_paths) {
                return Ok(resolved);
            }
            checked_paths.push(path);
        }

        if let Some(found_in_path) = find_in_path(&self.kernel_binary) {
            if is_executable_file(&found_in_path) {
                return Ok(found_in_path);
            }
            non_executable_paths.push(found_in_path);
        }

        let mut candidate_paths = self.known_kernel_candidates();
        candidate_paths.insert(0, PathBuf::from(&self.kernel_binary));

        for path in candidate_paths {
            if let Some(resolved) = resolve_kernel_candidate_path(&path, &mut non_executable_paths) {
                return Ok(resolved);
            }
            checked_paths.push(path);
        }

        let hint_path = self.default_install_path();
        let mut message = format!(
            "mihomo binary not found. Set `LINKPAD_MIHOMO_PATH`, or place it at `{}`. checked: {}",
            hint_path.display(),
            checked_paths
                .iter()
                .map(|path| path.display().to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );

        if !non_executable_paths.is_empty() {
            message.push_str(&format!(
                ". found but not executable: {}. run `chmod +x <path>`",
                non_executable_paths
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        Err(CoreError::InvalidConfig(format!(
            "{message}"
        )))
    }

    fn known_kernel_candidates(&self) -> Vec<PathBuf> {
        let mut candidates = Vec::new();
        let binary_name = platform_kernel_binary_name();

        candidates.push(self.runtime_dir.join("bin").join(binary_name));

        if let Some(mut config_dir) = dirs::config_dir() {
            config_dir.push("linkpad");
            config_dir.push("bin");
            config_dir.push(binary_name);
            candidates.push(config_dir);
        }

        if let Ok(current_exe) = std::env::current_exe() {
            if let Some(exe_dir) = current_exe.parent() {
                candidates.push(exe_dir.join(binary_name));
                candidates.push(exe_dir.join("bin").join(binary_name));
            }
        }

        #[cfg(target_os = "macos")]
        {
            candidates.push(PathBuf::from("/opt/homebrew/bin").join(binary_name));
            candidates.push(PathBuf::from("/usr/local/bin").join(binary_name));
        }

        candidates
    }

    fn default_install_path(&self) -> PathBuf {
        if let Some(mut config_dir) = dirs::config_dir() {
            config_dir.push("linkpad");
            config_dir.push("bin");
            config_dir.push(platform_kernel_binary_name());
            return config_dir;
        }
        self.runtime_dir.join("bin").join(platform_kernel_binary_name())
    }
}

fn find_in_path(binary_name: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(binary_name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn detect_kernel_version(path: &Path) -> Option<String> {
    let commands: &[&[&str]] = &[&["-v"], &["--version"], &["version"]];
    for args in commands {
        let output = match Command::new(path).args(*args).output() {
            Ok(output) => output,
            Err(_) => continue,
        };
        let mut all_output = String::new();
        all_output.push_str(&String::from_utf8_lossy(&output.stdout));
        if !all_output.is_empty() {
            all_output.push('\n');
        }
        all_output.push_str(&String::from_utf8_lossy(&output.stderr));
        for line in all_output.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Some(version) = extract_version_from_text(trimmed) {
                return Some(version);
            }
            return Some(trimmed.to_string());
        }
    }
    None
}

fn extract_version_from_text(input: &str) -> Option<String> {
    for raw in input.split(|c: char| c.is_whitespace() || [',', ';', '(', ')'].contains(&c)) {
        let token = raw.trim_matches(|c: char| c == '"' || c == '\'' || c == '[' || c == ']');
        if token.len() >= 2
            && token.starts_with('v')
            && token
                .chars()
                .nth(1)
                .map(|ch| ch.is_ascii_digit())
                .unwrap_or(false)
        {
            return Some(token.to_string());
        }
    }
    None
}

fn resolve_kernel_candidate_path(
    path: &Path,
    non_executable_paths: &mut Vec<PathBuf>,
) -> Option<PathBuf> {
    if path.is_file() {
        if is_executable_file(path) {
            return Some(path.to_path_buf());
        }
        non_executable_paths.push(path.to_path_buf());
        return None;
    }

    if !path.is_dir() {
        return None;
    }

    find_kernel_binary_in_dir(path, non_executable_paths)
}

fn find_kernel_binary_in_dir(dir: &Path, non_executable_paths: &mut Vec<PathBuf>) -> Option<PathBuf> {
    let entries = fs::read_dir(dir).ok()?;
    let mut candidates = Vec::new();
    let mut fallback = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let file_name = entry.file_name().to_string_lossy().to_ascii_lowercase();
        if file_name == "mihomo"
            || file_name.starts_with("mihomo-")
            || file_name.starts_with("mihomo_")
            || file_name.starts_with("mihomo.")
        {
            candidates.push(path);
        } else if file_name.contains("mihomo") {
            fallback.push(path);
        }
    }

    candidates.extend(fallback);
    candidates.sort();
    for path in candidates {
        if is_executable_file(&path) {
            return Some(path);
        }
        non_executable_paths.push(path);
    }
    None
}

#[cfg(unix)]
fn is_executable_file(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path)
        .map(|meta| meta.is_file() && (meta.permissions().mode() & 0o111) != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable_file(path: &Path) -> bool {
    path.is_file()
}

#[cfg(target_os = "windows")]
fn platform_kernel_binary_name() -> &'static str {
    "mihomo.exe"
}

#[cfg(not(target_os = "windows"))]
fn platform_kernel_binary_name() -> &'static str {
    "mihomo"
}
