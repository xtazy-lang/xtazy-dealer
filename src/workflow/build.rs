use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::cli::BuildMode;
use crate::compiler_contract::CompilerBackend;
use crate::error::{DealerError, DealerResult};
use crate::names::sanitize_package_name;
use crate::project::{ProjectRoot, resolve_dependencies};
use crate::state::DealerState;
use crate::toolchain::ToolchainSelection;
use crate::workflow::layout::BuildLayout;

pub(crate) fn run_build_or_exit(
    project: &ProjectRoot,
    toolchain: &ToolchainSelection,
    mode: BuildMode,
) {
    let compiler = toolchain.compiler_backend();
    let state = DealerState::from_home(toolchain.dealer_home.clone());
    match run_build(project, toolchain, mode, &compiler, &state) {
        Ok(product_path) => println!("Product written to {}", product_path.display()),
        Err(error) => {
            eprintln!("dealer: {error}");
            std::process::exit(1);
        }
    }
}

pub(crate) fn run_build(
    project: &ProjectRoot,
    toolchain: &ToolchainSelection,
    mode: BuildMode,
    compiler: &dyn CompilerBackend,
    state: &DealerState,
) -> DealerResult<PathBuf> {
    let layout = BuildLayout::for_project(project);
    prepare_build_folders(&layout).map_err(|error| DealerError::io(&layout.dealer_dir, error))?;

    let deps = resolve_dependencies(project, state).map_err(|error| {
        DealerError::PackageResolution(format!("failed to resolve dependencies: {error}"))
    })?;

    compiler.build(project, &deps, &layout.rust_dir, &toolchain.rusttime_path)?;

    toolchain.backend.build(&layout.rust_dir, mode)?;

    let is_executable = is_app_root(project);
    let product_path = copy_product_artifact(
        project,
        &layout.rust_dir,
        &layout.product_dir,
        is_executable,
        mode,
    )?;

    write_last_build_state(project, toolchain, &layout, &product_path).map_err(|error| {
        DealerError::io(
            layout
                .xtazy_dir
                .join(crate::constants::files::LAST_BUILD_FILE),
            error,
        )
    })?;
    Ok(product_path)
}

pub(crate) fn is_app_root(project: &ProjectRoot) -> bool {
    project.root_file.file_name().and_then(|n| n.to_str()) == Some("app.x")
}

fn prepare_build_folders(layout: &BuildLayout) -> io::Result<()> {
    fs::create_dir_all(&layout.dealer_dir)?;
    fs::create_dir_all(&layout.rust_dir)?;
    fs::create_dir_all(&layout.xtazy_dir)?;
    fs::create_dir_all(&layout.product_dir)?;
    Ok(())
}

fn copy_product_artifact(
    project: &ProjectRoot,
    rust_dir: &Path,
    product_dir: &Path,
    is_executable: bool,
    mode: BuildMode,
) -> DealerResult<PathBuf> {
    let package_name = sanitize_package_name(&project.project_name);
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

fn write_last_build_state(
    project: &ProjectRoot,
    toolchain: &ToolchainSelection,
    layout: &BuildLayout,
    product_path: &Path,
) -> io::Result<()> {
    let state = format!(
        "{}={}\n{}={}\n{}={}\n{}={}\n{}={}\n{}={}\n{}={}\n{}={}\n{}={}\n{}={}\n{}={}\n{}={}\n{}={}\n{}={}\n{}={}\n",
        crate::constants::metadata::KEY_PROJECT_ROOT,
        project.root_dir.display(),
        crate::constants::metadata::KEY_ROOT_FILE,
        project.root_file.display(),
        crate::constants::metadata::KEY_RUST_DIR,
        layout.rust_dir.display(),
        crate::constants::metadata::KEY_PRODUCT_PATH,
        product_path.display(),
        crate::constants::metadata::KEY_DEALER_HOME,
        toolchain.dealer_home.display(),
        crate::constants::metadata::KEY_TOOLCHAIN_VERSION,
        toolchain.version,
        crate::constants::metadata::KEY_TOOLCHAIN_DIR,
        toolchain.toolchain_dir.display(),
        crate::constants::metadata::KEY_COMPILER_SOURCE,
        toolchain.compiler_source.as_metadata_value(),
        crate::constants::metadata::KEY_COMPILER_PATH,
        toolchain.compiler_path.display(),
        crate::constants::metadata::KEY_RUST_BACKEND_ID,
        toolchain.rust_backend_id,
        crate::constants::metadata::KEY_RUST_BACKEND_DIR,
        toolchain.rust_backend_dir.display(),
        crate::constants::metadata::KEY_BACKEND_SOURCE,
        toolchain.backend_source().as_metadata_value(),
        crate::constants::metadata::KEY_CARGO_PATH,
        toolchain.backend.cargo_path.display(),
        crate::constants::metadata::KEY_RUSTTIME_SOURCE,
        toolchain.rusttime_source.as_metadata_value(),
        crate::constants::metadata::KEY_RUSTTIME_PATH,
        toolchain.rusttime_path.display(),
    );
    fs::write(
        layout
            .xtazy_dir
            .join(crate::constants::files::LAST_BUILD_FILE),
        state,
    )
}
