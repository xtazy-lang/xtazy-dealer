use std::path::Path;

use crate::cli::{AutoUpdateAction, XtazySubcommand};
use crate::error::{DealerError, DealerResult};
use crate::state::{DEFAULT_TOOLCHAIN_VERSION, DealerState};

pub(crate) fn run_subcommand(
    subcommand: XtazySubcommand,
    workspace_root: &Path,
) -> DealerResult<String> {
    let state = DealerState::from_process_env(workspace_root);
    match subcommand {
        XtazySubcommand::Install { version } => install(version, &state),
        XtazySubcommand::Update => update(&state),
        XtazySubcommand::AutoUpdate { action } => auto_update(action, &state),
        XtazySubcommand::UseVersion { version } => use_version(&version, &state),
        XtazySubcommand::Active => active(&state),
        XtazySubcommand::List => list(&state),
        XtazySubcommand::Remove { version } => remove(&version, &state),
    }
}

fn install(version: Option<String>, state: &DealerState) -> DealerResult<String> {
    let version = version.unwrap_or_else(|| DEFAULT_TOOLCHAIN_VERSION.to_string());
    if state.has_complete_toolchain(&version) {
        state.set_active_version(&version)?;
        return Ok(format!(
            "Xtazy toolchain {version} is already installed and active"
        ));
    }

    Err(DealerError::NotImplemented {
        feature: format!(
            "xtazy install {version} download flow; expected final target '{}'",
            state.toolchain_dir(&version).display()
        ),
    })
}

fn update(state: &DealerState) -> DealerResult<String> {
    Err(DealerError::NotImplemented {
        feature: format!(
            "xtazy update download flow for active toolchain {}",
            state.active_version()
        ),
    })
}

fn auto_update(action: Option<AutoUpdateAction>, state: &DealerState) -> DealerResult<String> {
    match action {
        Some(AutoUpdateAction::Off) => {
            state.set_auto_update_enabled(false)?;
            Ok("Xtazy toolchain auto-update disabled".to_string())
        }
        Some(AutoUpdateAction::Status) => Ok(format!(
            "Xtazy toolchain auto-update is {}",
            if state.auto_update_enabled() {
                "enabled"
            } else {
                "disabled"
            }
        )),
        None => {
            state.set_auto_update_enabled(true)?;
            Ok(
                "Xtazy toolchain auto-update enabled; update download flow is not implemented yet"
                    .to_string(),
            )
        }
    }
}

fn use_version(version: &str, state: &DealerState) -> DealerResult<String> {
    if !state.has_complete_toolchain(version) {
        return Err(DealerError::Backend(format!(
            "cannot activate Xtazy toolchain {version}: '{}' is missing or incomplete; expected piko, rusttime/, and std/",
            state.toolchain_dir(version).display()
        )));
    }
    state.set_active_version(version)?;
    Ok(format!("Active Xtazy toolchain set to {version}"))
}

fn active(state: &DealerState) -> DealerResult<String> {
    let version = state.active_version();
    Ok(format!(
        "active_xtazy_toolchain={version}\npath={}\ncomplete={}",
        state.toolchain_dir(&version).display(),
        state.has_complete_toolchain(&version)
    ))
}

fn list(state: &DealerState) -> DealerResult<String> {
    let installed = state.installed_toolchains()?;
    let active = state.active_version();
    if installed.is_empty() {
        return Ok(format!(
            "No Xtazy toolchains installed under {}",
            state.xtazy_dir().display()
        ));
    }

    let mut lines = Vec::new();
    for toolchain in installed {
        let active_marker = if toolchain.version == active {
            " active"
        } else {
            ""
        };
        let completeness = if toolchain.complete {
            "complete"
        } else {
            "incomplete"
        };
        lines.push(format!(
            "{} [{}]{} {}",
            toolchain.version,
            completeness,
            active_marker,
            toolchain.path.display()
        ));
    }
    Ok(lines.join("\n"))
}

fn remove(version: &str, state: &DealerState) -> DealerResult<String> {
    if version == state.active_version() {
        return Err(DealerError::Backend(format!(
            "cannot remove active Xtazy toolchain {version}; switch active version first"
        )));
    }
    state.remove_toolchain(version)?;
    Ok(format!("Removed Xtazy toolchain {version}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempProject;
    use std::fs;

    fn state(temp: &TempProject) -> DealerState {
        DealerState::for_home(temp.path().join("dealer-home"))
    }

    fn install_fake_toolchain(state: &DealerState, version: &str) {
        let dir = state.toolchain_dir(version);
        fs::create_dir_all(dir.join("rusttime")).expect("rusttime dir should be created");
        fs::create_dir_all(dir.join("std")).expect("std dir should be created");
        fs::write(dir.join("piko"), "").expect("piko marker should be written");
    }

    #[test]
    fn use_active_list_and_remove_local_toolchains() {
        let temp = TempProject::new("xtazy-local-toolchains");
        let state = state(&temp);
        install_fake_toolchain(&state, "0.2.0");
        install_fake_toolchain(&state, "0.3.0");

        assert_eq!(
            use_version("0.2.0", &state).expect("use should pass"),
            "Active Xtazy toolchain set to 0.2.0"
        );
        assert!(
            active(&state)
                .expect("active should pass")
                .contains("complete=true")
        );
        assert!(
            list(&state)
                .expect("list should pass")
                .contains("0.2.0 [complete] active")
        );

        assert!(remove("0.2.0", &state).is_err());
        assert_eq!(
            remove("0.3.0", &state).expect("remove should pass"),
            "Removed Xtazy toolchain 0.3.0"
        );
    }

    #[test]
    fn auto_update_status_round_trips() {
        let temp = TempProject::new("xtazy-auto-update");
        let state = state(&temp);

        assert_eq!(
            auto_update(Some(AutoUpdateAction::Status), &state).expect("status should pass"),
            "Xtazy toolchain auto-update is disabled"
        );
        assert!(
            auto_update(None, &state)
                .expect("enable should pass")
                .contains("enabled")
        );
        assert_eq!(
            auto_update(Some(AutoUpdateAction::Status), &state).expect("status should pass"),
            "Xtazy toolchain auto-update is enabled"
        );
    }
}
