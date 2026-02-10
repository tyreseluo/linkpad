use std::fmt;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod unsupported;
#[cfg(target_os = "windows")]
mod windows;

#[derive(Debug, Clone)]
pub struct SystemProxyError {
    message: String,
}

impl SystemProxyError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for SystemProxyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for SystemProxyError {}

pub type SystemProxyResult<T> = Result<T, SystemProxyError>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemProxyConfig {
    pub host: String,
    pub port: u16,
}

impl SystemProxyConfig {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
        }
    }
}

pub trait SystemProxyBackend: Send + fmt::Debug {
    fn enable(&mut self, config: &SystemProxyConfig) -> SystemProxyResult<()>;
    fn disable(&mut self) -> SystemProxyResult<()>;
}

#[derive(Debug)]
pub struct SystemProxyManager {
    backend: Box<dyn SystemProxyBackend>,
}

impl Default for SystemProxyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemProxyManager {
    pub fn new() -> Self {
        Self {
            backend: create_default_backend(),
        }
    }

    pub fn new_with_backend(backend: Box<dyn SystemProxyBackend>) -> Self {
        Self { backend }
    }

    pub fn enable(&mut self, host: &str, port: u16) -> SystemProxyResult<()> {
        let config = SystemProxyConfig::new(host, port);
        self.enable_with_config(&config)
    }

    pub fn enable_with_config(&mut self, config: &SystemProxyConfig) -> SystemProxyResult<()> {
        self.backend.enable(config)
    }

    pub fn disable(&mut self) -> SystemProxyResult<()> {
        self.backend.disable()
    }
}

#[cfg(target_os = "macos")]
fn create_default_backend() -> Box<dyn SystemProxyBackend> {
    macos::create_backend()
}

#[cfg(target_os = "windows")]
fn create_default_backend() -> Box<dyn SystemProxyBackend> {
    windows::create_backend()
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn create_default_backend() -> Box<dyn SystemProxyBackend> {
    unsupported::create_backend()
}
