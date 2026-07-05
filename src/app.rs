use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::cli;

use crate::package::{
    run_cache_clean, run_install_package, run_outdated_packages, run_remove_package,
    run_update_packages,
};
use crate::project::{self, ProjectRoot, resolve_xtazy_version, validate_project_root};
use crate::scaffold;
use crate::state::DealerState;
use crate::support::net::fetch_url_string;
use crate::toolchain::{ToolchainEnv, ToolchainSelection, install_xtazy_composition};
use crate::update;
use crate::workflow;

pub fn run() {
    let args: Vec<String> = env::args().collect();
    let command = match cli::parse_args(&args) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(1);
        }
    };

    let workspace_root = workspace_root();
    run_preflight_checks(&command, &workspace_root);

    match command {
        cli::CommandKind::Version => {
            println!("dealer {}", env!("CARGO_PKG_VERSION"));
        }
        cli::CommandKind::Check {
            project: project_arg,
        } => {
            let project = validate_project_or_exit(&project_arg);
            let state = DealerState::from_process_env(&workspace_root);
            let resolved = resolve_and_ensure_toolchain(&project, &state);
            let toolchain = ToolchainSelection::discover(
                &workspace_root,
                &ToolchainEnv::from_process_env(),
                resolved,
            )
            .unwrap_or_else(|e| {
                eprintln!("dealer: {e}");
                std::process::exit(1);
            });
            workflow::run_check_or_exit(&project, &toolchain);
        }
        cli::CommandKind::Build {
            project: project_arg,
            mode,
        } => {
            let project = validate_project_or_exit(&project_arg);
            let state = DealerState::from_process_env(&workspace_root);
            let resolved = resolve_and_ensure_toolchain(&project, &state);
            let toolchain = ToolchainSelection::discover(
                &workspace_root,
                &ToolchainEnv::from_process_env(),
                resolved,
            )
            .unwrap_or_else(|e| {
                eprintln!("dealer: {e}");
                std::process::exit(1);
            });
            workflow::run_build_or_exit(&project, &toolchain, mode);
        }
        cli::CommandKind::Fmt {
            project: project_arg,
            check,
        } => {
            let project = validate_project_or_exit(&project_arg);
            let state = DealerState::from_process_env(&workspace_root);
            let resolved = resolve_and_ensure_toolchain(&project, &state);
            let toolchain = ToolchainSelection::discover(
                &workspace_root,
                &ToolchainEnv::from_process_env(),
                resolved,
            )
            .unwrap_or_else(|e| {
                eprintln!("dealer: {e}");
                std::process::exit(1);
            });
            workflow::run_fmt_or_exit(Path::new(&project_arg), &toolchain, check);
        }
        cli::CommandKind::Install { package } => {
            let project = validate_project_or_exit(".");
            let state = DealerState::from_process_env(&workspace_root);
            if let Some(pkg) = package {
                if let Err(e) = run_install_package(&project, &pkg, &state) {
                    eprintln!("dealer: {e}");
                    std::process::exit(1);
                }
            } else {
                match project::resolve_dependencies(&project, &state) {
                    Ok(_) => println!("Dependencies installed successfully."),
                    Err(e) => {
                        eprintln!("dealer: {e}");
                        std::process::exit(1);
                    }
                }
            }
        }
        cli::CommandKind::Update { package } => {
            let project = validate_project_or_exit(".");
            let state = DealerState::from_process_env(&workspace_root);
            if let Err(e) = run_update_packages(&project, package.as_deref(), &state) {
                eprintln!("dealer: {e}");
                std::process::exit(1);
            }
        }
        cli::CommandKind::Outdated => {
            let project = validate_project_or_exit(".");
            if let Err(e) = run_outdated_packages(&project) {
                eprintln!("dealer: {e}");
                std::process::exit(1);
            }
        }
        cli::CommandKind::Remove { package } => {
            let project = validate_project_or_exit(".");
            let state = DealerState::from_process_env(&workspace_root);
            if let Err(e) = run_remove_package(&project, &package, &state) {
                eprintln!("dealer: {e}");
                std::process::exit(1);
            }
        }
        cli::CommandKind::CacheClean => {
            let state = DealerState::from_process_env(&workspace_root);
            if let Err(e) = run_cache_clean(&state) {
                eprintln!("dealer: {e}");
                std::process::exit(1);
            }
            println!("Global cache cleaned successfully.");
        }
        cli::CommandKind::Run { project, mode } => {
            let project = validate_project_or_exit(&project);
            let state = DealerState::from_process_env(&workspace_root);
            let resolved = resolve_and_ensure_toolchain(&project, &state);
            let toolchain = ToolchainSelection::discover(
                &workspace_root,
                &ToolchainEnv::from_process_env(),
                resolved,
            )
            .unwrap_or_else(|e| {
                eprintln!("dealer: {e}");
                std::process::exit(1);
            });
            workflow::run_project_or_exit(&project, &toolchain, mode);
        }
        cli::CommandKind::Test { project } => {
            let project = validate_project_or_exit(&project);
            let state = DealerState::from_process_env(&workspace_root);
            let resolved = resolve_and_ensure_toolchain(&project, &state);
            let toolchain = ToolchainSelection::discover(
                &workspace_root,
                &ToolchainEnv::from_process_env(),
                resolved,
            )
            .unwrap_or_else(|e| {
                eprintln!("dealer: {e}");
                std::process::exit(1);
            });
            workflow::run_test_or_exit(&project, &toolchain);
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
        cli::CommandKind::Tooling { subcommand } => {
            match update::run_subcommand(subcommand, &workspace_root) {
                Ok(message) => println!("{message}"),
                Err(error) => {
                    eprintln!("dealer: {error}");
                    std::process::exit(1);
                }
            }
        }
        cli::CommandKind::SelfUpdate => {
            let state = DealerState::from_process_env(&workspace_root);
            match update::run_self_update(&state) {
                Ok(message) => println!("{message}"),
                Err(error) => {
                    eprintln!("dealer: {error}");
                    std::process::exit(1);
                }
            }
        }
        cli::CommandKind::SelfAutoUpdate { action } => {
            let state = DealerState::from_process_env(&workspace_root);
            match action {
                cli::AutoUpdateAction::On => {
                    if let Err(e) = state.set_self_auto_update_enabled(true) {
                        eprintln!("dealer: {e}");
                        std::process::exit(1);
                    }
                    println!("dealer self auto-update enabled");
                }
                cli::AutoUpdateAction::Off => {
                    if let Err(e) = state.set_self_auto_update_enabled(false) {
                        eprintln!("dealer: {e}");
                        std::process::exit(1);
                    }
                    println!("dealer self auto-update disabled");
                }
                cli::AutoUpdateAction::Status => {
                    println!(
                        "dealer self auto-update is {}",
                        if state.self_auto_update_enabled() {
                            "enabled"
                        } else {
                            "disabled"
                        }
                    );
                }
            }
        }
    }
}

