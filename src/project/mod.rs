pub(crate) mod dealer_block;
pub(crate) mod dependency;
pub(crate) mod resolve;
pub(crate) mod root;

pub(crate) use resolve::{resolve_dependencies, resolve_xtazy_version};
pub(crate) use root::{ProjectRoot, validate_project_root};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::DealerState;
    use crate::test_support::TempProject;
    use dealer_block::{parse_project_file, strip_comments};
    use std::fs;

    #[test]
    fn validate_project_accepts_app_root() {
        let temp = TempProject::new("app-root");
        fs::write(temp.path().join("app.x"), "app Main 1.0.0\n")
            .expect("app root should be written");

        let project = validate_project_root(temp.path()).expect("app.x root should be valid");
        let expected_root = fs::canonicalize(temp.path()).expect("temp root should canonicalize");

        assert_eq!(project.root_file, expected_root.join("app.x"));
        assert_eq!(project.project_name, "Main");
    }

    #[test]
    fn validate_project_accepts_package_root() {
        let temp = TempProject::new("package-root");
        fs::write(temp.path().join("package.x"), "package Thing 1.0.0\n")
            .expect("package root should be written");

        let project = validate_project_root(temp.path()).expect("package.x root should be valid");
        let expected_root = fs::canonicalize(temp.path()).expect("temp root should canonicalize");

        assert_eq!(project.root_file, expected_root.join("package.x"));
        assert_eq!(project.project_name, "Thing");
    }

    #[test]
    fn validate_project_rejects_missing_root_file() {
        let temp = TempProject::new("missing-root");

        let error = validate_project_root(temp.path()).expect_err("missing root should fail");

        assert!(error.to_string().contains("expected app.x or package.x"));
    }

    #[test]
    fn validate_project_rejects_ambiguous_root_files() {
        let temp = TempProject::new("ambiguous-root");
        fs::write(temp.path().join("app.x"), "app Main 1.0.0\n")
            .expect("app root should be written");
        fs::write(temp.path().join("package.x"), "package Thing 1.0.0\n")
            .expect("package root should be written");

        let error = validate_project_root(temp.path()).expect_err("ambiguous root should fail");

        assert!(
            error
                .to_string()
                .contains("app.x and package.x cannot exist together")
        );
    }

    #[test]
    fn test_strip_comments_both_markers() {
        assert_eq!(strip_comments("foo 1.2.3 // comment"), "foo 1.2.3");
        assert_eq!(strip_comments("foo 1.2.3 # comment"), "foo 1.2.3");
        assert_eq!(strip_comments("foo 1.2.3 // comment # other"), "foo 1.2.3");
        assert_eq!(strip_comments("foo 1.2.3 # comment // other"), "foo 1.2.3");
        assert_eq!(
            strip_comments("foo \"path/with//#or//chars\" // comment"),
            "foo \"path/with//#or//chars\""
        );
    }

    #[test]
    fn test_duplicate_top_level_dealer_blocks() {
        let content =
            "app Main 1.0.0\n\tdealer\n\t\tdep1 \"1.0.0\"\n\tdealer\n\t\tdep2 \"2.0.0\"\n";
        let err = parse_project_file(content).unwrap_err();
        assert!(
            err.to_string()
                .contains("duplicate top-level dealer block found")
        );
    }

    #[test]
    fn test_nested_dealer_blocks_ignored() {
        let content = "app Main 1.0.0\n\tdealer\n\t\tdep1 \"1.0.0\"\n\tsome_block\n\t\tdealer\n\t\t\tmessage \"hi\"\n";
        let decl = parse_project_file(content).unwrap();
        assert_eq!(decl.dependencies.len(), 1);
        assert_eq!(decl.dependencies[0].name, "dep1");
    }

    #[test]
    fn test_has_complete_toolchain_missing_components() {
        let temp = TempProject::new("complete-check");
        let state = DealerState::for_home(temp.path().join("dealer-home"));
        let dir = state.toolchain_dir("0.1.0");
        fs::create_dir_all(&dir).unwrap();

        // 1. Missing xtazy.parts entirely
        assert!(!state.has_complete_toolchain("0.1.0"));

        // Write xtazy.parts
        let parts_file = dir.join(crate::constants::files::XTAZY_PARTS);
        fs::write(&parts_file, "xtazy 0.1.0\npiko 0.1.0 sha256:1\nrusttime 0.1.0 sha256:2\nstd 0.1.0 sha256:3\nrust 1.80.0 sha256:4\n").unwrap();

        // 2. Missing piko, rusttime, std, cargo, rustc
        assert!(!state.has_complete_toolchain("0.1.0"));

        // Create piko
        let piko = state
            .dealer_home
            .join("piko")
            .join("0.1.0")
            .join(format!("piko{}", std::env::consts::EXE_SUFFIX));
        fs::create_dir_all(piko.parent().unwrap()).unwrap();
        fs::write(&piko, "").unwrap();
        assert!(!state.has_complete_toolchain("0.1.0"));

        // Create rusttime
        let rusttime = state.dealer_home.join("rusttime").join("0.1.0");
        fs::create_dir_all(&rusttime).unwrap();
        assert!(!state.has_complete_toolchain("0.1.0"));

        // Create std
        let std_dir = state.dealer_home.join("std").join("0.1.0");
        fs::create_dir_all(&std_dir).unwrap();
        assert!(!state.has_complete_toolchain("0.1.0"));

        // Create cargo
        let backend_id = crate::state::rust_backend_id_for_version("1.80.0");
        let bin_dir = state.rust_backend_dir(&backend_id).join("bin");
        let cargo = bin_dir.join(format!("cargo{}", std::env::consts::EXE_SUFFIX));
        fs::create_dir_all(&bin_dir).unwrap();
        fs::write(&cargo, "").unwrap();
        assert!(!state.has_complete_toolchain("0.1.0"));

        // Create rustc
        let rustc = bin_dir.join(format!("rustc{}", std::env::consts::EXE_SUFFIX));
        fs::write(&rustc, "").unwrap();
        // Should still fail because lib/rustlib is missing
        assert!(!state.has_complete_toolchain("0.1.0"));

        // Create lib/rustlib (completing it)
        let rustlib_dir = state
            .rust_backend_dir(&backend_id)
            .join("lib")
            .join("rustlib");
        fs::create_dir_all(&rustlib_dir).unwrap();
        assert!(state.has_complete_toolchain("0.1.0"));
    }

    #[test]
    fn test_reject_dealer_block_at_indent_0() {
        let content = "app Bad 1.0.0\ndealer\n\tcore 1.0.0\n";
        let err = parse_project_file(content).unwrap_err();
        assert!(
            err.to_string()
                .contains("dealer block must be indented as a root child block")
        );
    }

    #[test]
    fn test_registry_dependency_does_not_resolve_to_local_workspace_package() {
        let temp = TempProject::new("bypass-check");
        let pkg_dir = temp.path().join("package").join("core");
        fs::create_dir_all(&pkg_dir).unwrap();
        fs::write(pkg_dir.join("package.x"), "package core 1.2.0\n").unwrap();

        let app_file = temp.path().join("app.x");
        fs::write(&app_file, "app MyApp 1.0.0\n\tdealer\n\t\tcore 1.2.0\n").unwrap();

        let state = DealerState::for_home(temp.path().join("dealer-home"));

        // Setup mock registry values
        crate::package::registry::set_mock_registry(
            Some("1.2.0 sha256:abc123hash https://example.invalid/core-1.2.0.tar.gz\n".to_string()),
            Some("1.2.0 sha256:abc123hash https://example.invalid/core-1.2.0.tar.gz".to_string()),
        );

        // Pre-populate the cache so that the resolver sees the package exists in the registry cache
        // and doesn't hit the network.
        let cache_source = state
            .cache_dir()
            .join("packages")
            .join("core")
            .join("1.2.0")
            .join("source");
        fs::create_dir_all(&cache_source).unwrap();
        fs::write(cache_source.join("package.x"), "package core 1.2.0\n").unwrap();

        let project = validate_project_root(temp.path()).unwrap();

        let resolved =
            resolve_dependencies(&project, &state).expect("resolution should succeed via cache");

        // Reset mock registry values
        crate::package::registry::set_mock_registry(None, None);

        let core_path = resolved
            .get("core")
            .expect("should contain core in resolved map");

        // Assert that the resolved path points to the cache source and NOT the local package/core directory
        assert_eq!(core_path, &cache_source);
        assert_ne!(core_path, &pkg_dir);
    }

    #[test]
    fn test_explicit_local_dependency_resolves_locally() {
        let temp = TempProject::new("explicit-local");
        let sub_dir = temp.path().join("my-sub");
        fs::create_dir_all(&sub_dir).unwrap();
        fs::write(sub_dir.join("package.x"), "package sub 1.0.0\n").unwrap();

        let app_file = temp.path().join("app.x");
        fs::write(
            &app_file,
            "app MyApp 1.0.0\n\tdealer\n\t\tsub \"./my-sub\"\n",
        )
        .unwrap();

        let state = DealerState::for_home(temp.path().join("dealer-home"));
        let project = validate_project_root(temp.path()).unwrap();

        let resolved = resolve_dependencies(&project, &state).unwrap();
        assert!(resolved.contains_key("sub"));
        let sub_path = resolved.get("sub").unwrap();
        assert!(sub_path.join("package.x").is_file());
    }

    #[test]
    fn test_version_required() {
        assert!(parse_project_file("app sample_app\n").is_err());
        assert!(parse_project_file("package sample_package\n").is_err());

        let decl = parse_project_file("app sample_app 1.0.0\n").unwrap();
        assert_eq!(decl.version, "1.0.0");
    }

    #[test]
    fn test_tab_only_indentation() {
        // Space-indented dealer block must fail
        assert!(parse_project_file("app Main 1.0.0\n    dealer\n\t\tcore 1.0.0\n").is_err());

        // Mixed space/tab indentation must fail
        assert!(parse_project_file("app Main 1.0.0\n \tdealer\n\t\tcore 1.0.0\n").is_err());

        // Space-indented comment-only line must fail
        assert!(
            parse_project_file("app Main 1.0.0\n\tdealer\n    // comment\n\t\tcore 1.0.0\n")
                .is_err()
        );

        // Tab-indented must succeed
        assert!(parse_project_file("app Main 1.0.0\n\tdealer\n\t\tcore 1.0.0\n").is_ok());
    }

    #[test]
    fn test_version_req_parsing_and_bounds() {
        use crate::project::dependency::VersionReq;

        let exact = VersionReq::parse("2.1.3").unwrap();
        assert_eq!(exact.min_bound(), (2, 1, 3));
        assert_eq!(exact.max_bound(), (3, 0, 0));

        let major_wild = VersionReq::parse("2.x.x").unwrap();
        assert_eq!(major_wild.min_bound(), (2, 0, 0));
        assert_eq!(major_wild.max_bound(), (3, 0, 0));

        let minor_wild = VersionReq::parse("2.2.x").unwrap();
        assert_eq!(minor_wild.min_bound(), (2, 2, 0));
        assert_eq!(minor_wild.max_bound(), (2, 3, 0));

        // Invalid version requirement parsing must fail
        assert!(VersionReq::parse("2.x.3").is_err());
        assert!(VersionReq::parse("2.1").is_err());
        assert!(VersionReq::parse("abc").is_err());
    }

    #[test]
    fn test_version_req_satisfies() {
        use crate::project::dependency::VersionReq;

        let exact = VersionReq::parse("2.1.3").unwrap();
        assert!(exact.satisfies("2.1.3"));
        assert!(exact.satisfies("2.5.0"));
        assert!(!exact.satisfies("3.0.0"));
        assert!(!exact.satisfies("2.1.2"));

        let minor_wild = VersionReq::parse("2.2.x").unwrap();
        assert!(minor_wild.satisfies("2.2.0"));
        assert!(minor_wild.satisfies("2.2.9"));
        assert!(!minor_wild.satisfies("2.3.0"));
        assert!(!minor_wild.satisfies("2.1.9"));
    }

    #[test]
    fn test_version_req_merge_all() {
        use crate::project::dependency::VersionReq;

        // Compatible
        let r1 = VersionReq::parse("2.x.x").unwrap();
        let r2 = VersionReq::parse("2.2.x").unwrap();
        let merged = VersionReq::merge_all(&[r1, r2]).unwrap();
        assert_eq!(merged.min, (2, 2, 0));
        assert_eq!(merged.max, (2, 3, 0));

        // Incompatible
        let r3 = VersionReq::parse("2.1.x").unwrap();
        let r4 = VersionReq::parse("2.2.x").unwrap();
        assert!(VersionReq::merge_all(&[r3, r4]).is_none());
    }

    #[test]
    fn test_recursive_local_dependencies() {
        let temp = TempProject::new("recursive-local");

        // Setup pkg C
        let c_dir = temp.path().join("c");
        fs::create_dir_all(&c_dir).unwrap();
        fs::write(c_dir.join("package.x"), "package c 2.1.0\n").unwrap();

        // Setup pkg B which depends on C
        let b_dir = temp.path().join("b");
        fs::create_dir_all(&b_dir).unwrap();
        fs::write(
            b_dir.join("package.x"),
            "package b 1.2.0\n\tdealer\n\t\tc \"../c\"\n",
        )
        .unwrap();

        // Setup app A which depends on B
        let app_file = temp.path().join("app.x");
        fs::write(app_file, "app a 1.0.0\n\tdealer\n\t\tb \"./b\"\n").unwrap();

        let state = DealerState::for_home(temp.path().join("dealer-home"));
        let project = validate_project_root(temp.path()).unwrap();
        let resolved = resolve_dependencies(&project, &state).unwrap();

        assert_eq!(resolved.len(), 2);
        assert_eq!(
            fs::canonicalize(resolved.get("b").unwrap()).unwrap(),
            fs::canonicalize(&b_dir).unwrap()
        );
        assert_eq!(
            fs::canonicalize(resolved.get("c").unwrap()).unwrap(),
            fs::canonicalize(&c_dir).unwrap()
        );
    }

    #[test]
    fn test_circular_dependency_detection() {
        let temp = TempProject::new("cyclic");

        // pkg A depends on B
        let a_dir = temp.path().join("a");
        fs::create_dir_all(&a_dir).unwrap();
        fs::write(
            a_dir.join("package.x"),
            "package a 1.0.0\n\tdealer\n\t\tb \"../b\"\n",
        )
        .unwrap();

        // pkg B depends on A
        let b_dir = temp.path().join("b");
        fs::create_dir_all(&b_dir).unwrap();
        fs::write(
            b_dir.join("package.x"),
            "package b 1.0.0\n\tdealer\n\t\ta \"../a\"\n",
        )
        .unwrap();

        // App depends on A
        let app_file = temp.path().join("app.x");
        fs::write(app_file, "app main 1.0.0\n\tdealer\n\t\ta \"./a\"\n").unwrap();

        let state = DealerState::for_home(temp.path().join("dealer-home"));
        let project = validate_project_root(temp.path()).unwrap();
        let err = resolve_dependencies(&project, &state).unwrap_err();
        assert!(err.to_string().contains("Circular dependency detected"));
    }

    #[test]
    fn test_project_lock_exclusivity() {
        let temp = TempProject::new("lock-exclusivity");
        let lock_file = temp.path().join("project.lock");

        // Acquire lock 1
        let f1 = fs::File::create(&lock_file).unwrap();
        let mut lock1 = fd_lock::RwLock::new(f1);
        let _guard1 = lock1.write().unwrap();

        // Trying to acquire lock 2 must fail immediately (try_write)
        let f2 = fs::File::create(&lock_file).unwrap();
        let mut lock2 = fd_lock::RwLock::new(f2);
        let guard2_res = lock2.try_write();
        assert!(guard2_res.is_err());
    }

    #[test]
    fn test_transitive_stale_dependency_removal() {
        use std::collections::HashMap;
        let temp = TempProject::new("stale-transitive");

        // Setup local pkg A which depends on core 1.x.x
        let a_dir = temp.path().join("a");
        fs::create_dir_all(&a_dir).unwrap();
        fs::write(
            a_dir.join("package.x"),
            "package a 1.0.0\n\tdealer\n\t\tcore 1.x.x\n",
        )
        .unwrap();

        // Setup local pkg B which depends on core 1.2.x
        let b_dir = temp.path().join("b");
        fs::create_dir_all(&b_dir).unwrap();
        fs::write(
            b_dir.join("package.x"),
            "package b 1.0.0\n\tdealer\n\t\tcore 1.2.x\n",
        )
        .unwrap();

        // App A depends on a and b
        let app_file = temp.path().join("app.x");
        fs::write(
            app_file,
            "app app_main 1.0.0\n\tdealer\n\t\ta \"./a\"\n\t\tb \"./b\"\n",
        )
        .unwrap();

        let state = DealerState::for_home(temp.path().join("dealer-home"));

        // Setup Mock Registry Maps
        let mut versions = HashMap::new();
        // core versions list
        versions.insert(
            "core".to_string(),
            "1.9.0 sha256:hashold https://example.invalid/core-1.9.0.tar.gz\n1.2.5 sha256:hashnew https://example.invalid/core-1.2.5.tar.gz\n".to_string()
        );
        // old_dep versions list
        versions.insert(
            "old_dep".to_string(),
            "1.0.0 sha256:hasholddep https://example.invalid/old_dep-1.0.0.tar.gz\n".to_string(),
        );
        // new_dep versions list
        versions.insert(
            "new_dep".to_string(),
            "1.0.0 sha256:hashnewdep https://example.invalid/new_dep-1.0.0.tar.gz\n".to_string(),
        );

        let mut metadata = HashMap::new();
        metadata.insert(
            "core-1.9.0".to_string(),
            "1.9.0 sha256:hashold https://example.invalid/core-1.9.0.tar.gz".to_string(),
        );
        metadata.insert(
            "core-1.2.5".to_string(),
            "1.2.5 sha256:hashnew https://example.invalid/core-1.2.5.tar.gz".to_string(),
        );
        metadata.insert(
            "old_dep-1.0.0".to_string(),
            "1.0.0 sha256:hasholddep https://example.invalid/old_dep-1.0.0.tar.gz".to_string(),
        );
        metadata.insert(
            "new_dep-1.0.0".to_string(),
            "1.0.0 sha256:hashnewdep https://example.invalid/new_dep-1.0.0.tar.gz".to_string(),
        );

        crate::package::registry::set_mock_registry_map(versions, metadata);

        // Pre-populate sources in Cache so they resolve without network download
        let cache_dir = state.cache_dir().join("packages");

        let core_1_9_0_src = cache_dir.join("core").join("1.9.0").join("source");
        fs::create_dir_all(&core_1_9_0_src).unwrap();
        fs::write(
            core_1_9_0_src.join("package.x"),
            "package core 1.9.0\n\tdealer\n\t\told_dep 1.0.0\n",
        )
        .unwrap();

        let core_1_2_5_src = cache_dir.join("core").join("1.2.5").join("source");
        fs::create_dir_all(&core_1_2_5_src).unwrap();
        fs::write(
            core_1_2_5_src.join("package.x"),
            "package core 1.2.5\n\tdealer\n\t\tnew_dep 1.0.0\n",
        )
        .unwrap();

        let old_dep_src = cache_dir.join("old_dep").join("1.0.0").join("source");
        fs::create_dir_all(&old_dep_src).unwrap();
        fs::write(old_dep_src.join("package.x"), "package old_dep 1.0.0\n").unwrap();

        let new_dep_src = cache_dir.join("new_dep").join("1.0.0").join("source");
        fs::create_dir_all(&new_dep_src).unwrap();
        fs::write(new_dep_src.join("package.x"), "package new_dep 1.0.0\n").unwrap();

        let project = validate_project_root(temp.path()).unwrap();
        let resolved = resolve_dependencies(&project, &state).unwrap();

        // Reset mock registry
        crate::package::registry::set_mock_registry_map(HashMap::new(), HashMap::new());

        // Assert core is resolved to 1.2.5 (highest satisfying compatibility with both 1.x.x and 1.2.x)
        assert_eq!(resolved.get("core").unwrap(), &core_1_2_5_src);

        // Assert new_dep is present
        assert!(resolved.contains_key("new_dep"));

        // Assert old_dep is ABSENT (no longer reachable, garbage-collected)
        assert!(!resolved.contains_key("old_dep"));
    }

    #[test]
    fn test_compatible_registry_constraints_resolve_highest() {
        use std::collections::HashMap;
        let temp = TempProject::new("compatible-highest");

        // Setup local pkg A which depends on core 2.x.x
        let a_dir = temp.path().join("a");
        fs::create_dir_all(&a_dir).unwrap();
        fs::write(
            a_dir.join("package.x"),
            "package a 1.0.0\n\tdealer\n\t\tcore 2.x.x\n",
        )
        .unwrap();

        // Setup local pkg B which depends on core 2.2.x
        let b_dir = temp.path().join("b");
        fs::create_dir_all(&b_dir).unwrap();
        fs::write(
            b_dir.join("package.x"),
            "package b 1.0.0\n\tdealer\n\t\tcore 2.2.x\n",
        )
        .unwrap();

        // App main depends on a and b
        let app_file = temp.path().join("app.x");
        fs::write(
            app_file,
            "app app_main 1.0.0\n\tdealer\n\t\ta \"./a\"\n\t\tb \"./b\"\n",
        )
        .unwrap();

        let state = DealerState::for_home(temp.path().join("dealer-home"));

        let mut versions = HashMap::new();
        versions.insert(
            "core".to_string(),
            "2.1.0 sha256:h1 https://example.invalid/c-2.1.0.tar.gz\n2.2.0 sha256:h2 https://example.invalid/c-2.2.0.tar.gz\n2.2.8 sha256:h3 https://example.invalid/c-2.2.8.tar.gz\n2.3.0 sha256:h4 https://example.invalid/c-2.3.0.tar.gz\n".to_string()
        );

        let mut metadata = HashMap::new();
        metadata.insert(
            "core-2.2.8".to_string(),
            "2.2.8 sha256:h3 https://example.invalid/c-2.2.8.tar.gz".to_string(),
        );
        metadata.insert(
            "core-2.3.0".to_string(),
            "2.3.0 sha256:h4 https://example.invalid/c-2.3.0.tar.gz".to_string(),
        );

        crate::package::registry::set_mock_registry_map(versions, metadata);

        // Pre-populate cache
        let cache_source = state
            .cache_dir()
            .join("packages")
            .join("core")
            .join("2.2.8")
            .join("source");
        fs::create_dir_all(&cache_source).unwrap();
        fs::write(cache_source.join("package.x"), "package core 2.2.8\n").unwrap();

        let cache_2_3_0 = state
            .cache_dir()
            .join("packages")
            .join("core")
            .join("2.3.0")
            .join("source");
        fs::create_dir_all(&cache_2_3_0).unwrap();
        fs::write(cache_2_3_0.join("package.x"), "package core 2.3.0\n").unwrap();

        let project = validate_project_root(temp.path()).unwrap();
        let resolved = resolve_dependencies(&project, &state).unwrap();

        crate::package::registry::set_mock_registry_map(HashMap::new(), HashMap::new());

        // Assert core is resolved to 2.2.8
        assert_eq!(resolved.get("core").unwrap(), &cache_source);
    }

    #[test]
    fn test_incompatible_registry_constraints_fail() {
        use std::collections::HashMap;
        let temp = TempProject::new("incompatible-fail");

        // Setup local pkg A which depends on core 2.1.x
        let a_dir = temp.path().join("a");
        fs::create_dir_all(&a_dir).unwrap();
        fs::write(
            a_dir.join("package.x"),
            "package a 1.0.0\n\tdealer\n\t\tcore 2.1.x\n",
        )
        .unwrap();

        // Setup local pkg B which depends on core 2.2.x
        let b_dir = temp.path().join("b");
        fs::create_dir_all(&b_dir).unwrap();
        fs::write(
            b_dir.join("package.x"),
            "package b 1.0.0\n\tdealer\n\t\tcore 2.2.x\n",
        )
        .unwrap();

        // App main depends on a and b
        let app_file = temp.path().join("app.x");
        fs::write(
            app_file,
            "app app_main 1.0.0\n\tdealer\n\t\ta \"./a\"\n\t\tb \"./b\"\n",
        )
        .unwrap();

        let state = DealerState::for_home(temp.path().join("dealer-home"));

        let mut versions = HashMap::new();
        versions.insert(
            "core".to_string(),
            "2.1.0 sha256:h1 https://example.invalid/c-2.1.0.tar.gz\n2.2.0 sha256:h2 https://example.invalid/c-2.2.0.tar.gz\n".to_string()
        );

        let mut metadata = HashMap::new();
        metadata.insert(
            "core-2.1.0".to_string(),
            "2.1.0 sha256:h1 https://example.invalid/c-2.1.0.tar.gz".to_string(),
        );
        metadata.insert(
            "core-2.2.0".to_string(),
            "2.2.0 sha256:h2 https://example.invalid/c-2.2.0.tar.gz".to_string(),
        );

        crate::package::registry::set_mock_registry_map(versions, metadata);

        // Pre-populate cache
        let cache_2_1_0 = state
            .cache_dir()
            .join("packages")
            .join("core")
            .join("2.1.0")
            .join("source");
        fs::create_dir_all(&cache_2_1_0).unwrap();
        fs::write(cache_2_1_0.join("package.x"), "package core 2.1.0\n").unwrap();

        let project = validate_project_root(temp.path()).unwrap();
        let res = resolve_dependencies(&project, &state);

        crate::package::registry::set_mock_registry_map(HashMap::new(), HashMap::new());

        assert!(res.is_err());
        assert!(
            res.unwrap_err()
                .to_string()
                .contains("Conflict: incompatible version requirements for package 'core'")
        );
    }
}
