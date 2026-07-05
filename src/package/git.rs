use std::fs;
use std::path::PathBuf;

use crate::error::{DealerError, DealerResult};
use crate::state::DealerState;
use crate::support::git::ResolvedGitRev;

pub(crate) fn resolve_git_dependency(
    state: &DealerState,
    name: &str,
    url: &str,
    rev: &ResolvedGitRev,
) -> DealerResult<PathBuf> {
    let rev_str = match rev {
        ResolvedGitRev::Tag(t) => t.clone(),
        ResolvedGitRev::Branch(b) => b.clone(),
    };

    let pkg_dir = state
        .cache_dir()
        .join(crate::constants::dirs::GIT_DIR)
        .join(name)
        .join(&rev_str);
    let source_dir = pkg_dir.join(crate::constants::dirs::SOURCE_DIR);
    if source_dir.is_dir() {
        return Ok(source_dir);
    }

    // Lock cache write
    let lock_file = pkg_dir.join("lock");
    fs::create_dir_all(&pkg_dir).map_err(|e| DealerError::io(&pkg_dir, e))?;
    let f = fs::File::create(&lock_file).map_err(|e| DealerError::io(&lock_file, e))?;
    let mut lock = fd_lock::RwLock::new(f);
    let _write_lock = lock
        .write()
        .map_err(|e| DealerError::Package(format!("failed to lock cache: {e}")))?;

    if source_dir.is_dir() {
        return Ok(source_dir);
    }

    let temp_source_dir = pkg_dir.join("temp_source");
    if temp_source_dir.exists() {
        fs::remove_dir_all(&temp_source_dir).ok();
    }

    // Fetch and checkout using gix
    crate::support::git::materialize_git_source(url, rev, &temp_source_dir)
        .map_err(|e| DealerError::Package(format!("git checkout failed: {e}")))?;

    // Rename to source_dir
    fs::rename(&temp_source_dir, &source_dir).map_err(|e| DealerError::io(&source_dir, e))?;
    Ok(source_dir)
}
