use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{DealerError, DealerResult, process_output_message};

pub(crate) trait CompilerBackend {
    fn check(&self, entry_file: &Path, deps: &HashMap<String, PathBuf>) -> DealerResult<()>;

    fn build(
        &self,
        entry_file: &Path,
        deps: &HashMap<String, PathBuf>,
        output_dir: &Path,
        package_name: &str,
        rusttime_path: &Path,
    ) -> DealerResult<()>;

    fn metadata(&self, entry_file: &Path) -> DealerResult<ProjectMetadata>;
}

#[derive(Debug, Clone)]
pub(crate) struct PikoExecutableBackend {
    pub(crate) piko_path: PathBuf,
}

impl CompilerBackend for PikoExecutableBackend {
    fn check(&self, entry_file: &Path, deps: &HashMap<String, PathBuf>) -> DealerResult<()> {
        let mut command = self.command();
        command.arg("check").arg(entry_file);
        add_deps(&mut command, deps);
        run_empty(command, "piko check")
    }

    fn build(
        &self,
        entry_file: &Path,
        deps: &HashMap<String, PathBuf>,
        output_dir: &Path,
        package_name: &str,
        rusttime_path: &Path,
    ) -> DealerResult<()> {
        let mut command = self.command();
        command
            .arg("build")
            .arg(entry_file)
            .arg("--output")
            .arg(output_dir)
            .arg("--package-name")
            .arg(package_name)
            .arg("--rusttime-path")
            .arg(rusttime_path);
        add_deps(&mut command, deps);
        run_empty(command, "piko build")
    }

    fn metadata(&self, entry_file: &Path) -> DealerResult<ProjectMetadata> {
        let mut command = self.command();
        command.arg("metadata").arg(entry_file);
        let output = run_output(command, "piko metadata")?;
        serde_json::from_slice(&output.stdout).map_err(|error| {
            DealerError::Compiler(format!("failed to parse piko metadata JSON: {error}"))
        })
    }
}

impl PikoExecutableBackend {
    fn command(&self) -> Command {
        Command::new(&self.piko_path)
    }
}

fn add_deps(command: &mut Command, deps: &HashMap<String, PathBuf>) {
    for (name, path) in deps {
        command
            .arg("--dep")
            .arg(format!("{}={}", name, path.display()));
    }
}

fn run_empty(command: Command, label: &str) -> DealerResult<()> {
    let output = run_output(command, label)?;
    if output.status.success() {
        Ok(())
    } else {
        Err(DealerError::Compiler(process_output_message(output, label)))
    }
}

fn run_output(mut command: Command, label: &str) -> DealerResult<std::process::Output> {
    command
        .output()
        .map_err(|error| DealerError::Compiler(format!("failed to execute {label}: {error}")))
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct ProjectMetadata {
    pub(crate) project_type: String,
    pub(crate) name: String,
    pub(crate) dependencies: Vec<MetadataDependency>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct MetadataDependency {
    pub(crate) name: String,
    pub(crate) source_type: String,
    pub(crate) arg1: String,
    pub(crate) arg2: Option<String>,
}

#[cfg(test)]
pub(crate) struct StaticCompilerBackend {
    pub(crate) metadata: HashMap<PathBuf, ProjectMetadata>,
}

#[cfg(test)]
impl CompilerBackend for StaticCompilerBackend {
    fn check(&self, _entry_file: &Path, _deps: &HashMap<String, PathBuf>) -> DealerResult<()> {
        Ok(())
    }

    fn build(
        &self,
        _entry_file: &Path,
        _deps: &HashMap<String, PathBuf>,
        output_dir: &Path,
        package_name: &str,
        _rusttime_path: &Path,
    ) -> DealerResult<()> {
        std::fs::create_dir_all(output_dir.join("src"))
            .map_err(|error| DealerError::io(output_dir.join("src"), error))?;
        std::fs::write(
            output_dir.join("Cargo.toml"),
            format!(
                "[workspace]\n\n[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
                crate::names::sanitize_package_name(package_name)
            ),
        )
        .map_err(|error| DealerError::io(output_dir.join("Cargo.toml"), error))?;
        std::fs::write(output_dir.join("src/main.rs"), "fn main() {}\n")
            .map_err(|error| DealerError::io(output_dir.join("src/main.rs"), error))?;
        Ok(())
    }

    fn metadata(&self, entry_file: &Path) -> DealerResult<ProjectMetadata> {
        self.metadata.get(entry_file).cloned().ok_or_else(|| {
            DealerError::Compiler(format!(
                "missing test metadata for '{}'",
                entry_file.display()
            ))
        })
    }
}
