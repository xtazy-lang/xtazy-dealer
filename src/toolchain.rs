use crate::backend::{RustBackend, ToolSource};
use crate::compiler::PikoExecutableBackend;
use crate::state::{self, DealerState};
use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub(crate) struct ToolchainEnv {
    dealer_home: Option<PathBuf>,
    user_home: Option<PathBuf>,
}

impl ToolchainEnv {
    pub(crate) fn from_process_env() -> Self {
        Self {
            dealer_home: None,
            user_home: directories::BaseDirs::new().map(|dirs| dirs.home_dir().to_path_buf()),
        }
    }

    #[cfg(test)]
    pub(crate) fn for_test(dealer_home: Option<PathBuf>, user_home: Option<PathBuf>) -> Self {
        Self {
            dealer_home,
            user_home,
        }
    }

    fn dealer_home(&self, workspace_root: &Path) -> PathBuf {
        self.dealer_home
            .clone()
            .unwrap_or_else(|| state::resolve_dealer_home(self.user_home.clone(), workspace_root))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ToolchainSelection {
    pub(crate) dealer_home: PathBuf,
    pub(crate) version: String,
    pub(crate) toolchain_dir: PathBuf,
    pub(crate) rust_backend_id: String,
    pub(crate) rust_backend_dir: PathBuf,
    pub(crate) backend: RustBackend,
    pub(crate) piko_path: PathBuf,
    pub(crate) piko_source: ToolSource,
    pub(crate) rusttime_path: PathBuf,
    pub(crate) rusttime_source: ToolSource,
}

impl ToolchainSelection {
    pub(crate) fn discover(workspace_root: &Path, toolchain_env: &ToolchainEnv) -> Self {
        let dealer_home = toolchain_env.dealer_home(workspace_root);
        let state = DealerState::from_home(dealer_home.clone());
        let version = state.active_version();
        let rust_backend_id = state.active_rust_backend();
        let toolchain_dir = state.toolchain_dir(&version);
        let rust_backend_dir = state.rust_backend_dir(&rust_backend_id);

        let pinned_cargo = rust_backend_dir
            .join("bin")
            .join(format!("cargo{}", env::consts::EXE_SUFFIX));
        let backend = if pinned_cargo.is_file() {
            RustBackend::pinned_toolchain(pinned_cargo)
        } else {
            RustBackend::development_system()
        };

        let xtazy_toolchain_ready = state::toolchain_is_complete(&toolchain_dir);
        let pinned_piko = toolchain_dir.join(format!("piko{}", env::consts::EXE_SUFFIX));
        let pinned_rusttime = toolchain_dir.join("rusttime");

        let (piko_path, piko_source) = if xtazy_toolchain_ready {
            (pinned_piko, ToolSource::PinnedToolchain)
        } else {
            (
                workspace_root
                    .join("target")
                    .join("debug")
                    .join(format!("piko{}", env::consts::EXE_SUFFIX)),
                ToolSource::DevelopmentFallback,
            )
        };

        let (rusttime_path, rusttime_source) = if xtazy_toolchain_ready {
            (pinned_rusttime, ToolSource::PinnedToolchain)
        } else {
            (
                workspace_root.join("xtazy-std"),
                ToolSource::DevelopmentFallback,
            )
        };

        Self {
            dealer_home,
            version,
            toolchain_dir,
            rust_backend_id,
            rust_backend_dir,
            backend,
            piko_path,
            piko_source,
            rusttime_path,
            rusttime_source,
        }
    }

    pub(crate) fn backend_source(&self) -> ToolSource {
        self.backend.source
    }

    pub(crate) fn compiler_backend(&self) -> PikoExecutableBackend {
        PikoExecutableBackend {
            piko_path: self.piko_path.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::ToolSource;
    use crate::test_support::TempProject;
    use std::fs;

    fn workspace_root() -> PathBuf {
        crate::workspace_root()
    }

    fn make_xtazy_toolchain(state: &DealerState, version: &str) -> (PathBuf, PathBuf, PathBuf) {
        let dir = state.toolchain_dir(version);
        let piko = dir.join(format!("piko{}", env::consts::EXE_SUFFIX));
        let rusttime = dir.join("rusttime");
        let std = dir.join("std");
        fs::create_dir_all(&rusttime).expect("rusttime dir should be created");
        fs::create_dir_all(&std).expect("std dir should be created");
        fs::write(&piko, "").expect("piko marker should be written");
        (piko, rusttime, std)
    }

    fn make_rust_backend(state: &DealerState, backend: &str) -> PathBuf {
        let cargo = state
            .rust_backend_dir(backend)
            .join("bin")
            .join(format!("cargo{}", env::consts::EXE_SUFFIX));
        fs::create_dir_all(cargo.parent().unwrap()).expect("rust backend bin dir should be made");
        fs::write(&cargo, "").expect("cargo marker should be written");
        cargo
    }

    #[test]
    fn discovery_uses_separate_xtazy_and_rust_backend_paths() {
        let temp = TempProject::new("separate-toolchains");
        let state = DealerState::for_home(temp.path().join("dealer-home"));
        state
            .set_active_version("0.2.0")
            .expect("active xtazy version should be written");
        state
            .set_active_rust_backend("rust-1")
            .expect("active rust backend should be written");
        let (piko, rusttime, _) = make_xtazy_toolchain(&state, "0.2.0");
        let cargo = make_rust_backend(&state, "rust-1");

        let selection = ToolchainSelection::discover(
            &workspace_root(),
            &ToolchainEnv::for_test(Some(state.dealer_home.clone()), None),
        );

        assert_eq!(selection.dealer_home, state.dealer_home);
        assert_eq!(selection.version, "0.2.0");
        assert_eq!(selection.toolchain_dir, state.toolchain_dir("0.2.0"));
        assert_eq!(selection.rust_backend_id, "rust-1");
        assert_eq!(selection.rust_backend_dir, state.rust_backend_dir("rust-1"));
        assert_eq!(selection.backend.cargo_path, cargo);
        assert_eq!(selection.backend.source, ToolSource::PinnedToolchain);
        assert_eq!(selection.piko_path, piko);
        assert_eq!(selection.piko_source, ToolSource::PinnedToolchain);
        assert_eq!(selection.rusttime_path, rusttime);
        assert_eq!(selection.rusttime_source, ToolSource::PinnedToolchain);
    }

    #[test]
    fn discovery_uses_development_fallback_when_xtazy_layout_is_incomplete() {
        let temp = TempProject::new("incomplete-xtazy-toolchain");
        let state = DealerState::for_home(temp.path().join("dealer-home"));
        state
            .set_active_version("0.2.0")
            .expect("active version should be written");
        let incomplete = state.toolchain_dir("0.2.0");
        fs::create_dir_all(&incomplete).expect("incomplete toolchain dir should be made");
        fs::write(incomplete.join("piko"), "").expect("piko marker should be written");

        let workspace_root = workspace_root();
        let selection = ToolchainSelection::discover(
            &workspace_root,
            &ToolchainEnv::for_test(Some(state.dealer_home), None),
        );

        assert_eq!(selection.backend.cargo_path, PathBuf::from("cargo"));
        assert_eq!(selection.backend.source, ToolSource::DevelopmentFallback);
        assert_eq!(selection.piko_source, ToolSource::DevelopmentFallback);
        assert_eq!(selection.rusttime_path, workspace_root.join("xtazy-std"));
        assert_eq!(selection.rusttime_source, ToolSource::DevelopmentFallback);
    }

    #[test]
    fn discovery_uses_active_toolchain_config() {
        let temp = TempProject::new("active-config-toolchain");
        let state = DealerState::for_home(temp.path().join("dealer-home"));
        state
            .set_active_version("active-test")
            .expect("active version should be written");

        let selection = ToolchainSelection::discover(
            &workspace_root(),
            &ToolchainEnv::for_test(Some(state.dealer_home.clone()), None),
        );

        assert_eq!(selection.version, "active-test");
        assert_eq!(selection.toolchain_dir, state.toolchain_dir("active-test"));
    }

    #[test]
    fn discovery_uses_system_cargo_when_active_rust_backend_is_missing() {
        let temp = TempProject::new("missing-rust-backend");
        let state = DealerState::for_home(temp.path().join("dealer-home"));
        state
            .set_active_rust_backend("missing-rust")
            .expect("active rust backend should be written");

        let selection = ToolchainSelection::discover(
            &workspace_root(),
            &ToolchainEnv::for_test(Some(state.dealer_home.clone()), None),
        );

        assert_eq!(selection.rust_backend_id, "missing-rust");
        assert_eq!(
            selection.rust_backend_dir,
            state.rust_backend_dir("missing-rust")
        );
        assert_eq!(selection.backend.cargo_path, PathBuf::from("cargo"));
        assert_eq!(selection.backend.source, ToolSource::DevelopmentFallback);
    }
}
