use crate::error::{DealerError, DealerResult};
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn resolve_local_dependency(
    base_dir: &Path,
    name: &str,
    path: &str,
) -> DealerResult<PathBuf> {
    let p = Path::new(path);
    let abs = if p.is_absolute() {
        p.to_path_buf()
    } else {
        base_dir.join(p)
    };
    fs::canonicalize(&abs).map_err(|e| {
        DealerError::Package(format!(
            "Failed to locate dependency '{}' at '{}': {e}",
            name,
            abs.display()
        ))
    })
}