fn validate_project_or_exit(project_arg: &str) -> ProjectRoot {
    match validate_project_root(Path::new(project_arg)) {
        Ok(project) => project,
        Err(message) => {
            eprintln!("dealer: {message}");
            std::process::exit(1);
        }
    }
}

pub(crate) fn workspace_root() -> PathBuf {
    crate::support::workspace_root()
}

fn run_preflight_checks(command: &cli::CommandKind, workspace_root: &Path) {
    let state = DealerState::from_process_env(workspace_root);

    let is_status = matches!(
        command,
        cli::CommandKind::Version
            | cli::CommandKind::SelfAutoUpdate {
                action: cli::AutoUpdateAction::Status
            }
            | cli::CommandKind::Tooling {
                subcommand: cli::ToolingSubcommand::AutoUpdate {
                    action: cli::AutoUpdateAction::Status
                }
            }
            | cli::CommandKind::Tooling {
                subcommand: cli::ToolingSubcommand::Doctor
            }
    );
    if is_status {
        return;
    }

    let is_explicit_self_update = matches!(command, cli::CommandKind::SelfUpdate);
    let is_explicit_xtazy_update = matches!(
        command,
        cli::CommandKind::Tooling {
            subcommand: cli::ToolingSubcommand::Update
        }
    );

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Self Update check
    if !is_explicit_self_update && !is_explicit_xtazy_update {
        let last_self_check = state.last_self_update_check().unwrap_or(0);
        if now - last_self_check > 3600 {
            let res = fetch_url_string(crate::constants::web::VERSION_TXT_URL);
            if let Ok(latest_version) = res {
                let current_version = env!("CARGO_PKG_VERSION");
                if latest_version != current_version {
                    if state.self_auto_update_enabled() {
                        println!("Updating dealer automatically...");
                        match update::run_self_update(&state) {
                            Ok(_) => {
                                let status = Command::new(std::env::current_exe().unwrap())
                                    .args(std::env::args().skip(1))
                                    .status()
                                    .unwrap();
                                std::process::exit(status.code().unwrap_or(0));
                            }
                            Err(e) => {
                                eprintln!("warning: automatic dealer update failed: {e}");
                            }
                        }
                    } else {
                        eprintln!(
                            "warning: newer dealer available: {} -> {}\n\
                             hint: run `dealer self update` or enable `dealer self auto-update on`",
                            current_version, latest_version
                        );
                    }
                }
                state.set_last_self_update_check(now).ok();
            }
        }
    }
}

fn resolve_and_ensure_toolchain(project: &ProjectRoot, state: &DealerState) -> String {
    let resolved = resolve_xtazy_version(project, state).unwrap_or_else(|e| {
        eprintln!("dealer: {e}");
        std::process::exit(1);
    });

    if !state.has_complete_toolchain(&resolved) {
        if state.xtazy_auto_update_enabled() {
            println!("Downloading toolchain {resolved}...");
            if let Err(e) = install_xtazy_composition(state, &resolved) {
                eprintln!("dealer: failed to install toolchain {resolved}: {e}");
                std::process::exit(1);
            }
        } else {
            eprintln!(
                "dealer: required xtazy composition '{resolved}' is missing. Run 'dealer xtazy update' to install it."
            );
            std::process::exit(1);
        }
    }

    resolved
}
