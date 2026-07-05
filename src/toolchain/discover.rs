use std::fs;
use std::path::{Path, PathBuf};

use crate::compiler_contract::ExecutableCompilerBackend;
use crate::rust_backend::{RustBackend, ToolSource};
use crate::state::{
    DealerState, parse_xtazy_parts, resolve_dealer_home, rust_backend_id_for_version,
};

#[derive(Debug, Clone)]
pub(crate) struct ToolchainEnv {
    dealer_home: Option<PathBuf>,
    user_home: Option<PathBuf>,
    pub(crate) allow_fallback: bool,
}

impl ToolchainEnv {
    pub(crate) fn from_process_env() -> Self {
        Self {
            dealer_home: None,
            user_home: directories::BaseDirs::new().map(|dirs| dirs.home_dir().to_path_buf()),
            allow_fallback: false,
        }
    }

    #[cfg(test)]
    pub(crate) fn for_test(dealer_home: Option<PathBuf>, user_home: Option<PathBuf>) -> Self {
        Self {
            dealer_home,
            user_home,
            allow_fallback: true,
        }
    }

    fn dealer_home(&self, workspace_root: &Path) -> PathBuf {
        self.dealer_home
            .clone()
            .unwrap_or_else(|| resolve_dealer_home(self.user_home.clone(), workspace_root))
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
    pub(crate) compiler_path: PathBuf,
    pub(crate) compiler_source: ToolSource,
    pub(crate) rusttime_path: PathBuf,
    pub(crate) rusttime_source: ToolSource,
    pub(crate) std_path: PathBuf,
    pub(crate) std_source: ToolSource,
}

impl ToolchainSelection {
    pub(crate) fn discover(
        workspace_root: &Path,
        toolchain_env: &ToolchainEnv,
        resolved_version: String,
    ) -> Result<Self, String> {
        let dealer_home = toolchain_env.dealer_home(workspace_root);
        let state = DealerState::from_home(dealer_home.clone());
        let version = resolved_version;
        let toolchain_dir = state.toolchain_dir(&version);

        let xtazy_toolchain_ready = state.has_complete_toolchain(&version);

        if xtazy_toolchain_ready {
            let parts_file = toolchain_dir.join(crate::constants::files::XTAZY_PARTS);
            let content = fs::read_to_string(&parts_file).map_err(|e| e.to_string())?;
            let parsed = parse_xtazy_parts(&content)?;
            let compiler_path = dealer_home
                .join(crate::constants::dirs::PIKO_COMPONENT_DIR)
                .join(&parsed.piko_version)
                .join(format!(
                    "{}{}",
                    crate::constants::executables::EXE_PIKO,
                    std::env::consts::EXE_SUFFIX
                ));
            let rusttime_path = dealer_home
                .join(crate::constants::dirs::RUSTTIME_DIR)
                .join(&parsed.rusttime_version);
            let std_path = dealer_home
                .join(crate::constants::dirs::STD_DIR)
                .join(&parsed.std_version);
            let rust_backend_id = rust_backend_id_for_version(&parsed.rust_version);

            let rust_backend_dir = state.rust_backend_dir(&rust_backend_id);
            let pinned_cargo =
                rust_backend_dir
                    .join(crate::constants::dirs::BIN_DIR)
                    .join(format!(
                        "{}{}",
                        crate::constants::executables::EXE_CARGO,
                        std::env::consts::EXE_SUFFIX
                    ));
            let backend = RustBackend::pinned_toolchain(pinned_cargo);

            Ok(Self {
                dealer_home,
                version,
                toolchain_dir,
                rust_backend_id,
                rust_backend_dir,
                backend,
                compiler_path,
                compiler_source: ToolSource::PinnedToolchain,
                rusttime_path,
                rusttime_source: ToolSource::PinnedToolchain,
                std_path,
                std_source: ToolSource::PinnedToolchain,
            })
        } else if toolchain_env.allow_fallback {
            let rust_backend_id = "development".to_string();
            let rust_backend_dir = state.rust_backend_dir(&rust_backend_id);
            let backend = RustBackend::development_system();

            let compiler_path = workspace_root.join("target").join("debug").join(format!(
                "{}{}",
                crate::constants::executables::EXE_PIKO,
                std::env::consts::EXE_SUFFIX
            ));
            let rusttime_path = workspace_root.join("xtazy-std");
            let std_path = workspace_root.join("xtazy-std");

            Ok(Self {
                dealer_home,
                version,
                toolchain_dir,
                rust_backend_id,
                rust_backend_dir,
                backend,
                compiler_path,
                compiler_source: ToolSource::DevelopmentFallback,
                rusttime_path,
                rusttime_source: ToolSource::DevelopmentFallback,
                std_path,
                std_source: ToolSource::DevelopmentFallback,
            })
        } else {
            Err(format!(
                "required xtazy composition '{version}' is missing. Run 'dealer xtazy update' to install it."
            ))
        }
    }

    pub(crate) fn backend_source(&self) -> ToolSource {
        self.backend.source
    }

    pub(crate) fn compiler_backend(&self) -> ExecutableCompilerBackend {
        ExecutableCompilerBackend {
            compiler_path: self.compiler_path.clone(),
        }
    }
}
