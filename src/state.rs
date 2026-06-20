use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::error::{DealerError, DealerResult};

pub(crate) const DEFAULT_TOOLCHAIN_VERSION: &str = "0.1.0";
pub(crate) const DEFAULT_RUST_BACKEND: &str = "default";

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
        self.dealer_home.join("config")
    }

    pub(crate) fn cache_dir(&self) -> PathBuf {
        self.dealer_home.join("cache")
    }

    pub(crate) fn xtazy_dir(&self) -> PathBuf {
        self.dealer_home.join("xtazy")
    }

    pub(crate) fn rust_dir(&self) -> PathBuf {
        self.dealer_home.join("rust")
    }

    pub(crate) fn active_toolchain_file(&self) -> PathBuf {
        self.config_dir().join("active-toolchain")
    }

    pub(crate) fn active_rust_backend_file(&self) -> PathBuf {
        self.config_dir().join("active-rust-backend")
    }

    pub(crate) fn auto_update_file(&self) -> PathBuf {
        self.config_dir().join("auto-update")
    }

    pub(crate) fn ensure_base_dirs(&self) -> DealerResult<()> {
        create_dir(&self.config_dir())?;
        create_dir(&self.cache_dir())?;
        create_dir(&self.xtazy_dir())?;
        create_dir(&self.rust_dir())?;
        Ok(())
    }

    pub(crate) fn active_version(&self) -> String {
        read_trimmed(&self.active_toolchain_file())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| DEFAULT_TOOLCHAIN_VERSION.to_string())
    }

    pub(crate) fn set_active_version(&self, version: &str) -> DealerResult<()> {
        self.ensure_base_dirs()?;
        fs::write(self.active_toolchain_file(), format!("{version}\n"))
            .map_err(|error| DealerError::io(self.active_toolchain_file(), error))
    }

    pub(crate) fn active_rust_backend(&self) -> String {
        read_trimmed(&self.active_rust_backend_file())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| DEFAULT_RUST_BACKEND.to_string())
    }

    #[cfg(test)]
    pub(crate) fn set_active_rust_backend(&self, backend: &str) -> DealerResult<()> {
        self.ensure_base_dirs()?;
        fs::write(self.active_rust_backend_file(), format!("{backend}\n"))
            .map_err(|error| DealerError::io(self.active_rust_backend_file(), error))
    }

    pub(crate) fn auto_update_enabled(&self) -> bool {
        matches!(
            read_trimmed(&self.auto_update_file()).as_deref(),
            Some("enabled")
        )
    }

    pub(crate) fn set_auto_update_enabled(&self, enabled: bool) -> DealerResult<()> {
        self.ensure_base_dirs()?;
        let value = if enabled { "enabled\n" } else { "disabled\n" };
        fs::write(self.auto_update_file(), value)
            .map_err(|error| DealerError::io(self.auto_update_file(), error))
    }

    pub(crate) fn installed_toolchains(&self) -> DealerResult<Vec<InstalledToolchain>> {
        let xtazy_dir = self.xtazy_dir();
        if !xtazy_dir.exists() {
            return Ok(Vec::new());
        }

        let mut toolchains = Vec::new();
        let entries =
            fs::read_dir(&xtazy_dir).map_err(|error| DealerError::io(&xtazy_dir, error))?;
        for entry in entries {
            let entry = entry.map_err(|error| DealerError::io(&xtazy_dir, error))?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let Some(version) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            toolchains.push(InstalledToolchain {
                version: version.to_string(),
                path: path.clone(),
                complete: toolchain_is_complete(&path),
            });
        }
        toolchains.sort_by(|left, right| left.version.cmp(&right.version));
        Ok(toolchains)
    }

    pub(crate) fn toolchain_dir(&self, version: &str) -> PathBuf {
        self.xtazy_dir().join(version)
    }

    pub(crate) fn rust_backend_dir(&self, backend: &str) -> PathBuf {
        self.rust_dir().join(backend)
    }

    pub(crate) fn has_complete_toolchain(&self, version: &str) -> bool {
        toolchain_is_complete(&self.toolchain_dir(version))
    }

    pub(crate) fn remove_toolchain(&self, version: &str) -> DealerResult<()> {
        let path = self.toolchain_dir(version);
        if path.exists() {
            fs::remove_dir_all(&path).map_err(|error| DealerError::io(&path, error))?;
        }
        Ok(())
    }
}

pub(crate) fn resolve_dealer_home(user_home: Option<PathBuf>, workspace_root: &Path) -> PathBuf {
    user_home
        .map(|home| home.join(".dealer"))
        .unwrap_or_else(|| workspace_root.join(".dealer"))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InstalledToolchain {
    pub(crate) version: String,
    pub(crate) path: PathBuf,
    pub(crate) complete: bool,
}

pub(crate) fn toolchain_is_complete(toolchain_dir: &Path) -> bool {
    toolchain_dir.join("piko").is_file()
        && toolchain_dir.join("rusttime").is_dir()
        && toolchain_dir.join("std").is_dir()
}

fn create_dir(path: &Path) -> DealerResult<()> {
    fs::create_dir_all(path).map_err(|error| DealerError::io(path, error))
}

fn read_trimmed(path: &Path) -> Option<String> {
    match fs::read_to_string(path) {
        Ok(value) => Some(value.trim().to_string()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempProject;

    #[test]
    fn active_version_defaults_to_mvp_version() {
        let temp = TempProject::new("state-default");
        let state = DealerState::for_home(temp.path().join("dealer-home"));

        assert_eq!(state.active_version(), DEFAULT_TOOLCHAIN_VERSION);
    }

    #[test]
    fn active_version_round_trips_through_config() {
        let temp = TempProject::new("state-active");
        let state = DealerState::for_home(temp.path().join("dealer-home"));

        state
            .set_active_version("0.2.0")
            .expect("active version should be written");

        assert_eq!(state.active_version(), "0.2.0");
        assert!(state.config_dir().is_dir());
        assert!(state.cache_dir().is_dir());
        assert!(state.xtazy_dir().is_dir());
        assert!(state.rust_dir().is_dir());
    }

    #[test]
    fn active_rust_backend_round_trips_through_config() {
        let temp = TempProject::new("state-rust-backend");
        let state = DealerState::for_home(temp.path().join("dealer-home"));

        assert_eq!(state.active_rust_backend(), DEFAULT_RUST_BACKEND);
        state
            .set_active_rust_backend("rust-1")
            .expect("active rust backend should be written");

        assert_eq!(state.active_rust_backend(), "rust-1");
    }

    #[test]
    fn installed_toolchains_mark_complete_layout_complete() {
        let temp = TempProject::new("state-installed");
        let state = DealerState::for_home(temp.path().join("dealer-home"));
        let version_dir = state.toolchain_dir("0.1.0");
        fs::create_dir_all(&version_dir).expect("toolchain dir should be created");
        fs::write(version_dir.join("piko"), "").expect("piko should be written");
        fs::create_dir_all(version_dir.join("rusttime")).expect("rusttime should be created");
        fs::create_dir_all(version_dir.join("std")).expect("std should be created");

        let installed = state
            .installed_toolchains()
            .expect("installed toolchains should be listed");

        assert_eq!(installed.len(), 1);
        assert_eq!(installed[0].version, "0.1.0");
        assert!(installed[0].complete);
    }
}
