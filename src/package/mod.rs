pub(crate) mod edit;
pub(crate) mod git;
pub(crate) mod install;
pub(crate) mod local;
pub(crate) mod outdated;
pub(crate) mod registry;
pub(crate) mod remove;
pub(crate) mod semver;
pub(crate) mod update;

pub(crate) use install::run_install_package;
pub(crate) use outdated::run_outdated_packages;
pub(crate) use remove::{run_cache_clean, run_remove_package};
pub(crate) use update::run_update_packages;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::validate_project_root;
    use crate::state::DealerState;
    use crate::test_support::TempProject;
    use std::fs;
    use std::path::Path;
    use update::run_update_packages_internal;

    fn create_dummy_package(temp_path: &Path, name: &str) {
        let pkg_dir = temp_path.join(name);
        fs::create_dir_all(&pkg_dir).unwrap();
        fs::write(pkg_dir.join("package.x"), format!("package {name} 1.0.0\n")).unwrap();
    }

    #[test]
    fn test_install_creates_dealer_block_when_missing() {
        let temp = TempProject::new("install-missing-block");
        let root_file = temp.path().join("app.x");
        fs::write(&root_file, "app MyProj 1.0.0\n").unwrap();
        create_dummy_package(temp.path(), "my-dep");

        let state = DealerState::for_home(temp.path().join("dealer-home"));
        let project = validate_project_root(temp.path()).unwrap();

        run_install_package(&project, "./my-dep", &state).unwrap();

        let content = fs::read_to_string(&root_file).unwrap();
        assert!(content.contains("\tdealer"));
        assert!(content.contains("my-dep \"./my-dep\""));

        // Validate tab-only indentation on all non-empty lines
        for (i, line) in content.lines().enumerate() {
            if !line.trim().is_empty() {
                let first_non_ws = line.find(|c: char| !c.is_whitespace()).unwrap();
                let leading = &line[..first_non_ws];
                assert!(
                    !leading.contains(' '),
                    "Line {} has spaces in leading indentation: {:?}",
                    i + 1,
                    line
                );
            }
        }
    }

    #[test]
    fn test_install_appends_to_existing_block() {
        let temp = TempProject::new("install-append");
        let root_file = temp.path().join("app.x");
        fs::write(
            &root_file,
            "app MyProj 1.0.0\n\tdealer\n\t\tother-dep \"./other-dep\"\n",
        )
        .unwrap();
        create_dummy_package(temp.path(), "my-dep");
        create_dummy_package(temp.path(), "other-dep");

        let state = DealerState::for_home(temp.path().join("dealer-home"));
        let project = validate_project_root(temp.path()).unwrap();

        run_install_package(&project, "./my-dep", &state).unwrap();

        let content = fs::read_to_string(&root_file).unwrap();
        assert!(content.contains("other-dep \"./other-dep\""));
        assert!(content.contains("my-dep \"./my-dep\""));
    }

    #[test]
    fn test_install_replaces_existing_dependency_with_same_name() {
        let temp = TempProject::new("install-replace");
        let root_file = temp.path().join("app.x");
        fs::write(
            &root_file,
            "app MyProj 1.0.0\n\tdealer\n\t\tmy-dep \"./old-path\"\n",
        )
        .unwrap();
        create_dummy_package(temp.path(), "my-dep");
        create_dummy_package(temp.path(), "old-path");

        let state = DealerState::for_home(temp.path().join("dealer-home"));
        let project = validate_project_root(temp.path()).unwrap();

        run_install_package(&project, "./my-dep", &state).unwrap();

        let content = fs::read_to_string(&root_file).unwrap();
        assert!(content.contains("my-dep \"./my-dep\""));
        assert!(!content.contains("old-path"));
    }
    #[test]
    fn test_install_git_ssh_url() {
        let temp = TempProject::new("install-git-ssh");
        let root_file = temp.path().join("app.x");
        fs::write(&root_file, "app MyProj 1.0.0\n").unwrap();
        create_dummy_package(temp.path(), "core");

        let state = DealerState::for_home(temp.path().join("dealer-home"));
        let project = validate_project_root(temp.path()).unwrap();

        // This will write the dependency line but fail during resolve.
        // We catch the error and verify the file content was updated.
        let res = run_install_package(&project, "git@github.com:xtazy-lang/core.git", &state);
        assert!(res.is_err());

        let content = fs::read_to_string(&root_file).unwrap();
        assert!(content.contains("core \"git@github.com:xtazy-lang/core.git\" \"main\""));
    }
    #[test]
    fn test_remove_deletes_dependency_line() {
        let temp = TempProject::new("remove-dep");
        let root_file = temp.path().join("app.x");
        fs::write(
            &root_file,
            "app MyProj 1.0.0\n\tdealer\n\t\tmy-dep \"./my-dep\"\n\t\tother-dep \"./other-dep\"\n",
        )
        .unwrap();
        create_dummy_package(temp.path(), "other-dep");

        let state = DealerState::for_home(temp.path().join("dealer-home"));
        let project = validate_project_root(temp.path()).unwrap();

        run_remove_package(&project, "my-dep", &state).unwrap();

        let content = fs::read_to_string(&root_file).unwrap();
        assert!(!content.contains("my-dep"));
        assert!(content.contains("other-dep \"./other-dep\""));
    }

    #[test]
    fn test_duplicate_blocks_returns_error() {
        let temp = TempProject::new("duplicate-blocks-err");
        let root_file = temp.path().join("app.x");
        fs::write(&root_file, "app MyProj 1.0.0\n\tdealer\n\t\tmy-dep \"./my-dep\"\n\tdealer\n\t\tother-dep \"./other-dep\"\n").unwrap();

        let err = validate_project_root(temp.path()).unwrap_err();
        assert!(
            err.to_string()
                .contains("duplicate top-level dealer block found")
        );
    }

    #[test]
    fn test_nested_dealer_blocks_are_ignored() {
        let temp = TempProject::new("nested-ignored");
        let root_file = temp.path().join("app.x");
        fs::write(
            &root_file,
            "app MyProj 1.0.0\n\tsome_other\n\t\tdealer\n\t\t\tmy-dep \"./my-dep\"\n",
        )
        .unwrap();
        create_dummy_package(temp.path(), "my-dep");
        create_dummy_package(temp.path(), "new-dep");

        let state = DealerState::for_home(temp.path().join("dealer-home"));
        let project = validate_project_root(temp.path()).unwrap();

        run_install_package(&project, "./new-dep", &state).unwrap();

        let content = fs::read_to_string(&root_file).unwrap();
        assert!(content.contains("\n\tdealer\n"));
        assert!(content.contains("new-dep \"./new-dep\""));
    }

    #[test]
    fn test_update_registry_dependency_preserves_comments() {
        let temp = TempProject::new("update-registry-dep");
        let root_file = temp.path().join("app.x");
        fs::write(
            &root_file,
            "app MyProj 1.0.0\n\tdealer\n\t\tcore 1.0.0 // keep this comment\n",
        )
        .unwrap();
        create_dummy_package(temp.path(), "core");

        let state = DealerState::for_home(temp.path().join("dealer-home"));
        let project = validate_project_root(temp.path()).unwrap();

        run_update_packages_internal(
            &project,
            Some("core"),
            &state,
            |_url| Ok("1.1.0".to_string()),
            |_url| Ok(vec![]),
            |_, _| Ok(()),
        )
        .unwrap();

        let content = fs::read_to_string(&root_file).unwrap();
        assert!(content.contains("core 1.1.0 // keep this comment"));
    }

    #[test]
    fn test_update_git_dependency_preserves_comments() {
        let temp = TempProject::new("update-git-dep");
        let root_file = temp.path().join("app.x");
        fs::write(
            &root_file,
            "app MyProj 1.0.0\n\tdealer\n\t\tcore \"https://github.com/xtazy-lang/core.git\" v1.0.0 # comment\n",
        )
        .unwrap();
        create_dummy_package(temp.path(), "core");

        let state = DealerState::for_home(temp.path().join("dealer-home"));
        let project = validate_project_root(temp.path()).unwrap();

        run_update_packages_internal(
            &project,
            Some("core"),
            &state,
            |_url| Ok("".to_string()),
            |_url| {
                Ok(vec![
                    "refs/tags/v1.1.0".to_string(),
                    "refs/tags/v1.2.0".to_string(),
                ])
            },
            |_, _| Ok(()),
        )
        .unwrap();

        let content = fs::read_to_string(&root_file).unwrap();
        assert!(
            content.contains("core \"https://github.com/xtazy-lang/core.git\" 1.2.0 # comment")
        );
    }

    #[test]
    fn test_update_git_dependency_uses_semver_sorting() {
        let temp = TempProject::new("update-git-semver");
        let root_file = temp.path().join("app.x");
        fs::write(
            &root_file,
            "app MyProj 1.0.0\n\tdealer\n\t\tcore \"https://github.com/xtazy-lang/core.git\" v1.9.0\n",
        )
        .unwrap();
        create_dummy_package(temp.path(), "core");

        let state = DealerState::for_home(temp.path().join("dealer-home"));
        let project = validate_project_root(temp.path()).unwrap();

        run_update_packages_internal(
            &project,
            Some("core"),
            &state,
            |_url| Ok("".to_string()),
            |_url| {
                Ok(vec![
                    "refs/tags/v1.9.0".to_string(),
                    "refs/tags/v1.10.0".to_string(),
                ])
            },
            |_, _| Ok(()),
        )
        .unwrap();

        let content = fs::read_to_string(&root_file).unwrap();
        assert!(content.contains("core \"https://github.com/xtazy-lang/core.git\" 1.10.0"));
    }

    #[test]
    fn test_update_git_dependency_ignores_older_or_non_version_tags() {
        let temp = TempProject::new("update-git-ignore");
        let root_file = temp.path().join("app.x");
        fs::write(
            &root_file,
            "app MyProj 1.0.0\n\tdealer\n\t\tcore \"https://github.com/xtazy-lang/core.git\" v1.10.0\n",
        )
        .unwrap();
        create_dummy_package(temp.path(), "core");

        let state = DealerState::for_home(temp.path().join("dealer-home"));
        let project = validate_project_root(temp.path()).unwrap();

        run_update_packages_internal(
            &project,
            Some("core"),
            &state,
            |_url| Ok("".to_string()),
            |_url| {
                Ok(vec![
                    "refs/tags/v1.9.0".to_string(),
                    "refs/tags/v1.10.0-alpha".to_string(),
                    "refs/tags/main".to_string(),
                ])
            },
            |_, _| Ok(()),
        )
        .unwrap();

        let content = fs::read_to_string(&root_file).unwrap();
        assert!(content.contains("core \"https://github.com/xtazy-lang/core.git\" v1.10.0"));
    }

    #[test]
    fn test_parse_registry_metadata_valid() {
        let meta = "1.2.0 sha256:abc123hash https://example.invalid/core-1.2.0.tar.gz";
        let res = registry::parse_registry_metadata(meta, "1.2.0").unwrap();
        assert_eq!(res.0, "https://example.invalid/core-1.2.0.tar.gz");
        assert_eq!(res.1, "abc123hash");
    }

    #[test]
    fn test_parse_registry_metadata_mismatched_version() {
        let meta = "1.2.0 sha256:abc123hash https://example.invalid/core-1.2.0.tar.gz";
        let res = registry::parse_registry_metadata(meta, "1.3.0");
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("version mismatch"));
    }

    #[test]
    fn test_parse_registry_metadata_missing_sha_prefix() {
        let meta = "1.2.0 abc123hash https://example.invalid/core-1.2.0.tar.gz";
        let res = registry::parse_registry_metadata(meta, "1.2.0");
        assert!(res.is_err());
        assert!(
            res.unwrap_err()
                .to_string()
                .contains("hash must start with 'sha256:'")
        );
    }

    #[test]
    fn test_update_registry_pin_rewrites_to_highest_compatible() {
        let temp = TempProject::new("update-pin");
        let root_file = temp.path().join("app.x");
        fs::write(&root_file, "app MyProj 1.0.0\n\tdealer\n\t\tcore 2.1.3\n").unwrap();
        create_dummy_package(temp.path(), "core");

        let state = DealerState::for_home(temp.path().join("dealer-home"));
        let project = validate_project_root(temp.path()).unwrap();

        run_update_packages_internal(
            &project,
            Some("core"),
            &state,
            |_url| Ok("2.1.3 sha256:h1 url1\n2.2.0 sha256:h2 url2\n2.2.5 sha256:h3 url3\n3.0.0 sha256:h4 url4\n".to_string()),
            |_url| Ok(vec![]),
            |_, _| Ok(()),
        )
        .unwrap();

        let content = fs::read_to_string(&root_file).unwrap();
        // 2.2.5 is the highest compatible (>= 2.1.3 and < 3.0.0)
        assert!(content.contains("core 2.2.5"));
        assert!(!content.contains("core 3.0.0"));
    }

    #[test]
    fn test_update_registry_wildcard_remains_unchanged() {
        let temp = TempProject::new("update-wildcard");
        let root_file = temp.path().join("app.x");
        fs::write(
            &root_file,
            "app MyProj 1.0.0\n\tdealer\n\t\tcore 2.x.x\n\t\tcore2 2.2.x\n",
        )
        .unwrap();
        create_dummy_package(temp.path(), "core");
        create_dummy_package(temp.path(), "core2");

        let state = DealerState::for_home(temp.path().join("dealer-home"));
        let project = validate_project_root(temp.path()).unwrap();

        run_update_packages_internal(
            &project,
            None,
            &state,
            |_url| Ok("2.9.0 sha256:h1 url1\n".to_string()),
            |_url| Ok(vec![]),
            |_, _| Ok(()),
        )
        .unwrap();

        let content = fs::read_to_string(&root_file).unwrap();
        assert!(content.contains("core 2.x.x"));
        assert!(content.contains("core2 2.2.x"));
    }

    #[test]
    fn test_update_git_quoted_ref_unchanged() {
        let temp = TempProject::new("update-quoted");
        let root_file = temp.path().join("app.x");
        fs::write(
            &root_file,
            "app MyProj 1.0.0\n\tdealer\n\t\tcore \"https://github.com/xtazy-lang/core.git\" \"v1.0.0\"\n",
        )
        .unwrap();
        create_dummy_package(temp.path(), "core");

        let state = DealerState::for_home(temp.path().join("dealer-home"));
        let project = validate_project_root(temp.path()).unwrap();

        run_update_packages_internal(
            &project,
            Some("core"),
            &state,
            |_url| Ok("".to_string()),
            |_url| {
                Ok(vec![
                    "refs/tags/v1.0.0".to_string(),
                    "refs/tags/v1.2.0".to_string(),
                ])
            },
            |_, _| Ok(()),
        )
        .unwrap();

        let content = fs::read_to_string(&root_file).unwrap();
        // Quoted Git ref should remain unchanged even if remote has newer tags
        assert!(content.contains("\"v1.0.0\""));
        assert!(!content.contains("\"v1.2.0\""));
    }

    #[test]
    fn test_outdated_git_quoted_ref_skipped() {
        let temp = TempProject::new("outdated-quoted");
        let root_file = temp.path().join("app.x");
        fs::write(
            &root_file,
            "app MyProj 1.0.0\n\tdealer\n\t\tcore \"https://github.com/xtazy-lang/core.git\" \"v1.0.0\"\n",
        )
        .unwrap();
        create_dummy_package(temp.path(), "core");

        let project = validate_project_root(temp.path()).unwrap();
        // Simply call run_outdated_packages, it should complete successfully and not fail on quoted refs
        outdated::run_outdated_packages(&project).unwrap();
    }

    #[test]
    fn test_update_git_numeric_updates_without_v() {
        let temp = TempProject::new("update-git-numeric");
        let root_file = temp.path().join("app.x");
        fs::write(
            &root_file,
            "app MyProj 1.0.0\n\tdealer\n\t\tcore \"https://github.com/xtazy-lang/core.git\" 1.0.0\n",
        )
        .unwrap();
        create_dummy_package(temp.path(), "core");

        let state = DealerState::for_home(temp.path().join("dealer-home"));
        let project = validate_project_root(temp.path()).unwrap();

        run_update_packages_internal(
            &project,
            Some("core"),
            &state,
            |_url| Ok("".to_string()),
            |_url| {
                Ok(vec![
                    "refs/tags/v1.0.0".to_string(),
                    "refs/tags/v1.2.0".to_string(),
                ])
            },
            |_, _| Ok(()),
        )
        .unwrap();

        let content = fs::read_to_string(&root_file).unwrap();
        // It must write canonical Xtazy numeric syntax (1.2.0) without a leading "v"
        assert!(content.contains("core \"https://github.com/xtazy-lang/core.git\" 1.2.0"));
        assert!(!content.contains("v1.2.0"));
    }
}
