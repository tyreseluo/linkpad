use crate::{CoreError, CoreResult};
use flate2::read::GzDecoder;
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, USER_AGENT};
use robius_directories::ProjectDirs;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs::{self, File, OpenOptions};
use std::io::{Cursor, Read, Write};
use std::path::Path;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;
use tracing::{info, warn};

const DEFAULT_KERNEL_BINARY: &str = "mihomo";
const RUNTIME_CONFIG_FILE: &str = "runtime.yaml";
const RUNTIME_LOG_FILE: &str = "mihomo.log";
const RUNTIME_PID_FILE: &str = "mihomo.pid";
const APP_QUALIFIER: &str = "";
const APP_ORGANIZATION: &str = "";
const APP_NAME: &str = "linkpad";
const MIHOMO_RELEASE_API: &str = "https://api.github.com/repos/MetaCubeX/mihomo/releases/latest";
const MIHOMO_RELEASE_LATEST_PAGE: &str = "https://github.com/MetaCubeX/mihomo/releases/latest";
const LINKPAD_HTTP_USER_AGENT: &str = "linkpad-core/0.1";

#[derive(Clone, Debug)]
pub struct KernelInfo {
    pub binary_path: Option<String>,
    pub version: Option<String>,
    pub suggested_path: String,
    pub status: String,
}

#[derive(Clone, Debug)]
pub struct KernelUpgradeInfo {
    pub version: String,
    pub binary_path: String,
    pub asset_name: String,
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    #[serde(default)]
    tag_name: String,
    #[serde(default)]
    assets: Vec<GithubReleaseAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubReleaseAsset {
    name: String,
    browser_download_url: String,
    #[serde(default)]
    digest: Option<String>,
}

#[derive(Debug)]
pub struct KernelRuntime {
    child: Option<Child>,
    runtime_dir: PathBuf,
    kernel_binary: String,
}

impl Default for KernelRuntime {
    fn default() -> Self {
        let mut runtime_dir = app_config_dir().unwrap_or_else(std::env::temp_dir);
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
        self.cleanup_stale_kernel_processes()?;
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
        let child_pid = child.id();

        thread::sleep(Duration::from_millis(400));
        if let Some(status) = child
            .try_wait()
            .map_err(|error| CoreError::InvalidConfig(error.to_string()))?
        {
            self.remove_pid_file();
            return Err(CoreError::InvalidConfig(format!(
                "mihomo exited early with status {status}; check {}",
                self.log_path().display()
            )));
        }

        self.write_pid_file(child_pid)?;
        self.child = Some(child);
        info!("started mihomo runtime pid={child_pid}");
        Ok(())
    }

    pub fn stop(&mut self) -> CoreResult<()> {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        let _ = self.cleanup_stale_kernel_processes();
        self.remove_pid_file();
        Ok(())
    }

