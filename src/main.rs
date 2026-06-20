mod backend;
mod cli;
mod compiler;
mod doctor;
mod error;
mod messages;
mod names;
mod project;
mod scaffold;
mod state;
#[cfg(test)]
mod test_support;
mod toolchain;
mod workflow;
mod xtazy;

use std::env;
use std::path::{Path, PathBuf};

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = match cli::parse_args(&args) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(1);
        }
    };

    match command {
        cli::CommandKind::Version => {
            println!("dealer {}", env!("CARGO_PKG_VERSION"));
        }
        cli::CommandKind::Check {
            project: project_arg,
        } => {
            let project = validate_project_or_exit(&project_arg);
            let workspace_root = workspace_root();
            let toolchain = toolchain::ToolchainSelection::discover(
                &workspace_root,
                &toolchain::ToolchainEnv::from_process_env(),
            );
            workflow::run_check_or_exit(&project, &toolchain);
        }
        cli::CommandKind::Build {
            project: project_arg,
            mode,
        } => {
            let project = validate_project_or_exit(&project_arg);
            let workspace_root = workspace_root();
            let toolchain = toolchain::ToolchainSelection::discover(
                &workspace_root,
                &toolchain::ToolchainEnv::from_process_env(),
            );
            workflow::run_build_or_exit(&project, &toolchain, mode);
        }
        cli::CommandKind::Doctor => {
            let workspace_root = workspace_root();
            let toolchain = toolchain::ToolchainSelection::discover(
                &workspace_root,
                &toolchain::ToolchainEnv::from_process_env(),
            );
            println!("{}", doctor::report(&toolchain));
        }
        cli::CommandKind::Fmt { project, check } => not_implemented(&format!(
            "fmt{} for project '{}'",
            if check { " --check" } else { "" },
            project
        )),
        cli::CommandKind::Install { package, project } => {
            if let Some(package) = package {
                not_implemented(&format!(
                    "install package '{}' into project '{}'",
                    package, project
                ));
            } else {
                not_implemented(&format!("install dependencies for project '{}'", project));
            }
        }
        cli::CommandKind::Run { project, mode } => {
            let project = validate_project_or_exit(&project);
            let workspace_root = workspace_root();
            let toolchain = toolchain::ToolchainSelection::discover(
                &workspace_root,
                &toolchain::ToolchainEnv::from_process_env(),
            );
            workflow::run_project_or_exit(&project, &toolchain, mode);
        }
        cli::CommandKind::Clean { project } => {
            let project = validate_project_or_exit(&project);
            workflow::run_clean_or_exit(&project);
        }
        cli::CommandKind::Init { kind, path } => {
            let target = path.as_deref().unwrap_or(".");
            match scaffold::init_project(kind, Path::new(target)) {
                Ok(root_file) => println!("Created {}", root_file.display()),
                Err(error) => {
                    eprintln!("dealer: {error}");
                    std::process::exit(1);
                }
            }
        }
        cli::CommandKind::Xtazy { subcommand } => {
            let workspace_root = workspace_root();
            match xtazy::run_subcommand(subcommand, &workspace_root) {
                Ok(message) => println!("{message}"),
                Err(error) => {
                    eprintln!("dealer: {error}");
                    std::process::exit(1);
                }
            }
        }
        cli::CommandKind::SelfUpdate => not_implemented("self update"),
    }
}

fn not_implemented(feature: &str) {
    println!("dealer: {}", messages::not_implemented(feature));
}

fn validate_project_or_exit(project_arg: &str) -> project::ProjectRoot {
    match project::validate_project_root(Path::new(project_arg)) {
        Ok(project) => project,
        Err(message) => {
            eprintln!("dealer: {message}");
            std::process::exit(1);
        }
    }
}

pub(crate) fn workspace_root() -> PathBuf {
    // Development fallback root only. Product installs should use `~/.dealer/xtazy/<version>/`
    // via ToolchainSelection, not this source-tree path.
    let compiled_workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtazy-dealer should live inside workspace root at build time")
        .to_path_buf();
    if compiled_workspace.join("xtazy-dealer").is_dir() {
        return compiled_workspace;
    }

    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}
