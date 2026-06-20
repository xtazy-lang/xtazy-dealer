use crate::state::DealerState;
use crate::toolchain::ToolchainSelection;

pub(crate) fn report(toolchain: &ToolchainSelection) -> String {
    let state = DealerState::from_home(toolchain.dealer_home.clone());
    format!(
        "dealer doctor\n\
dealer_home={}\n\
auto_update={}\n\
toolchain_version={}\n\
toolchain_dir={}\n\
xtazy_toolchain_complete={}\n\
piko_source={}\n\
piko_path={}\n\
piko_exists={}\n\
rust_backend_id={}\n\
rust_backend_dir={}\n\
rust_backend_source={}\n\
cargo_path={}\n\
rusttime_source={}\n\
rusttime_path={}\n\
rusttime_exists={}",
        toolchain.dealer_home.display(),
        if state.auto_update_enabled() {
            "enabled"
        } else {
            "disabled"
        },
        toolchain.version,
        toolchain.toolchain_dir.display(),
        state.has_complete_toolchain(&toolchain.version),
        toolchain.piko_source.as_metadata_value(),
        toolchain.piko_path.display(),
        toolchain.piko_path.is_file(),
        toolchain.rust_backend_id,
        toolchain.rust_backend_dir.display(),
        toolchain.backend_source().as_metadata_value(),
        toolchain.backend.cargo_path.display(),
        toolchain.rusttime_source.as_metadata_value(),
        toolchain.rusttime_path.display(),
        toolchain.rusttime_path.is_dir(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::toolchain::{ToolchainEnv, ToolchainSelection};

    #[test]
    fn report_includes_selected_toolchain_paths() {
        let workspace_root = crate::workspace_root();
        let toolchain = ToolchainSelection::discover(
            &workspace_root,
            &ToolchainEnv::for_test(Some(workspace_root.join(".dealer-test")), None),
        );

        let report = report(&toolchain);

        assert!(report.contains("dealer doctor"));
        assert!(report.contains("auto_update=disabled"));
        assert!(report.contains("toolchain_version=0.1.0"));
        assert!(report.contains("xtazy_toolchain_complete=false"));
        assert!(report.contains("piko_path="));
        assert!(report.contains("piko_exists="));
        assert!(report.contains("rust_backend_id="));
        assert!(report.contains("rusttime_path="));
    }
}