    pub fn is_running(&mut self) -> bool {
        let Some(child) = self.child.as_mut() else {
            return false;
        };

        match child.try_wait() {
            Ok(Some(_)) => {
                self.child = None;
                self.remove_pid_file();
                false
            }
            Ok(None) => true,
            Err(_) => {
                self.child = None;
                self.remove_pid_file();
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

    pub fn install_latest_kernel(&self) -> CoreResult<KernelUpgradeInfo> {
        self.ensure_runtime_dir()?;
        let release = fetch_latest_release()?;
        let (asset_name, bytes) = if let Some(asset) = select_release_asset(&release.assets) {
            let bytes = download_release_asset(asset)?;
            verify_download_digest(asset, &bytes)?;
            (asset.name.clone(), bytes)
        } else {
            let candidates = release_asset_name_candidates(&release.tag_name);
            download_release_asset_by_candidates(&release.tag_name, &candidates)?
        };
        let binary = decompress_gzip(&bytes)?;

        let install_path = self.install_target_path();
        if let Some(parent) = install_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| CoreError::InvalidConfig(error.to_string()))?;
        }

        if install_path.exists() && install_path.is_dir() {
            return Err(CoreError::InvalidConfig(format!(
                "kernel install target is still a directory: `{}`",
                install_path.display()
            )));
        }

        let temp_path = install_path.with_extension("tmp");
        {
            let mut temp_file = File::create(&temp_path)
                .map_err(|error| CoreError::InvalidConfig(error.to_string()))?;
            temp_file
                .write_all(&binary)
                .map_err(|error| CoreError::InvalidConfig(error.to_string()))?;
        }
        set_executable_permissions(&temp_path)?;

        if install_path.exists() {
            fs::remove_file(&install_path).map_err(|error| {
                CoreError::InvalidConfig(format!(
                    "failed to replace existing kernel binary `{}`: {error}",
                    install_path.display()
                ))
            })?;
        }

        fs::rename(&temp_path, &install_path).map_err(|error| {
            CoreError::InvalidConfig(format!(
                "failed to install kernel binary at `{}`: {error}",
                install_path.display()
            ))
        })?;

        Ok(KernelUpgradeInfo {
            version: release.tag_name,
            binary_path: install_path.display().to_string(),
            asset_name,
        })
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

    fn pid_path(&self) -> PathBuf {
        self.runtime_dir.join(RUNTIME_PID_FILE)
    }

    fn open_log_file(&self) -> CoreResult<File> {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.log_path())
            .map_err(|error| CoreError::InvalidConfig(error.to_string()))
    }

    fn write_pid_file(&self, pid: u32) -> CoreResult<()> {
        fs::write(self.pid_path(), format!("{pid}\n"))
            .map_err(|error| CoreError::InvalidConfig(error.to_string()))
    }

    fn remove_pid_file(&self) {
        let _ = fs::remove_file(self.pid_path());
    }

    fn find_managed_kernel_pids(&self) -> Vec<u32> {
        let mut pids = BTreeSet::new();
        if let Ok(raw_pid) = fs::read_to_string(self.pid_path()) {
            if let Ok(pid) = raw_pid.trim().parse::<u32>() {
                if is_runtime_mihomo_pid(pid, &self.runtime_dir, &self.config_path()) {
                    pids.insert(pid);
                }
            }
        }
        for pid in list_runtime_mihomo_processes(&self.runtime_dir, &self.config_path()) {
            pids.insert(pid);
        }
        pids.into_iter().collect()
    }

    fn cleanup_stale_kernel_processes(&self) -> CoreResult<()> {
        let stale_pids = self.find_managed_kernel_pids();
        if stale_pids.is_empty() {
            self.remove_pid_file();
            return Ok(());
        }

        for pid in stale_pids {
            warn!("terminating stale mihomo process pid={pid}");
            terminate_process(pid)?;
        }
        self.remove_pid_file();
        Ok(())
    }

    fn resolve_kernel_binary(&self) -> CoreResult<PathBuf> {
        let mut checked_paths: Vec<PathBuf> = Vec::new();
        let mut non_executable_paths: Vec<PathBuf> = Vec::new();

        if let Some(from_env) = std::env::var_os("LINKPAD_MIHOMO_PATH") {
            let path = PathBuf::from(from_env);
            if let Some(resolved) = resolve_kernel_candidate_path(&path, &mut non_executable_paths)
            {
                return Ok(resolved);
            }
            checked_paths.push(path);
        }

        let mut candidate_paths = self.known_kernel_candidates();
        candidate_paths.insert(0, PathBuf::from(&self.kernel_binary));

        for path in candidate_paths {
            if let Some(resolved) = resolve_kernel_candidate_path(&path, &mut non_executable_paths)
            {
                return Ok(resolved);
            }
            checked_paths.push(path);
        }

        if let Some(found_in_path) = find_in_path(&self.kernel_binary) {
            if is_executable_file(&found_in_path) {
                return Ok(found_in_path);
            }
            non_executable_paths.push(found_in_path.clone());
            checked_paths.push(found_in_path);
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

        Err(CoreError::InvalidConfig(format!("{message}")))
    }

    fn known_kernel_candidates(&self) -> Vec<PathBuf> {
        let mut candidates = Vec::new();
        let binary_name = platform_kernel_binary_name();

        let runtime_bin_dir = self.runtime_dir.join("bin");
        candidates.push(runtime_bin_dir.join(binary_name));
        candidates.push(runtime_bin_dir);

        if let Some(config_dir) = app_config_dir() {
            let config_bin_dir = config_dir.join("bin");
            candidates.push(config_bin_dir.join(binary_name));
            candidates.push(config_bin_dir);
        }

        if let Ok(current_exe) = std::env::current_exe() {
            if let Some(exe_dir) = current_exe.parent() {
                candidates.push(exe_dir.join(binary_name));
                let exe_bin_dir = exe_dir.join("bin");
                candidates.push(exe_bin_dir.join(binary_name));
                candidates.push(exe_bin_dir);
                if let Some(parent_dir) = exe_dir.parent() {
                    for resources_dir in ["Resources", "resources"] {
                        let resources_root = parent_dir.join(resources_dir);
                        let linkpad_nested_resources_bin_dir =
                            resources_root.join("linkpad").join("resources").join("bin");
                        candidates.push(linkpad_nested_resources_bin_dir.join(binary_name));
                        candidates.push(linkpad_nested_resources_bin_dir);

                        let linkpad_bin_dir = resources_root.join("linkpad").join("bin");
                        candidates.push(linkpad_bin_dir.join(binary_name));
                        candidates.push(linkpad_bin_dir);

                        let nested_resources_bin_dir = resources_root.join("resources").join("bin");
                        candidates.push(nested_resources_bin_dir.join(binary_name));
                        candidates.push(nested_resources_bin_dir);

                        let resources_bin_dir = resources_root.join("bin");
                        candidates.push(resources_bin_dir.join(binary_name));
                        candidates.push(resources_bin_dir);
                    }
                }
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
        if let Some(config_dir) = app_config_dir() {
            return config_dir.join("bin").join(platform_kernel_binary_name());
        }
        self.runtime_dir
            .join("bin")
            .join(platform_kernel_binary_name())
    }

    fn install_target_path(&self) -> PathBuf {
        let preferred = self.default_install_path();
        if preferred.is_dir() {
            return preferred.join(platform_kernel_binary_name());
        }
        preferred
    }
}

impl Drop for KernelRuntime {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

fn app_config_dir() -> Option<PathBuf> {
    let project_dirs = ProjectDirs::from(APP_QUALIFIER, APP_ORGANIZATION, APP_NAME)?;
    Some(project_dirs.config_dir().to_path_buf())
}

fn fetch_latest_release() -> CoreResult<GithubRelease> {
    match fetch_latest_release_from_api() {
        Ok(release) => Ok(release),
        Err(api_error) => {
            let fallback_tag = fetch_latest_release_tag_from_web().map_err(|web_error| {
                CoreError::Network(format!(
                    "{api_error}; fallback(web latest) failed: {web_error}"
                ))
            })?;
            Ok(GithubRelease {
                tag_name: fallback_tag,
                assets: Vec::new(),
            })
        }
    }
}

fn fetch_latest_release_from_api() -> CoreResult<GithubRelease> {
    let client = Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|error| CoreError::Network(error.to_string()))?;

    let mut request = client
        .get(MIHOMO_RELEASE_API)
        .header(USER_AGENT, LINKPAD_HTTP_USER_AGENT)
        .header(ACCEPT, "application/vnd.github+json");
    if let Ok(token) = std::env::var("LINKPAD_GITHUB_TOKEN") {
        let token = token.trim();
        if !token.is_empty() {
            request = request.bearer_auth(token);
        }
    }
    let response = request
        .send()
        .map_err(|error| CoreError::Network(error.to_string()))?;

    let status = response.status();
    let body = response
        .text()
        .map_err(|error| CoreError::Network(error.to_string()))?;
    if !status.is_success() {
        return Err(CoreError::Network(format!(
            "failed to fetch latest mihomo release: {status} {body}"
        )));
    }

    let mut release: GithubRelease =
        serde_json::from_str(&body).map_err(|error| CoreError::Parse(error.to_string()))?;
    release.tag_name = normalize_version_tag(&release.tag_name);
    if release.tag_name.is_empty() {
        return Err(CoreError::Parse(
            "latest release has an empty tag".to_string(),
        ));
    }
    Ok(release)
}

fn fetch_latest_release_tag_from_web() -> CoreResult<String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|error| CoreError::Network(error.to_string()))?;

    let response = client
        .get(MIHOMO_RELEASE_LATEST_PAGE)
        .header(USER_AGENT, LINKPAD_HTTP_USER_AGENT)
        .send()
        .map_err(|error| CoreError::Network(error.to_string()))?;

    let status = response.status();
    if !status.is_success() {
        let body = response
            .text()
            .unwrap_or_else(|_| "unable to read response body".to_string());
        return Err(CoreError::Network(format!(
            "failed to fetch latest release page: {status} {body}"
        )));
    }

    let path = response.url().path().to_string();
    let marker = "/releases/tag/";
    let tag = path
        .split(marker)
        .nth(1)
        .map(|value| value.trim_matches('/').to_string())
        .unwrap_or_default();
    let tag = normalize_version_tag(&tag);
    if tag.is_empty() {
        return Err(CoreError::Parse(
            "failed to parse latest release tag from github page".to_string(),
        ));
    }
    Ok(tag)
}

fn select_release_asset(assets: &[GithubReleaseAsset]) -> Option<&GithubReleaseAsset> {
    let os = release_os_tag();
    let arch = release_arch_tag();
    let required_prefix = format!("mihomo-{os}-{arch}-");

    let mut matches = assets
        .iter()
        .filter(|asset| {
            asset.name.starts_with(&required_prefix)
                && asset.name.to_ascii_lowercase().ends_with(".gz")
        })
        .collect::<Vec<_>>();

    matches.sort_by_key(|asset| score_release_asset(&asset.name));
    matches.into_iter().next()
}

fn release_asset_name_candidates(tag: &str) -> Vec<String> {
    let os = release_os_tag();
    let arch = release_arch_tag();
    let normalized_tag = normalize_version_tag(tag);
    let mut names = BTreeSet::new();

    names.insert(format!("mihomo-{os}-{arch}-{normalized_tag}.gz"));
    names.insert(format!("mihomo-{os}-{arch}-go124-{normalized_tag}.gz"));
    names.insert(format!("mihomo-{os}-{arch}-go122-{normalized_tag}.gz"));
    names.insert(format!("mihomo-{os}-{arch}-go120-{normalized_tag}.gz"));
    names.insert(format!("mihomo-{os}-{arch}-v1-{normalized_tag}.gz"));
    names.insert(format!("mihomo-{os}-{arch}-v2-{normalized_tag}.gz"));
    names.insert(format!("mihomo-{os}-{arch}-v1-go124-{normalized_tag}.gz"));
    names.insert(format!("mihomo-{os}-{arch}-v1-go122-{normalized_tag}.gz"));
    names.insert(format!("mihomo-{os}-{arch}-v1-go120-{normalized_tag}.gz"));
    names.insert(format!("mihomo-{os}-{arch}-v2-go124-{normalized_tag}.gz"));
    names.insert(format!("mihomo-{os}-{arch}-v2-go122-{normalized_tag}.gz"));
    names.insert(format!("mihomo-{os}-{arch}-v2-go120-{normalized_tag}.gz"));
    if arch == "amd64" {
        names.insert(format!("mihomo-{os}-{arch}-compatible-{normalized_tag}.gz"));
    }

    names.into_iter().collect()
}

fn download_release_asset_by_candidates(
    tag: &str,
    candidates: &[String],
) -> CoreResult<(String, Vec<u8>)> {
    let client = Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|error| CoreError::Network(error.to_string()))?;

    let mut attempts = Vec::new();
    for asset_name in candidates {
        let urls = [
            format!("https://github.com/MetaCubeX/mihomo/releases/download/{tag}/{asset_name}"),
            format!("https://github.com/MetaCubeX/mihomo/releases/latest/download/{asset_name}"),
        ];
        for url in &urls {
            match download_asset_from_url(&client, url) {
                Ok(Some(bytes)) => return Ok((asset_name.clone(), bytes)),
                Ok(None) => attempts.push(format!("{asset_name}@404")),
                Err(error) => attempts.push(format!("{asset_name}@{error}")),
            }
        }
    }

    Err(CoreError::Network(format!(
        "failed to download compatible kernel asset for tag `{tag}`. attempts: {}",
        attempts.join(", ")
    )))
}

fn score_release_asset(name: &str) -> (u8, u8, u8, usize) {
    let lower = name.to_ascii_lowercase();
    let has_alpha = lower.contains("alpha");
    let has_compatible = lower.contains("compatible");
    let has_go = lower.contains("-go");
    (
        has_alpha as u8,
        has_compatible as u8,
        has_go as u8,
        name.len(),
    )
}

fn download_release_asset(asset: &GithubReleaseAsset) -> CoreResult<Vec<u8>> {
    let client = Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|error| CoreError::Network(error.to_string()))?;

