use std::path::Path;

use crate::cli::{AutoUpdateAction, ToolingSubcommand};
use crate::error::{DealerError, DealerResult};
use crate::state::DealerState;
use crate::support::net::fetch_url_string;
use crate::toolchain::{ToolchainEnv, ToolchainSelection, install_xtazy_composition};

pub(crate) fn run_subcommand(
    subcommand: ToolingSubcommand,
    workspace_root: &Path,
) -> DealerResult<String> {
    let state = DealerState::from_process_env(workspace_root);
    match subcommand {
        ToolingSubcommand::Update => update(&state, None),
        ToolingSubcommand::AutoUpdate { action } => auto_update(action, &state),
        ToolingSubcommand::Doctor => {
            let resolved = state
                .latest_xtazy_version()
                .unwrap_or_else(|| crate::state::DEFAULT_TOOLCHAIN_VERSION.to_string());
            let toolchain = ToolchainSelection::discover(
                workspace_root,
                &ToolchainEnv::from_process_env(),
                resolved,
            )
            .map_err(DealerError::Backend)?;
            Ok(crate::doctor::report(&toolchain))
        }
    }
}

pub(crate) fn update(state: &DealerState, explicit_version: Option<&str>) -> DealerResult<String> {
    let version = match explicit_version {
        Some(v) => v.to_string(),
        None => {
            let latest_url = crate::constants::web::XTAZY_LATEST_URL;
            let latest = fetch_url_string(latest_url).map_err(|e| {
                DealerError::Backend(format!("failed to fetch latest xtazy version: {e}"))
            })?;
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            state.set_latest_xtazy_version(&latest)?;
            state.set_last_xtazy_update_check(now)?;
            latest
        }
    };

    install_xtazy_composition(state, &version)?;

    Ok(format!(
        "xtazy toolchain updated to composition version {version}"
    ))
}

pub(crate) fn auto_update(action: AutoUpdateAction, state: &DealerState) -> DealerResult<String> {
    match action {
        AutoUpdateAction::On => {
            state.set_xtazy_auto_update_enabled(true)?;
            update(state, None)?;
            Ok("xtazy toolchain auto-update enabled and updated now".to_string())
        }
        AutoUpdateAction::Off => {
            state.set_xtazy_auto_update_enabled(false)?;
            Ok("xtazy toolchain auto-update disabled".to_string())
        }
        AutoUpdateAction::Status => Ok(format!(
            "xtazy toolchain auto-update is {}",
            if state.xtazy_auto_update_enabled() {
                "enabled"
            } else {
                "disabled"
            }
        )),
    }
}
