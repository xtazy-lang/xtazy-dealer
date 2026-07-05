pub(crate) mod config;
pub(crate) mod layout;
pub(crate) mod parts;

pub(crate) use layout::{
    DEFAULT_TOOLCHAIN_VERSION, DealerState, resolve_dealer_home, rust_backend_id_for_version,
};
pub(crate) use parts::{XtazyParts, parse_xtazy_parts};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempProject;
    use std::fs;

    #[test]
    fn latest_version_defaults_to_none() {
        let temp = TempProject::new("state-default");
        let state = DealerState::for_home(temp.path().join("dealer-home"));

        assert_eq!(state.latest_xtazy_version(), None);
    }

    #[test]
    fn latest_version_round_trips_through_config() {
        let temp = TempProject::new("state-active");
        let state = DealerState::for_home(temp.path().join("dealer-home"));

        state
            .set_latest_xtazy_version("0.2.0")
            .expect("latest version should be written");

        assert_eq!(state.latest_xtazy_version(), Some("0.2.0".to_string()));
        assert!(!state.config_dir().join("active-toolchain").exists());
        assert!(state.config_dir().is_dir());
        assert!(state.cache_dir().is_dir());
        assert!(state.xtazy_dir().is_dir());
        assert!(state.rust_dir().is_dir());
    }

    #[test]
    fn migration_safe_config_reads() {
        let temp = TempProject::new("migration-config");
        let state = DealerState::for_home(temp.path().join("dealer-home"));
        state.ensure_base_dirs().unwrap();
        fs::write(state.config_dir().join("auto-update"), "enabled\n").unwrap();
        assert!(state.self_auto_update_enabled());
        assert!(state.xtazy_auto_update_enabled());
    }
}
