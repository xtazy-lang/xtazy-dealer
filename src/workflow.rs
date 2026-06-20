use crate::project::ProjectRoot;
use crate::toolchain::ToolchainSelection;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::cli::BuildMode;
use crate::compiler::CompilerBackend;
use crate::error::{DealerError, DealerResult};
use crate::names::sanitize_package_name;

struct BuildLayout {
    dealer_dir: PathBuf,
    rust_dir: PathBuf,
    product_dir: PathBuf,
    metadata_dir: PathBuf,
    logs_dir: PathBuf,
}

impl BuildLayout {
    fn for_project(project: &ProjectRoot) -> Self {
        let dealer_dir = project.root_dir.join(".dealer");
        Self {
            rust_dir: dealer_dir.join("rust"),
            product_dir: project.root_dir.join("product"),
            metadata_dir: dealer_dir.join("metadata"),
            logs_dir: dealer_dir.join("logs"),
            dealer_dir,
        }
    }
}

pub(crate) fn run_check_or_exit(project: &ProjectRoot, toolchain: &ToolchainSelection) {
    let compiler = toolchain.compiler_backend();
    exit_on_error(run_check(project, &compiler));
    println!("{}", crate::messages::check_passed());
}

pub(crate) fn run_build_or_exit(
    project: &ProjectRoot,
    toolchain: &ToolchainSelection,
    mode: BuildMode,
) {
    let compiler = toolchain.compiler_backend();
    match run_build(project, toolchain, mode, &compiler) {
        Ok(product_path) => println!("Product written to {}", product_path.display()),
        Err(error) => {
            eprintln!("dealer: {error}");
            std::process::exit(1);
        }
    }
}

pub(crate) fn run_project_or_exit(
    project: &ProjectRoot,
    toolchain: &ToolchainSelection,
    mode: BuildMode,
) {
    let compiler = toolchain.compiler_backend();
    match run_project(project, toolchain, mode, &compiler) {
        Ok(status_code) => std::process::exit(status_code),
        Err(error) => {
            eprintln!("dealer: {error}");
            std::process::exit(1);
        }
    }
}

pub(crate) fn run_clean_or_exit(project: &ProjectRoot) {
    exit_on_error(run_clean(project));
    println!(
        "Removed dealer build state for {}",
        project.root_dir.display()
    );
}

fn exit_on_error(result: DealerResult<()>) {
    if let Err(error) = result {
        eprintln!("dealer: {error}");
        std::process::exit(1);
    }
}

fn run_check(project: &ProjectRoot, compiler: &dyn CompilerBackend) -> DealerResult<()> {
    let deps = crate::project::resolve_dependencies(project, compiler).map_err(|error| {
        DealerError::PackageResolution(format!("failed to resolve dependencies: {error}"))
    })?;
    compiler.check(&project.root_file, &deps)
}

fn run_build(
    project: &ProjectRoot,
    toolchain: &ToolchainSelection,
    mode: BuildMode,
    compiler: &dyn CompilerBackend,
) -> DealerResult<PathBuf> {
    let layout = BuildLayout::for_project(project);
    prepare_build_folders(&layout).map_err(|error| DealerError::io(&layout.dealer_dir, error))?;

    let deps = crate::project::resolve_dependencies(project, compiler).map_err(|error| {
        DealerError::PackageResolution(format!("failed to resolve dependencies: {error}"))
    })?;

    compiler.build(
        &project.root_file,
        &deps,
        &layout.rust_dir,
        &project.package_name,
        &toolchain.rusttime_path,
    )?;

    toolchain.backend.build(&layout.rust_dir, mode)?;

    let is_executable = is_app_root(project);
    let product_path = copy_product_artifact(
        project,
        &layout.rust_dir,
        &layout.product_dir,
        is_executable,
        mode,
    )?;

    write_last_build_metadata(project, toolchain, &layout, &product_path)
        .map_err(|error| DealerError::io(layout.metadata_dir.join("last-build.txt"), error))?;
    Ok(product_path)
}

