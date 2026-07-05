pub(crate) mod build;
pub(crate) mod check;
pub(crate) mod clean;
pub(crate) mod fmt;
pub(crate) mod layout;
pub(crate) mod run;
pub(crate) mod test;

pub(crate) use build::run_build_or_exit;
pub(crate) use check::run_check_or_exit;
pub(crate) use clean::run_clean_or_exit;
pub(crate) use fmt::run_fmt_or_exit;
pub(crate) use run::run_project_or_exit;
pub(crate) use test::run_test_or_exit;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::BuildMode;
    use crate::compiler_contract::StaticCompilerBackend;
    use crate::project::validate_project_root;
    use crate::test_support::TempProject;
    use crate::toolchain::{ToolchainEnv, ToolchainSelection};
    use build::run_build;
    use clean::run_clean;
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;

    fn workspace_root() -> PathBuf {
        crate::support::workspace_root()
    }

    #[test]
    fn build_writes_generated_rust_under_dealer_and_final_artifact_under_product() {
        let temp = TempProject::new("build-layout");
        fs::write(
            temp.path().join("app.x"),
            "app DealerSmoke 1.0.0\n\tterminal.log\n\t\tmessage: \"dealer ok\"\n",
        )
        .expect("app root should be written");
        let project = validate_project_root(temp.path()).expect("app.x root should be valid");
        let workspace_root = workspace_root();
        let toolchain = ToolchainSelection::discover(
            &workspace_root,
            &ToolchainEnv::for_test(Some(temp.path().join("dealer-home")), None),
            "0.1.0".to_string(),
        )
        .expect("discover should pass");

        let compiler = StaticCompilerBackend;
        let state = crate::state::DealerState::from_home(toolchain.dealer_home.clone());

        let product_path = run_build(&project, &toolchain, BuildMode::Dev, &compiler, &state)
            .expect("build should pass");

        assert!(temp.path().join(".dealer/rust/Cargo.toml").is_file());
        assert!(temp.path().join(".dealer/rust/src/main.rs").is_file());
        assert!(temp.path().join(".dealer/xtazy/last-build.txt").is_file());
        assert!(
            !temp.path().join("Cargo.toml").exists(),
            "dealer must not write generated Cargo files next to app.x"
        );

        let build_state = fs::read_to_string(temp.path().join(".dealer/xtazy/last-build.txt"))
            .expect("build state should be readable");
        assert!(build_state.contains("backend_source=development_fallback"));
        assert!(build_state.contains("rusttime_source=development_fallback"));
        assert!(build_state.contains("toolchain_version=0.1.0"));

        assert!(product_path.is_file());

        let output = Command::new(product_path)
            .output()
            .expect("product artifact should run");
        assert!(
            output.status.success(),
            "expected product artifact to run successfully"
        );
        assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    }

    #[test]
    fn clean_removes_dealer_and_product_dirs() {
        let temp = TempProject::new("clean-layout");
        fs::write(temp.path().join("app.x"), "app CleanSmoke 1.0.0\n")
            .expect("app root should be written");
        fs::create_dir_all(temp.path().join(".dealer/rust")).expect("dealer dir should be made");
        fs::create_dir_all(temp.path().join("product")).expect("product dir should be made");
        let project = validate_project_root(temp.path()).expect("app.x root should be valid");

        run_clean(&project).expect("clean should pass");

        assert!(!temp.path().join(".dealer").exists());
        assert!(!temp.path().join("product").exists());
        assert!(temp.path().join("app.x").is_file());
    }

    #[test]
    fn run_project_builds_and_executes_app_artifact() {
        let temp = TempProject::new("run-app");
        fs::write(temp.path().join("app.x"), "app RunSmoke 1.0.0\n")
            .expect("app root should be written");
        let project = validate_project_root(temp.path()).expect("app.x root should be valid");
        let workspace_root = workspace_root();
        let toolchain = ToolchainSelection::discover(
            &workspace_root,
            &ToolchainEnv::for_test(Some(temp.path().join("dealer-home")), None),
            "0.1.0".to_string(),
        )
        .expect("discover should pass");
        let compiler = StaticCompilerBackend;
        let state = crate::state::DealerState::from_home(toolchain.dealer_home.clone());

        let status = run::run_project(&project, &toolchain, BuildMode::Dev, &compiler, &state)
            .expect("run should pass");

        assert_eq!(status, 0);
    }

    #[test]
    fn run_project_rejects_package_root() {
        let temp = TempProject::new("run-package");
        fs::write(temp.path().join("package.x"), "package Lib 1.0.0\n")
            .expect("package root should be written");
        let project = validate_project_root(temp.path()).expect("package.x root should be valid");
        let workspace_root = workspace_root();
        let toolchain = ToolchainSelection::discover(
            &workspace_root,
            &ToolchainEnv::for_test(Some(temp.path().join("dealer-home")), None),
            "0.1.0".to_string(),
        )
        .expect("discover should pass");
        let compiler = StaticCompilerBackend;
        let state = crate::state::DealerState::from_home(toolchain.dealer_home.clone());

        let error = run::run_project(&project, &toolchain, BuildMode::Dev, &compiler, &state)
            .expect_err("package run should fail");

        assert!(error.to_string().contains("requires app.x"));
    }
}
