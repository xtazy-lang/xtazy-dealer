use std::fs;
use std::io;
use std::path::Path;

use crate::error::{DealerError, DealerResult};
use crate::state::layout::DealerState;

impl DealerState {
    pub(crate) fn latest_xtazy_version(&self) -> Option<String> {
        read_trimmed(&self.latest_xtazy_version_file()).filter(|value| !value.is_empty())
    }

    pub(crate) fn set_latest_xtazy_version(&self, version: &str) -> DealerResult<()> {
        self.ensure_base_dirs()?;
        fs::write(self.latest_xtazy_version_file(), format!("{version}\n"))
            .map_err(|error| DealerError::io(self.latest_xtazy_version_file(), error))
    }

    pub(crate) fn self_auto_update_enabled(&self) -> bool {
        if let Some(val) = read_trimmed(&self.self_auto_update_file()) {
            val == "enabled"
        } else {
            // Migration check
            if let Some(old_val) = read_trimmed(&self.config_dir().join("auto-update")) {
                old_val == "enabled"
            } else {
                false
            }
        }
    }

    pub(crate) fn set_self_auto_update_enabled(&self, enabled: bool) -> DealerResult<()> {
        self.ensure_base_dirs()?;
        let value = if enabled { "enabled\n" } else { "disabled\n" };
        fs::write(self.self_auto_update_file(), value)
            .map_err(|error| DealerError::io(self.self_auto_update_file(), error))
    }

    pub(crate) fn xtazy_auto_update_enabled(&self) -> bool {
        if let Some(val) = read_trimmed(&self.xtazy_auto_update_file()) {
            val == "enabled"
        } else {
            // Migration check
            if let Some(old_val) = read_trimmed(&self.config_dir().join("auto-update")) {
                old_val == "enabled"
            } else {
                false
            }
        }
    }

    pub(crate) fn set_xtazy_auto_update_enabled(&self, enabled: bool) -> DealerResult<()> {
        self.ensure_base_dirs()?;
        let value = if enabled { "enabled\n" } else { "disabled\n" };
        fs::write(self.xtazy_auto_update_file(), value)
            .map_err(|error| DealerError::io(self.xtazy_auto_update_file(), error))
    }

    pub(crate) fn last_self_update_check(&self) -> Option<u64> {
        read_trimmed(&self.last_self_update_check_file()).and_then(|val| val.parse::<u64>().ok())
    }

    pub(crate) fn set_last_self_update_check(&self, timestamp: u64) -> DealerResult<()> {
        self.ensure_base_dirs()?;
        fs::write(
            self.last_self_update_check_file(),
            format!("{}\n", timestamp),
        )
        .map_err(|error| DealerError::io(self.last_self_update_check_file(), error))
    }

    pub(crate) fn last_xtazy_update_check(&self) -> Option<u64> {
        read_trimmed(&self.last_xtazy_update_check_file()).and_then(|val| val.parse::<u64>().ok())
    }

    pub(crate) fn set_last_xtazy_update_check(&self, timestamp: u64) -> DealerResult<()> {
        self.ensure_base_dirs()?;
        fs::write(
            self.last_xtazy_update_check_file(),
            format!("{}\n", timestamp),
        )
        .map_err(|error| DealerError::io(self.last_xtazy_update_check_file(), error))
    }
}

fn read_trimmed(path: &Path) -> Option<String> {
    match fs::read_to_string(path) {
        Ok(value) => Some(value.trim().to_string()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => None,
        Err(_) => None,
    }
}
