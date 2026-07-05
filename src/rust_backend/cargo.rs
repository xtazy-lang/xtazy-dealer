use std::path::{Path, PathBuf};
use std::process::Command;

use crate::cli::BuildMode;
use crate::error::{DealerError, DealerResult, process_output_message};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ToolSource {
    PinnedToolchain,
    DevelopmentFallback,
}

impl ToolSource {
    pub(crate) fn as_metadata_value(self) -> &'static str {
        match self {
            Self::PinnedToolchain => "pinned_toolchain",
            Self::DevelopmentFallback => "development_fallback",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::PinnedToolchain => "pinned xtazy toolchain",
            Self::DevelopmentFallback => "development fallback",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RustBackend {
    pub(crate) cargo_path: PathBuf,
    pub(crate) source: ToolSource,
}

impl RustBackend {
    pub(crate) fn pinned_toolchain(path: impl Into<PathBuf>) -> Self {
        Self {
            cargo_path: path.into(),
            source: ToolSource::PinnedToolchain,
        }
    }

    pub(crate) fn development_system() -> Self {
        Self::from_cargo_path("cargo")
    }

    pub(crate) fn from_cargo_path(path: impl Into<PathBuf>) -> Self {
        Self {
            cargo_path: path.into(),
            source: ToolSource::DevelopmentFallback,
        }
    }

    pub(crate) fn build(&self, rust_dir: &Path, mode: BuildMode) -> DealerResult<()> {
        let mut command = Command::new(&self.cargo_path);
        command.args(["build", "--quiet"]);
        if mode == BuildMode::Prod {
            command.arg("--release");
        }

        let output = command.current_dir(rust_dir).output().map_err(|error| {
            DealerError::Backend(format!(
                "failed to start {} Cargo backend '{}' in '{}': {error}",
                self.source.label(),
                self.cargo_path.display(),
                rust_dir.display()
            ))
        })?;

        if output.status.success() {
            return Ok(());
        }

        Err(DealerError::Backend(process_output_message(
            output,
            &format!("{} Cargo backend", self.source.label()),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempProject;
    use std::fs;

    #[test]
    fn development_backend_is_explicit_system_cargo_fallback() {
        let backend = RustBackend::development_system();

        assert_eq!(backend.cargo_path, PathBuf::from("cargo"));
        assert_eq!(backend.source, ToolSource::DevelopmentFallback);
    }

    #[test]
    fn backend_reports_missing_cargo_executable() {
        let temp = TempProject::new("missing-cargo-backend");
        let backend = RustBackend::from_cargo_path(temp.path().join("missing-cargo"));

        let error = backend
            .build(temp.path(), BuildMode::Dev)
            .expect_err("missing cargo executable should fail");

        let message = error.to_string();
        assert!(message.contains("failed to start development fallback Cargo backend"));
        assert!(message.contains("missing-cargo"));
    }

    #[test]
    fn backend_reports_rust_build_failure_output() {
        let temp = TempProject::new("cargo-build-failure");
        fs::create_dir_all(temp.path().join("src")).expect("src dir should be created");
        fs::write(
            temp.path().join("Cargo.toml"),
            "[workspace]\n\n[package]\nname = \"dealer_backend_failure\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .expect("Cargo.toml should be written");
        fs::write(temp.path().join("src/main.rs"), "fn main() { let = ; }\n")
            .expect("invalid main.rs should be written");

        let error = RustBackend::development_system()
            .build(temp.path(), BuildMode::Dev)
            .expect_err("invalid generated Rust should fail");
        let message = error.to_string();

        assert!(
            message.contains("dealer_backend_failure"),
            "expected Cargo/Rust error output, got:\n{}",
            error
        );
    }
}
