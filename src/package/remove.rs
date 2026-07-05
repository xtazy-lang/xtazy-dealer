use std::fs;

use crate::error::{DealerError, DealerResult};
use crate::package::edit::{find_dealer_block_range, get_token_spans};
use crate::project::{ProjectRoot, resolve_dependencies};
use crate::state::DealerState;

pub(crate) fn run_remove_package(
    project: &ProjectRoot,
    package: &str,
    state: &DealerState,
) -> DealerResult<()> {
    // Acquire the exclusive project lock
    let dealer_dir = project.root_dir.join(".dealer");
    fs::create_dir_all(&dealer_dir).map_err(|e| DealerError::io(&dealer_dir, e))?;
    let lock_file = dealer_dir.join("project.lock");
    let lock_file_handle =
        fs::File::create(&lock_file).map_err(|e| DealerError::io(&lock_file, e))?;
    let mut rw_lock = fd_lock::RwLock::new(lock_file_handle);
    let _project_lock_guard = rw_lock.write().map_err(|e| {
        DealerError::Backend(format!(
            "failed to acquire project lock at {}: {}",
            lock_file.display(),
            e
        ))
    })?;

    let content = fs::read_to_string(&project.root_file)
        .map_err(|e| DealerError::io(&project.root_file, e))?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    let lines_ref: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
    if let Some((_h_idx, start_idx, end_idx)) = find_dealer_block_range(&lines_ref)
        .map_err(|e| DealerError::Project(format!("failed to parse project structure: {e}")))?
    {
        let mut remove_idx = None;
        for (i, line) in lines.iter().enumerate().take(end_idx + 1).skip(start_idx) {
            if line.trim().is_empty() {
                continue;
            }
            let spans = get_token_spans(line);
            if !spans.is_empty() && spans[0].text == package {
                remove_idx = Some(i);
                break;
            }
        }

        if let Some(idx) = remove_idx {
            lines.remove(idx);
        } else {
            return Err(DealerError::Package(format!(
                "package '{package}' is not declared in this project"
            )));
        }
    } else {
        return Err(DealerError::Package(format!(
            "package '{package}' is not declared in this project"
        )));
    }

    fs::write(&project.root_file, lines.join("\n") + "\n")
        .map_err(|e| DealerError::io(&project.root_file, e))?;
    resolve_dependencies(project, state)?;
    Ok(())
}

pub(crate) fn run_cache_clean(state: &DealerState) -> DealerResult<()> {
    let cache_dir = state.cache_dir();
    if !cache_dir.exists() {
        return Ok(());
    }

    let lock_file = state.dealer_home.join("cache_clean.lock");
    fs::create_dir_all(&state.dealer_home).map_err(|e| DealerError::io(&state.dealer_home, e))?;
    let f = fs::File::create(&lock_file).map_err(|e| DealerError::io(&lock_file, e))?;
    let mut lock = fd_lock::RwLock::new(f);
    let _write_lock = lock
        .write()
        .map_err(|e| DealerError::Package(format!("failed to lock cache: {e}")))?;

    let entries = fs::read_dir(&cache_dir).map_err(|e| DealerError::io(&cache_dir, e))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            fs::remove_dir_all(&path).map_err(|e| DealerError::io(&path, e))?;
        } else {
            fs::remove_file(&path).map_err(|e| DealerError::io(&path, e))?;
        }
    }
    Ok(())
}
