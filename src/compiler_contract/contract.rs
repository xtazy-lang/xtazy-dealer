use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::DealerResult;
use crate::project::ProjectRoot;

pub(crate) struct BuildRequestArgs {
    pub(crate) rust_output_dir: PathBuf,
    pub(crate) generated_package_name: String,
    pub(crate) rusttime_path: PathBuf,
}

pub(crate) trait CompilerBackend {
    fn check(&self, project: &ProjectRoot, deps: &HashMap<String, PathBuf>) -> DealerResult<()>;

    fn build(
        &self,
        project: &ProjectRoot,
        deps: &HashMap<String, PathBuf>,
        output_dir: &Path,
        rusttime_path: &Path,
    ) -> DealerResult<()>;

    fn test(&self, project: &ProjectRoot, deps: &HashMap<String, PathBuf>) -> DealerResult<()>;

    fn fmt(
        &self,
        project_root: &Path,
        entry_file: &Path,
        project_name: &str,
        check: bool,
    ) -> DealerResult<()>;
}

#[cfg(test)]
pub(crate) struct StaticCompilerBackend;

#[cfg(test)]
impl CompilerBackend for StaticCompilerBackend {
    fn check(&self, _project: &ProjectRoot, _deps: &HashMap<String, PathBuf>) -> DealerResult<()> {
        Ok(())
    }

    fn build(
        &self,
        project: &ProjectRoot,
        _deps: &HashMap<String, PathBuf>,
        output_dir: &Path,
        _rusttime_path: &Path,
    ) -> DealerResult<()> {
        std::fs::create_dir_all(output_dir.join("src"))
            .map_err(|error| crate::error::DealerError::io(output_dir.join("src"), error))?;
        std::fs::write(
            output_dir.join("Cargo.toml"),
            format!(
                "[workspace]\n\n[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
                crate::names::sanitize_package_name(&project.project_name)
            ),
        )
        .map_err(|error| crate::error::DealerError::io(output_dir.join("Cargo.toml"), error))?;
        std::fs::write(output_dir.join("src/main.rs"), "fn main() {}\n").map_err(|error| {
            crate::error::DealerError::io(output_dir.join("src/main.rs"), error)
        })?;
        Ok(())
    }

    fn test(&self, _project: &ProjectRoot, _deps: &HashMap<String, PathBuf>) -> DealerResult<()> {
        Ok(())
    }

    fn fmt(
        &self,
        _project_root: &Path,
        _entry_file: &Path,
        _project_name: &str,
        _check: bool,
    ) -> DealerResult<()> {
        Ok(())
    }
}
