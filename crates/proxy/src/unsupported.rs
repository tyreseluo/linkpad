use crate::{SystemProxyBackend, SystemProxyConfig, SystemProxyError, SystemProxyResult};

pub(crate) fn create_backend() -> Box<dyn SystemProxyBackend> {
    Box::new(UnsupportedSystemProxyBackend)
}

#[derive(Debug, Default)]
struct UnsupportedSystemProxyBackend;

impl SystemProxyBackend for UnsupportedSystemProxyBackend {
    fn enable(&mut self, _config: &SystemProxyConfig) -> SystemProxyResult<()> {
        Err(SystemProxyError::new(format!(
            "system proxy manager is not implemented for platform `{}`",
            std::env::consts::OS
        )))
    }

    fn disable(&mut self) -> SystemProxyResult<()> {
        Err(SystemProxyError::new(format!(
            "system proxy manager is not implemented for platform `{}`",
            std::env::consts::OS
        )))
    }
}