fn run_project(
    project: &ProjectRoot,
    toolchain: &ToolchainSelection,
    mode: BuildMode,
    compiler: &dyn CompilerBackend,
) -> DealerResult<i32> {
    if !is_app_root(project) {
        return Err(DealerError::Backend(format!(
            "dealer run requires app.x; '{}' is a package root",
            project.root_file.display()
        )));
    }

    let product_path = run_build(project, toolchain, mode, compiler)?;
    let status = Command::new(&product_path)
        .status()
        .map_err(|error| DealerError::io(&product_path, error))?;
    Ok(status.code().unwrap_or(1))
}

fn run_clean(project: &ProjectRoot) -> DealerResult<()> {
    let layout = BuildLayout::for_project(project);
    remove_dir_if_exists(&layout.dealer_dir)
        .map_err(|error| DealerError::io(&layout.dealer_dir, error))?;
    remove_dir_if_exists(&layout.product_dir)
        .map_err(|error| DealerError::io(&layout.product_dir, error))?;
    Ok(())
}

fn remove_dir_if_exists(path: &Path) -> io::Result<()> {
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}

fn is_app_root(project: &ProjectRoot) -> bool {
    project.root_file.file_name().and_then(|n| n.to_str()) == Some("app.x")
}

fn prepare_build_folders(layout: &BuildLayout) -> io::Result<()> {
    fs::create_dir_all(&layout.dealer_dir)?;
    fs::create_dir_all(&layout.rust_dir)?;
    fs::create_dir_all(&layout.product_dir)?;
    fs::create_dir_all(&layout.metadata_dir)?;
    fs::create_dir_all(&layout.logs_dir)?;
    Ok(())
}

fn copy_product_artifact(
    project: &ProjectRoot,
    rust_dir: &Path,
    product_dir: &Path,
    is_executable: bool,
    mode: BuildMode,
) -> DealerResult<PathBuf> {
    let package_name = sanitize_package_name(&project.package_name);
    let profile = match mode {
        BuildMode::Dev => "debug",
        BuildMode::Prod => "release",
    };
    let artifact = if is_executable {
        rust_dir
            .join("target")
            .join(profile)
            .join(format!("{package_name}{}", std::env::consts::EXE_SUFFIX))
    } else {
        rust_dir
            .join("target")
            .join(profile)
            .join(format!("lib{package_name}.rlib"))
    };

    if !artifact.is_file() {
        return Err(DealerError::Backend(format!(
            "dealer: expected backend artifact '{}' was not produced",
            artifact.display()
        )));
    }

    let product_name = artifact.file_name().unwrap_or_default();
    let product_path = product_dir.join(product_name);
    copy_file(&artifact, &product_path).map_err(|error| {
        DealerError::Backend(format!(
            "dealer: failed to copy product artifact to '{}': {error}",
            product_path.display()
        ))
    })?;
    Ok(product_path)
}

fn copy_file(from: &Path, to: &Path) -> io::Result<()> {
    if to.exists() {
        fs::remove_file(to)?;
    }
    fs::copy(from, to)?;
    Ok(())
}

