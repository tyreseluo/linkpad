pub mod profile_store;
pub mod settings_store;

use robius_directories::ProjectDirs;
use std::path::PathBuf;

const APP_QUALIFIER: &str = "";
const APP_ORGANIZATION: &str = "";
const APP_NAME: &str = "linkpad";

pub(crate) fn app_config_dir() -> Option<PathBuf> {
    let project_dirs = ProjectDirs::from(APP_QUALIFIER, APP_ORGANIZATION, APP_NAME)?;
    Some(project_dirs.config_dir().to_path_buf())
}
