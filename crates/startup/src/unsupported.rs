use crate::{StartupBackend, StartupResult, StartupStatus};

pub(crate) fn create_backend() -> Box<dyn StartupBackend> {
    Box::new(UnsupportedStartupBackend)
}

#[derive(Debug, Default)]
struct UnsupportedStartupBackend;

impl StartupBackend for UnsupportedStartupBackend {
    fn configure(&self, _auto_launch: bool, _silent_start: bool) -> StartupResult<()> {
        Ok(())
    }

    fn status(&self) -> StartupResult<StartupStatus> {
        Ok(StartupStatus::default())
    }
}
