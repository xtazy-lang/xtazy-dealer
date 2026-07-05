use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{DealerError, DealerResult};

pub(crate) const DEFAULT_TOOLCHAIN_VERSION: &str = "0.1.0";

pub(crate) fn rust_backend_id_for_version(rust_version: &str) -> String {
    format!("default-{}", rust_version)
}

pub(crate) fn resolve_dealer_home(user_home: Option<PathBuf>, workspace_root: &Path) -> PathBuf {
    user_home
        .map(|home| home.join(crate::constants::dirs::PROJECT_DEALER_DIR))
        .unwrap_or_else(|| workspace_root.join(crate::constants::dirs::PROJECT_DEALER_DIR))
}

#[derive(Debug, Clone)]
pub(crate) struct DealerState {
    pub(crate) dealer_home: PathBuf,
}

impl DealerState {
    pub(crate) fn from_process_env(workspace_root: &Path) -> Self {
        Self::from_home(resolve_dealer_home(
            directories::BaseDirs::new().map(|dirs| dirs.home_dir().to_path_buf()),
            workspace_root,
        ))
    }

    pub(crate) fn from_home(dealer_home: PathBuf) -> Self {
        Self { dealer_home }
    }

    #[cfg(test)]
    pub(crate) fn for_home(dealer_home: PathBuf) -> Self {
        Self::from_home(dealer_home)
    }

    pub(crate) fn config_dir(&self) -> PathBuf {
        self.dealer_home.join(crate::constants::dirs::CONFIG_DIR)
    }

    pub(crate) fn cache_dir(&self) -> PathBuf {
        self.dealer_home.join(crate::constants::dirs::CACHE_DIR)
    }

    pub(crate) fn xtazy_dir(&self) -> PathBuf {
        self.dealer_home.join(crate::constants::dirs::XTAZY_DIR)
    }

    pub(crate) fn versions_dir(&self) -> PathBuf {
        self.xtazy_dir().join(crate::constants::dirs::VERSIONS_DIR)
    }

    pub(crate) fn rust_dir(&self) -> PathBuf {
        self.dealer_home.join(crate::constants::dirs::RUST_DIR)
    }

    pub(crate) fn latest_xtazy_version_file(&self) -> PathBuf {
        self.config_dir()
            .join(crate::constants::files::LATEST_XTAZY_VERSION)
    }

    pub(crate) fn self_auto_update_file(&self) -> PathBuf {
        self.config_dir()
            .join(crate::constants::files::SELF_AUTO_UPDATE)
    }

    pub(crate) fn xtazy_auto_update_file(&self) -> PathBuf {
        self.config_dir()
            .join(crate::constants::files::XTAZY_AUTO_UPDATE)
    }

    pub(crate) fn last_self_update_check_file(&self) -> PathBuf {
        self.config_dir()
            .join(crate::constants::files::LAST_SELF_UPDATE_CHECK)
    }

    pub(crate) fn last_xtazy_update_check_file(&self) -> PathBuf {
        self.config_dir()
            .join(crate::constants::files::LAST_XTAZY_UPDATE_CHECK)
    }

    pub(crate) fn ensure_base_dirs(&self) -> DealerResult<()> {
        fs::create_dir_all(self.config_dir())
            .map_err(|error| DealerError::io(self.config_dir(), error))?;
        fs::create_dir_all(self.cache_dir())
            .map_err(|error| DealerError::io(self.cache_dir(), error))?;
        fs::create_dir_all(self.xtazy_dir())
            .map_err(|error| DealerError::io(self.xtazy_dir(), error))?;
        fs::create_dir_all(self.versions_dir())
            .map_err(|error| DealerError::io(self.versions_dir(), error))?;
        fs::create_dir_all(self.rust_dir())
            .map_err(|error| DealerError::io(self.rust_dir(), error))?;
        Ok(())
    }

    pub(crate) fn toolchain_dir(&self, version: &str) -> PathBuf {
        self.versions_dir().join(version)
    }

    pub(crate) fn rust_backend_dir(&self, backend: &str) -> PathBuf {
        self.rust_dir().join(backend)
    }

    pub(crate) fn has_complete_toolchain(&self, version: &str) -> bool {
        crate::toolchain::components::has_complete_toolchain(self, version)
    }
}
