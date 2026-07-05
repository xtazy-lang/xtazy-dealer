use std::path::Path;

use crate::compiler_contract::CompilerBackend;
use crate::error::DealerResult;
use crate::project::validate_project_root;
use crate::toolchain::ToolchainSelection;

pub(crate) fn run_fmt_or_exit(project_root: &Path, toolchain: &ToolchainSelection, check: bool) {
    let compiler = toolchain.compiler_backend();
    let project = match validate_project_root(project_root) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("dealer: {e}");
            std::process::exit(1);
        }
    };
    exit_on_error(compiler.fmt(
        &project.root_dir,
        &project.root_file,
        &project.project_name,
        check,
    ));
    if check {
        println!("Formatting check passed.");
    } else {
        println!("Formatted successfully.");
    }
}

fn exit_on_error(result: DealerResult<()>) {
    if let Err(error) = result {
        eprintln!("dealer: {error}");
        std::process::exit(1);
    }
}
