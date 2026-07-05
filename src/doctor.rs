use crate::state::DealerState;
use crate::toolchain::ToolchainSelection;

pub(crate) fn report(toolchain: &ToolchainSelection) -> String {
    let state = DealerState::from_home(toolchain.dealer_home.clone());
    format!(
        "dealer xtazy doctor\n\
        {}={}\n\
        {}={}\n\
        {}={}\n\
        {}={}\n\
        {}={}\n\
        {}={}\n\
        {}={}\n\
        {}={}\n\
        {}={}\n\
        {}={}\n\
        {}={}\n\
        {}={}\n\
        {}={}\n\
        {}={}\n\
        {}={}\n\
        {}={}\n\
        {}={}\n\
        {}={}\n\
        {}={}",
        crate::constants::metadata::KEY_DEALER_HOME,
        toolchain.dealer_home.display(),
        crate::constants::metadata::KEY_SELF_AUTO_UPDATE,
        if state.self_auto_update_enabled() {
            "enabled"
        } else {
            "disabled"
        },
        crate::constants::metadata::KEY_XTAZY_AUTO_UPDATE,
        if state.xtazy_auto_update_enabled() {
            "enabled"
        } else {
            "disabled"
        },
        crate::constants::metadata::KEY_TOOLCHAIN_VERSION,
        toolchain.version,
        crate::constants::metadata::KEY_TOOLCHAIN_DIR,
        toolchain.toolchain_dir.display(),
        crate::constants::metadata::KEY_XTAZY_TOOLCHAIN_COMPLETE,
        state.has_complete_toolchain(&toolchain.version),
        crate::constants::metadata::KEY_COMPILER_SOURCE,
        toolchain.compiler_source.as_metadata_value(),
        crate::constants::metadata::KEY_COMPILER_PATH,
        toolchain.compiler_path.display(),
        crate::constants::metadata::KEY_COMPILER_EXISTS,
        toolchain.compiler_path.is_file(),
        crate::constants::metadata::KEY_RUST_BACKEND_ID,
        toolchain.rust_backend_id,
        crate::constants::metadata::KEY_RUST_BACKEND_DIR,
        toolchain.rust_backend_dir.display(),
        crate::constants::metadata::KEY_RUST_BACKEND_SOURCE,
        toolchain.backend_source().as_metadata_value(),
        crate::constants::metadata::KEY_CARGO_PATH,
        toolchain.backend.cargo_path.display(),
        crate::constants::metadata::KEY_RUSTTIME_SOURCE,
        toolchain.rusttime_source.as_metadata_value(),
        crate::constants::metadata::KEY_RUSTTIME_PATH,
        toolchain.rusttime_path.display(),
        crate::constants::metadata::KEY_RUSTTIME_EXISTS,
        toolchain.rusttime_path.is_dir(),
        crate::constants::metadata::KEY_STD_SOURCE,
        toolchain.std_source.as_metadata_value(),
        crate::constants::metadata::KEY_STD_PATH,
        toolchain.std_path.display(),
        crate::constants::metadata::KEY_STD_EXISTS,
        toolchain.std_path.is_dir(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::toolchain::{ToolchainEnv, ToolchainSelection};

    #[test]
    fn report_includes_selected_toolchain_paths() {
        let workspace_root = crate::support::workspace_root();
        let toolchain = ToolchainSelection::discover(
            &workspace_root,
            &ToolchainEnv::for_test(Some(workspace_root.join(".dealer-test")), None),
            "0.1.0".to_string(),
        )
        .expect("discover should pass");

        let report = report(&toolchain);

        assert!(report.contains("dealer xtazy doctor"));
        assert!(report.contains(&format!(
            "{}={}",
            crate::constants::metadata::KEY_DEALER_HOME,
            toolchain.dealer_home.display()
        )));
        assert!(report.contains(&format!(
            "{}=disabled",
            crate::constants::metadata::KEY_SELF_AUTO_UPDATE
        )));
        assert!(report.contains(&format!(
            "{}=disabled",
            crate::constants::metadata::KEY_XTAZY_AUTO_UPDATE
        )));
        assert!(report.contains(&format!(
            "{}={}",
            crate::constants::metadata::KEY_TOOLCHAIN_VERSION,
            "0.1.0"
        )));
        assert!(report.contains(&format!(
            "{}=false",
            crate::constants::metadata::KEY_XTAZY_TOOLCHAIN_COMPLETE
        )));
        assert!(report.contains(&format!(
            "{}=",
            crate::constants::metadata::KEY_COMPILER_PATH
        )));
        assert!(report.contains(&format!(
            "{}=",
            crate::constants::metadata::KEY_COMPILER_EXISTS
        )));
        assert!(report.contains(&format!(
            "{}=",
            crate::constants::metadata::KEY_RUST_BACKEND_ID
        )));
        assert!(report.contains(&format!(
            "{}=",
            crate::constants::metadata::KEY_RUSTTIME_PATH
        )));
        assert!(report.contains(&format!("{}=", crate::constants::metadata::KEY_STD_PATH)));
    }
}
