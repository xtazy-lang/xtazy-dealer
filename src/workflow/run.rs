use std::process::Command;

use crate::cli::BuildMode;
use crate::compiler_contract::CompilerBackend;
use crate::error::{DealerError, DealerResult};
use crate::project::ProjectRoot;
use crate::state::DealerState;
use crate::toolchain::ToolchainSelection;
use crate::workflow::build::{is_app_root, run_build};

pub(crate) fn run_project_or_exit(
    project: &ProjectRoot,
    toolchain: &ToolchainSelection,
    mode: BuildMode,
) {
    let compiler = toolchain.compiler_backend();
    let state = DealerState::from_home(toolchain.dealer_home.clone());
    match run_project(project, toolchain, mode, &compiler, &state) {
        Ok(status_code) => std::process::exit(status_code),
        Err(error) => {
            eprintln!("dealer: {error}");
            std::process::exit(1);
        }
    }
}

pub(crate) fn run_project(
    project: &ProjectRoot,
    toolchain: &ToolchainSelection,
    mode: BuildMode,
    compiler: &dyn CompilerBackend,
    state: &DealerState,
) -> DealerResult<i32> {
    if !is_app_root(project) {
        return Err(DealerError::Backend(format!(
            "dealer run requires app.x; '{}' is a package root",
            project.root_file.display()
        )));
    }

    let product_path = run_build(project, toolchain, mode, compiler, state)?;
    let status = Command::new(&product_path)
        .status()
        .map_err(|error| DealerError::io(&product_path, error))?;
    Ok(status.code().unwrap_or(1))
}
