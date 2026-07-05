use std::fs;
use std::path::Path;

use crate::error::{DealerError, DealerResult};
use crate::package::edit::{
    count_leading_spaces_or_tabs, find_dealer_block_range, get_token_spans,
};
use crate::project::{ProjectRoot, resolve_dependencies};
use crate::state::DealerState;
use crate::support::net::fetch_url_string;

pub(crate) fn run_install_package(
    project: &ProjectRoot,
    package_spec: &str,
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

    let (name, dep_line) = if package_spec.contains("://") || package_spec.starts_with("git@") {
        let url = package_spec;
        let name_with_git = url
            .split(['/', ':'])
            .next_back()
            .ok_or_else(|| DealerError::Package(format!("invalid git url: {url}")))?;
        let name = name_with_git
            .strip_suffix(".git")
            .unwrap_or(name_with_git)
            .to_string();
        (name.clone(), format!("\t\t{} \"{}\" \"main\"", name, url))
    } else if package_spec.starts_with("../")
        || package_spec.starts_with("./")
        || package_spec.starts_with('/')
        || (package_spec.contains('/') && !package_spec.contains("://"))
    {
        let path = Path::new(package_spec);
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| DealerError::Package(format!("invalid local path: {package_spec}")))?
            .to_string();
        (name.clone(), format!("\t\t{} \"{}\"", name, package_spec))
    } else {
        let url = crate::constants::web::package_latest_url(package_spec);
        let meta = fetch_url_string(&url)
            .map_err(|e| DealerError::Package(format!("failed to fetch package metadata: {e}")))?;
        let version = meta
            .split_whitespace()
            .next()
            .ok_or_else(|| {
                DealerError::Package(format!("invalid registry metadata for {package_spec}"))
            })?
            .to_string();
        (
            package_spec.to_string(),
            format!("\t\t{package_spec} {version}"),
        )
    };

    let content = fs::read_to_string(&project.root_file)
        .map_err(|e| DealerError::io(&project.root_file, e))?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    let lines_ref: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
    if let Some((_h_idx, start_idx, end_idx)) = find_dealer_block_range(&lines_ref)
        .map_err(|e| DealerError::Project(format!("failed to parse project file structure: {e}")))?
    {
        let mut exists_idx = None;
        for (i, line) in lines.iter().enumerate().take(end_idx + 1).skip(start_idx) {
            if line.trim().is_empty() {
                continue;
            }
            let spans = get_token_spans(line);
            if !spans.is_empty() && spans[0].text == name {
                exists_idx = Some(i);
                break;
            }
        }

        if let Some(idx) = exists_idx {
            lines[idx] = dep_line;
        } else {
            let mut indent_str = "\t\t".to_string();
            if end_idx >= start_idx {
                let last_dep_line = &lines[end_idx];
                let leading_count = count_leading_spaces_or_tabs(last_dep_line);
                indent_str = last_dep_line[..leading_count].to_string();
            }
            lines.insert(end_idx + 1, format!("{}{}", indent_str, dep_line.trim()));
        }
    } else {
        let mut first_non_empty = 0;
        for (i, line) in lines.iter().enumerate() {
            if !line.trim().is_empty() {
                first_non_empty = i;
                break;
            }
        }
        lines.insert(first_non_empty + 1, "\tdealer".to_string());
        lines.insert(first_non_empty + 2, format!("\t\t{}", dep_line.trim()));
    }

    fs::write(&project.root_file, lines.join("\n") + "\n")
        .map_err(|e| DealerError::io(&project.root_file, e))?;
    resolve_dependencies(project, state)?;
    println!("Installed package '{name}' successfully.");
    Ok(())
}