    match download_asset_from_url(&client, &asset.browser_download_url)? {
        Some(bytes) => Ok(bytes),
        None => Err(CoreError::Network(format!(
            "failed to download kernel asset `{}`: 404 Not Found",
            asset.name
        ))),
    }
}

fn download_asset_from_url(client: &Client, url: &str) -> CoreResult<Option<Vec<u8>>> {
    let response = client
        .get(url)
        .header(USER_AGENT, LINKPAD_HTTP_USER_AGENT)
        .send()
        .map_err(|error| CoreError::Network(error.to_string()))?;

    let status = response.status();
    if status == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    if !status.is_success() {
        let body = response
            .text()
            .unwrap_or_else(|_| "unable to read response body".to_string());
        return Err(CoreError::Network(format!(
            "download failed: {status} {body}"
        )));
    }

    response
        .bytes()
        .map(|bytes| Some(bytes.to_vec()))
        .map_err(|error| CoreError::Network(error.to_string()))
}

fn verify_download_digest(asset: &GithubReleaseAsset, bytes: &[u8]) -> CoreResult<()> {
    let Some(raw_digest) = asset.digest.as_ref() else {
        return Ok(());
    };

    let expected = raw_digest
        .strip_prefix("sha256:")
        .unwrap_or(raw_digest.as_str())
        .trim()
        .to_ascii_lowercase();
    if expected.is_empty() {
        return Ok(());
    }

    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let actual = format!("{:x}", hasher.finalize());
    if actual != expected {
        return Err(CoreError::InvalidConfig(format!(
            "downloaded kernel digest mismatch for `{}`",
            asset.name
        )));
    }
    Ok(())
}

fn decompress_gzip(bytes: &[u8]) -> CoreResult<Vec<u8>> {
    let mut decoder = GzDecoder::new(Cursor::new(bytes));
    let mut output = Vec::new();
    decoder
        .read_to_end(&mut output)
        .map_err(|error| CoreError::InvalidConfig(error.to_string()))?;
    if output.is_empty() {
        return Err(CoreError::InvalidConfig(
            "downloaded kernel archive is empty".to_string(),
        ));
    }
    Ok(output)
}

#[cfg(unix)]
fn list_runtime_mihomo_processes(runtime_dir: &Path, config_path: &Path) -> Vec<u32> {
    let output = match Command::new("ps").args(["-axo", "pid=,command="]).output() {
        Ok(output) if output.status.success() => output,
        _ => return Vec::new(),
    };
    let runtime_marker = runtime_dir.display().to_string();
    let config_marker = config_path.display().to_string();

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim_start();
            let mut parts = trimmed.splitn(2, char::is_whitespace);
            let pid = parts.next()?.trim().parse::<u32>().ok()?;
            let command = parts.next()?.trim();
            if !command.contains("mihomo") {
                return None;
            }
            if !command.contains(&runtime_marker) || !command.contains(&config_marker) {
                return None;
            }
            Some(pid)
        })
        .collect()
}

