use crate::project::ProjectRoot;
use std::path::PathBuf;

pub(crate) struct BuildLayout {
    pub(crate) dealer_dir: PathBuf,
    pub(crate) rust_dir: PathBuf,
    pub(crate) xtazy_dir: PathBuf,
    pub(crate) product_dir: PathBuf,
}

impl BuildLayout {
    pub(crate) fn for_project(project: &ProjectRoot) -> Self {
        let dealer_dir = project
            .root_dir
            .join(crate::constants::dirs::PROJECT_DEALER_DIR);
        Self {
            rust_dir: dealer_dir.join(crate::constants::dirs::PROJECT_RUST_DIR),
            xtazy_dir: dealer_dir.join(crate::constants::dirs::PROJECT_XTAZY_DIR),
            product_dir: project
                .root_dir
                .join(crate::constants::dirs::PROJECT_PRODUCT_DIR),
            dealer_dir,
        }
    }
}
