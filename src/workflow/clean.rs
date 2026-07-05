use std::fs;
use std::io;
use std::path::Path;

use crate::error::{DealerError, DealerResult};
use crate::project::ProjectRoot;
use crate::workflow::layout::BuildLayout;

pub(crate) fn run_clean_or_exit(project: &ProjectRoot) {
    exit_on_error(run_clean(project));
    println!(
        "Removed dealer build state for {}",
        project.root_dir.display()
    );
}

pub(crate) fn run_clean(project: &ProjectRoot) -> DealerResult<()> {
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

fn exit_on_error(result: DealerResult<()>) {
    if let Err(error) = result {
        eprintln!("dealer: {error}");
        std::process::exit(1);
    }
}