#[cfg(unix)]
fn is_runtime_mihomo_pid(pid: u32, runtime_dir: &Path, config_path: &Path) -> bool {
    let pid_arg = pid.to_string();
    let output = match Command::new("ps")
        .args(["-p", &pid_arg, "-o", "command="])
        .output()
    {
        Ok(output) if output.status.success() => output,
        _ => return false,
    };
    let command = String::from_utf8_lossy(&output.stdout);
    let runtime_marker = runtime_dir.display().to_string();
    let config_marker = config_path.display().to_string();
    command.contains("mihomo")
        && command.contains(&runtime_marker)
        && command.contains(&config_marker)
}

#[cfg(not(unix))]
fn is_runtime_mihomo_pid(_pid: u32, _runtime_dir: &Path, _config_path: &Path) -> bool {
    false
}

#[cfg(not(unix))]
fn list_runtime_mihomo_processes(_runtime_dir: &Path, _config_path: &Path) -> Vec<u32> {
    Vec::new()
}

#[cfg(unix)]
fn terminate_process(pid: u32) -> CoreResult<()> {
    let pid_str = pid.to_string();
    let _ = Command::new("kill").arg("-TERM").arg(&pid_str).status();
    for _ in 0..12 {
        if !process_exists(pid) {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(100));
    }

    let _ = Command::new("kill").arg("-KILL").arg(&pid_str).status();
    for _ in 0..12 {
        if !process_exists(pid) {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(100));
    }

    Err(CoreError::InvalidConfig(format!(
        "failed to terminate stale mihomo process pid={pid}"
    )))
}

