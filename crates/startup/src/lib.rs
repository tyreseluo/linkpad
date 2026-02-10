use std::fmt;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod unsupported;
#[cfg(target_os = "windows")]
mod windows;

pub(crate) const SILENT_START_ARG: &str = "--silent-start";

#[derive(Debug, Clone)]
pub struct StartupError {
    message: String,
}

impl StartupError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for StartupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for StartupError {}

pub type StartupResult<T> = Result<T, StartupError>;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct StartupStatus {
    pub auto_launch: bool,
    pub silent_start: bool,
}

pub trait StartupBackend: Send + Sync + fmt::Debug {
    fn configure(&self, auto_launch: bool, silent_start: bool) -> StartupResult<()>;
    fn status(&self) -> StartupResult<StartupStatus>;
}

#[derive(Debug)]
pub struct StartupManager {
    backend: Box<dyn StartupBackend>,
}

impl Default for StartupManager {
    fn default() -> Self {
        Self::new()
    }
}

impl StartupManager {
    pub fn new() -> Self {
        Self {
            backend: create_default_backend(),
        }
    }

    pub fn new_with_backend(backend: Box<dyn StartupBackend>) -> Self {
        Self { backend }
    }

    pub fn configure(&self, auto_launch: bool, silent_start: bool) -> StartupResult<()> {
        self.backend.configure(auto_launch, silent_start)
    }

    pub fn status(&self) -> StartupResult<StartupStatus> {
        self.backend.status()
    }
}

#[cfg(target_os = "macos")]
fn create_default_backend() -> Box<dyn StartupBackend> {
    macos::create_backend()
}

#[cfg(target_os = "windows")]
fn create_default_backend() -> Box<dyn StartupBackend> {
    windows::create_backend()
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn create_default_backend() -> Box<dyn StartupBackend> {
    unsupported::create_backend()
}
