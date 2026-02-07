use std::fmt;
use std::sync::{Arc, Mutex};

pub type CoreResult<T> = Result<T, CoreError>;

#[derive(Debug)]
pub enum CoreError {
    AlreadyRunning,
    NotRunning,
    ProfileNotFound,
    InvalidConfig(String),
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreError::AlreadyRunning => write!(f, "core already running"),
            CoreError::NotRunning => write!(f, "core not running"),
            CoreError::ProfileNotFound => write!(f, "profile not found"),
            CoreError::InvalidConfig(msg) => write!(f, "invalid config: {msg}"),
        }
    }
}

impl std::error::Error for CoreError {}

#[derive(Clone, Debug)]
pub struct Core {
    inner: Arc<Mutex<CoreState>>,
}

#[derive(Clone, Debug)]
struct CoreState {
    running: bool,
    config: Config,
    profiles: Vec<Profile>,
}

impl Default for CoreState {
    fn default() -> Self {
        Self {
            running: false,
            config: Config::default(),
            profiles: vec![
                Profile::new("default", "Default", true),
                Profile::new("backup", "Backup", false),
            ],
        }
    }
}

impl Core {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(CoreState::default())),
        }
    }

    pub fn start(&self) -> CoreResult<()> {
        let mut state = self.inner.lock().expect("core state poisoned");
        if state.running {
            return Err(CoreError::AlreadyRunning);
        }
        state.running = true;
        Ok(())
    }

    pub fn stop(&self) -> CoreResult<()> {
        let mut state = self.inner.lock().expect("core state poisoned");
        if !state.running {
            return Err(CoreError::NotRunning);
        }
        state.running = false;
        Ok(())
    }

    pub fn restart(&self) -> CoreResult<()> {
        let _ = self.stop();
        self.start()
    }

    pub fn is_running(&self) -> bool {
        let state = self.inner.lock().expect("core state poisoned");
        state.running
    }

    pub fn config(&self) -> Config {
        let state = self.inner.lock().expect("core state poisoned");
        state.config.clone()
    }

    pub fn update_config(&self, config: Config) -> CoreResult<()> {
        let mut state = self.inner.lock().expect("core state poisoned");
        state.config = config;
        Ok(())
    }

    pub fn profiles(&self) -> Vec<Profile> {
        let state = self.inner.lock().expect("core state poisoned");
        state.profiles.clone()
    }

    pub fn set_active_profile(&self, id: &str) -> CoreResult<()> {
        let mut state = self.inner.lock().expect("core state poisoned");
        let mut found = false;
        for profile in &mut state.profiles {
            if profile.id == id {
                profile.active = true;
                found = true;
            } else {
                profile.active = false;
            }
        }
        if found {
            Ok(())
        } else {
            Err(CoreError::ProfileNotFound)
        }
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    pub mode: ProxyMode,
    pub mixed_port: u16,
    pub allow_lan: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: ProxyMode::Rule,
            mixed_port: 7890,
            allow_lan: false,
        }
    }
}

#[derive(Clone, Debug)]
pub enum ProxyMode {
    Rule,
    Global,
    Direct,
}

#[derive(Clone, Debug)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub active: bool,
}

impl Profile {
    pub fn new(id: &str, name: &str, active: bool) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            active,
        }
    }
}
