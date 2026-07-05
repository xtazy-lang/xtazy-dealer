use crate::compiler_contract::CompilerBackend;
use crate::error::{DealerError, DealerResult};
use crate::project::{ProjectRoot, resolve_dependencies};
use crate::state::DealerState;
use crate::toolchain::ToolchainSelection;

pub(crate) fn run_check_or_exit(project: &ProjectRoot, toolchain: &ToolchainSelection) {
    let compiler = toolchain.compiler_backend();
    let state = DealerState::from_home(toolchain.dealer_home.clone());
    exit_on_error(run_check(project, &compiler, &state));
    println!("xtazy project check passed");
}

fn run_check(
    project: &ProjectRoot,
    compiler: &dyn CompilerBackend,
    state: &DealerState,
) -> DealerResult<()> {
    let deps = resolve_dependencies(project, state).map_err(|error| {
        DealerError::PackageResolution(format!("failed to resolve dependencies: {error}"))
    })?;
    compiler.check(project, &deps)
}

fn exit_on_error(result: DealerResult<()>) {
    if let Err(error) = result {
        eprintln!("dealer: {error}");
        std::process::exit(1);
    }
}