#[cfg(not(unix))]
fn terminate_process(_pid: u32) -> CoreResult<()> {
    Ok(())
}

#[cfg(unix)]
fn process_exists(pid: u32) -> bool {
    Command::new("kill")
        .arg("-0")
        .arg(pid.to_string())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

#[cfg(unix)]
fn set_executable_permissions(path: &Path) -> CoreResult<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path)
        .map_err(|error| CoreError::InvalidConfig(error.to_string()))?
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms).map_err(|error| CoreError::InvalidConfig(error.to_string()))
}

#[cfg(not(unix))]
fn set_executable_permissions(_path: &Path) -> CoreResult<()> {
    Ok(())
}

#[cfg(target_os = "macos")]
fn release_os_tag() -> &'static str {
    "darwin"
}

#[cfg(target_os = "linux")]
fn release_os_tag() -> &'static str {
    "linux"
}

#[cfg(target_os = "windows")]
fn release_os_tag() -> &'static str {
    "windows"
}

#[cfg(all(
    not(target_os = "macos"),
    not(target_os = "linux"),
    not(target_os = "windows")
))]
fn release_os_tag() -> &'static str {
    std::env::consts::OS
}

#[cfg(target_arch = "aarch64")]
fn release_arch_tag() -> &'static str {
    "arm64"
}

#[cfg(target_arch = "x86_64")]
fn release_arch_tag() -> &'static str {
    "amd64"
}

#[cfg(target_arch = "x86")]
fn release_arch_tag() -> &'static str {
    "386"
}

#[cfg(target_arch = "arm")]
fn release_arch_tag() -> &'static str {
    "armv7"
}

#[cfg(all(
    not(target_arch = "aarch64"),
    not(target_arch = "x86_64"),
    not(target_arch = "x86"),
    not(target_arch = "arm")
))]
fn release_arch_tag() -> &'static str {
    std::env::consts::ARCH
}

fn normalize_version_tag(tag: &str) -> String {
    let trimmed = tag.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.starts_with('v') {
        trimmed.to_string()
    } else {
        format!("v{trimmed}")
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

fn find_kernel_binary_in_dir(
    dir: &Path,
    non_executable_paths: &mut Vec<PathBuf>,
) -> Option<PathBuf> {
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
