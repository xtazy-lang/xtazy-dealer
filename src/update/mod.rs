pub(crate) mod self_update;
pub(crate) mod xtazy_update;

pub(crate) use self_update::run_self_update;
pub(crate) use xtazy_update::run_subcommand;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::AutoUpdateAction;
    use crate::state::DealerState;
    use crate::test_support::TempProject;

    fn state(temp: &TempProject) -> DealerState {
        DealerState::for_home(temp.path().join("dealer-home"))
    }

    #[test]
    fn auto_update_status_round_trips() {
        let temp = TempProject::new("xtazy-auto-update");
        let state = state(&temp);

        assert_eq!(
            xtazy_update::auto_update(AutoUpdateAction::Status, &state)
                .expect("status should pass"),
            "xtazy toolchain auto-update is disabled"
        );
        state.set_xtazy_auto_update_enabled(true).unwrap();
        assert_eq!(
            xtazy_update::auto_update(AutoUpdateAction::Status, &state)
                .expect("status should pass"),
            "xtazy toolchain auto-update is enabled"
        );
    }
}