fn write_last_build_metadata(
    project: &ProjectRoot,
    toolchain: &ToolchainSelection,
    layout: &BuildLayout,
    product_path: &Path,
) -> io::Result<()> {
    let metadata = format!(
        "project_root={}\nroot_file={}\nrust_dir={}\nproduct_path={}\ndealer_home={}\ntoolchain_version={}\ntoolchain_dir={}\ncompiler_source={}\npiko_path={}\nrust_backend_id={}\nrust_backend_dir={}\nbackend_source={}\ncargo_path={}\nrusttime_source={}\nrusttime_path={}\n",
        project.root_dir.display(),
        project.root_file.display(),
        layout.rust_dir.display(),
        product_path.display(),
        toolchain.dealer_home.display(),
        toolchain.version,
        toolchain.toolchain_dir.display(),
        toolchain.piko_source.as_metadata_value(),
        toolchain.piko_path.display(),
        toolchain.rust_backend_id,
        toolchain.rust_backend_dir.display(),
        toolchain.backend_source().as_metadata_value(),
        toolchain.backend.cargo_path.display(),
        toolchain.rusttime_source.as_metadata_value(),
        toolchain.rusttime_path.display(),
    );
    fs::write(layout.metadata_dir.join("last-build.txt"), metadata)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::{ProjectMetadata, StaticCompilerBackend};
    use crate::project::validate_project_root;
    use crate::test_support::TempProject;
    use crate::toolchain::{ToolchainEnv, ToolchainSelection};
    use std::collections::HashMap;
    use std::process::Command;

    #[test]
    fn build_writes_generated_rust_under_dealer_and_final_artifact_under_product() {
        let temp = TempProject::new("build-layout");
        fs::write(
            temp.path().join("app.x"),
            "app DealerSmoke\n\tterminal.log\n\t\tmessage: \"dealer ok\"\n",
        )
        .expect("app root should be written");
        let project = validate_project_root(temp.path()).expect("app.x root should be valid");
        let workspace_root = crate::workspace_root();
        let toolchain = ToolchainSelection::discover(
            &workspace_root,
            &ToolchainEnv::for_test(Some(temp.path().join("dealer-home")), None),
        );

        let compiler = StaticCompilerBackend {
            metadata: HashMap::from([(
                project.root_file.clone(),
                ProjectMetadata {
                    project_type: "app".to_string(),
                    name: "DealerSmoke".to_string(),
                    dependencies: Vec::new(),
                },
            )]),
        };

        let product_path =
            run_build(&project, &toolchain, BuildMode::Dev, &compiler).expect("build should pass");

        assert!(temp.path().join(".dealer/rust/Cargo.toml").is_file());
        assert!(temp.path().join(".dealer/rust/src/main.rs").is_file());
        assert!(
            temp.path()
                .join(".dealer/metadata/last-build.txt")
                .is_file()
        );
        assert!(temp.path().join(".dealer/logs").is_dir());
        assert!(
            !temp.path().join("Cargo.toml").exists(),
            "dealer must not write generated Cargo files next to app.x"
        );

        let metadata = fs::read_to_string(temp.path().join(".dealer/metadata/last-build.txt"))
            .expect("build metadata should be readable");
        assert!(metadata.contains("backend_source=development_fallback"));
        assert!(metadata.contains("rusttime_source=development_fallback"));
        assert!(metadata.contains("toolchain_version=0.1.0"));

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
        fs::write(temp.path().join("app.x"), "app CleanSmoke\n")
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
        fs::write(temp.path().join("app.x"), "app RunSmoke\n").expect("app root should be written");
        let project = validate_project_root(temp.path()).expect("app.x root should be valid");
        let workspace_root = crate::workspace_root();
        let toolchain = ToolchainSelection::discover(
            &workspace_root,
            &ToolchainEnv::for_test(Some(temp.path().join("dealer-home")), None),
        );
        let compiler = StaticCompilerBackend {
            metadata: HashMap::from([(
                project.root_file.clone(),
                ProjectMetadata {
                    project_type: "app".to_string(),
                    name: "RunSmoke".to_string(),
                    dependencies: Vec::new(),
                },
            )]),
        };

        let status =
            run_project(&project, &toolchain, BuildMode::Dev, &compiler).expect("run should pass");

        assert_eq!(status, 0);
    }

    #[test]
    fn run_project_rejects_package_root() {
        let temp = TempProject::new("run-package");
        fs::write(temp.path().join("package.x"), "package Lib\n")
            .expect("package root should be written");
        let project = validate_project_root(temp.path()).expect("package.x root should be valid");
        let workspace_root = crate::workspace_root();
        let toolchain = ToolchainSelection::discover(
            &workspace_root,
            &ToolchainEnv::for_test(Some(temp.path().join("dealer-home")), None),
        );
        let compiler = StaticCompilerBackend {
            metadata: HashMap::new(),
        };

        let error = run_project(&project, &toolchain, BuildMode::Dev, &compiler)
            .expect_err("package run should fail");

        assert!(error.to_string().contains("requires app.x"));
    }
}
