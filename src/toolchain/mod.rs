pub(crate) mod components;
pub(crate) mod discover;
pub(crate) mod install;

pub(crate) use discover::{ToolchainEnv, ToolchainSelection};
pub(crate) use install::install_xtazy_composition;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rust_backend::ToolSource;
    use crate::state::DealerState;
    use crate::test_support::TempProject;
    use std::fs;
    use std::path::PathBuf;

    fn workspace_root() -> PathBuf {
        crate::support::workspace_root()
    }

    fn make_xtazy_toolchain(
        state: &DealerState,
        version: &str,
        rust_version: &str,
    ) -> (PathBuf, PathBuf, PathBuf) {
        let dir = state.toolchain_dir(version);
        fs::create_dir_all(&dir).unwrap();

        let parts_file = dir.join(crate::constants::files::XTAZY_PARTS);
        let content = format!(
            "xtazy {}\npiko 0.1.0 sha256:123\nrusttime 0.1.0 sha256:456\nstd 0.1.0 sha256:789\nrust {} sha256:abc\n",
            version, rust_version
        );
        fs::write(&parts_file, content).unwrap();

        let piko = state
            .dealer_home
            .join("piko")
            .join("0.1.0")
            .join(format!("piko{}", std::env::consts::EXE_SUFFIX));
        let rusttime = state.dealer_home.join("rusttime").join("0.1.0");
        let std = state.dealer_home.join("std").join("0.1.0");

        fs::create_dir_all(piko.parent().unwrap()).unwrap();
        fs::write(&piko, "").unwrap();
        fs::create_dir_all(&rusttime).unwrap();
        fs::create_dir_all(&std).unwrap();

        (piko, rusttime, std)
    }

    fn make_rust_backend(state: &DealerState, rust_version: &str) -> (PathBuf, PathBuf) {
        let backend_id = crate::state::rust_backend_id_for_version(rust_version);
        let backend_dir = state.rust_backend_dir(&backend_id);
        let bin_dir = backend_dir.join("bin");
        let cargo = bin_dir.join(format!("cargo{}", std::env::consts::EXE_SUFFIX));
        let rustc = bin_dir.join(format!("rustc{}", std::env::consts::EXE_SUFFIX));
        fs::create_dir_all(&bin_dir).unwrap();
        fs::write(&cargo, "").unwrap();
        fs::write(&rustc, "").unwrap();
        fs::create_dir_all(backend_dir.join("lib").join("rustlib")).unwrap();
        (cargo, rustc)
    }

    #[test]
    fn discovery_uses_separate_xtazy_and_rust_backend_paths() {
        let temp = TempProject::new("separate-toolchains");
        let state = DealerState::for_home(temp.path().join("dealer-home"));
        let (piko, rusttime, _) = make_xtazy_toolchain(&state, "0.2.0", "1.80.0");
        let (cargo, _) = make_rust_backend(&state, "1.80.0");

        let selection = ToolchainSelection::discover(
            &workspace_root(),
            &ToolchainEnv::for_test(Some(state.dealer_home.clone()), None),
            "0.2.0".to_string(),
        )
        .expect("discover should pass");

        let expected_backend_id = crate::state::rust_backend_id_for_version("1.80.0");

        assert_eq!(selection.dealer_home, state.dealer_home);
        assert_eq!(selection.version, "0.2.0");
        assert_eq!(selection.toolchain_dir, state.toolchain_dir("0.2.0"));
        assert_eq!(selection.rust_backend_id, expected_backend_id);
        assert_eq!(
            selection.rust_backend_dir,
            state.rust_backend_dir(&expected_backend_id)
        );
        assert_eq!(selection.backend.cargo_path, cargo);
        assert_eq!(selection.backend.source, ToolSource::PinnedToolchain);
        assert_eq!(selection.compiler_path, piko);
        assert_eq!(selection.compiler_source, ToolSource::PinnedToolchain);
        assert_eq!(selection.rusttime_path, rusttime);
        assert_eq!(selection.rusttime_source, ToolSource::PinnedToolchain);
    }

    #[test]
    fn discovery_uses_development_fallback_when_xtazy_layout_is_incomplete() {
        let temp = TempProject::new("incomplete-xtazy-toolchain");
        let state = DealerState::for_home(temp.path().join("dealer-home"));
        let incomplete = state.toolchain_dir("0.2.0");
        fs::create_dir_all(&incomplete).expect("incomplete toolchain dir should be made");
        fs::write(incomplete.join("piko"), "").expect("piko marker should be written");

        let workspace_root = workspace_root();
        let selection = ToolchainSelection::discover(
            &workspace_root,
            &ToolchainEnv::for_test(Some(state.dealer_home), None),
            "0.2.0".to_string(),
        )
        .expect("discover should pass");

        assert_eq!(selection.rust_backend_id, "development");
        assert_eq!(selection.backend.cargo_path, PathBuf::from("cargo"));
        assert_eq!(selection.backend.source, ToolSource::DevelopmentFallback);
        assert_eq!(selection.compiler_source, ToolSource::DevelopmentFallback);
        assert_eq!(selection.rusttime_path, workspace_root.join("xtazy-std"));
        assert_eq!(selection.rusttime_source, ToolSource::DevelopmentFallback);
    }

    #[test]
    fn test_toolchain_env_from_process_env_unconditionally_allow_fallback_false() {
        let env = ToolchainEnv::from_process_env();
        assert!(!env.allow_fallback);
    }

    #[test]
    fn test_toolchain_env_from_process_env_ignores_env_var() {
        unsafe {
            std::env::set_var("DEALER_DEV_FALLBACK", "1");
        }
        let env = ToolchainEnv::from_process_env();
        unsafe {
            std::env::remove_var("DEALER_DEV_FALLBACK");
        }
        assert!(!env.allow_fallback);
    }
}
