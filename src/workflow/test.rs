use crate::compiler_contract::CompilerBackend;
use crate::error::{DealerError, DealerResult};
use crate::project::{ProjectRoot, resolve_dependencies};
use crate::state::DealerState;
use crate::toolchain::ToolchainSelection;

pub(crate) fn run_test_or_exit(project: &ProjectRoot, toolchain: &ToolchainSelection) {
    let compiler = toolchain.compiler_backend();
    let state = DealerState::from_home(toolchain.dealer_home.clone());
    exit_on_error(run_test(project, &compiler, &state));
    println!("Tests completed successfully.");
}

fn run_test(
    project: &ProjectRoot,
    compiler: &dyn CompilerBackend,
    state: &DealerState,
) -> DealerResult<()> {
    let deps = resolve_dependencies(project, state).map_err(|error| {
        DealerError::PackageResolution(format!("failed to resolve dependencies: {error}"))
    })?;
    compiler.test(project, &deps)
}

fn exit_on_error(result: DealerResult<()>) {
    if let Err(error) = result {
        eprintln!("dealer: {error}");
        std::process::exit(1);
    }
}
