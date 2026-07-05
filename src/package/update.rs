use std::fs;

use crate::error::{DealerError, DealerResult};
use crate::package::edit::{find_dealer_block_range, get_token_spans, split_code_and_comment};
use crate::project::{ProjectRoot, resolve_dependencies};
use crate::state::DealerState;
use crate::support::net::fetch_url_string;

pub(crate) fn run_update_packages_internal<F, G, H>(
    project: &ProjectRoot,
    package: Option<&str>,
    state: &DealerState,
    fetch_latest: F,
    git_ls_remote: G,
    resolve_deps: H,
) -> DealerResult<()>
where
    F: Fn(&str) -> DealerResult<String>,
    G: Fn(&str) -> DealerResult<Vec<String>>,
    H: Fn(&ProjectRoot, &DealerState) -> DealerResult<()>,
{
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
    let Some((_h_idx, start_idx, end_idx)) = find_dealer_block_range(&lines_ref)
        .map_err(|e| DealerError::Project(format!("failed to parse project structure: {e}")))?
    else {
        return Ok(());
    };

    for line in lines.iter_mut().take(end_idx + 1).skip(start_idx) {
        if line.trim().is_empty() {
            continue;
        }

        let (code, comment) = split_code_and_comment(line);
        let spans = get_token_spans(code);
        if spans.is_empty() {
            continue;
        }

        let dep_name = &spans[0].text;
        if package.is_none() || package == Some(dep_name.as_str()) {
            if spans.len() == 2 {
                let req_str = spans[1].text.trim_matches('"');
                if let Ok(req) = crate::project::dependency::VersionReq::parse(req_str) {
                    match req {
                        crate::project::dependency::VersionReq::MajorWildcard { .. }
                        | crate::project::dependency::VersionReq::MinorWildcard { .. } => {
                            // Wildcard requirements are left unchanged by dealer update and skipped.
                            continue;
                        }
                        crate::project::dependency::VersionReq::Exact { .. } => {
                            let versions_url =
                                crate::constants::web::package_versions_url(dep_name);
                            if let Ok(content) = fetch_latest(&versions_url) {
                                let mut available = Vec::new();
                                for line in content.lines() {
                                    let line = line.trim();
                                    if line.is_empty() {
                                        continue;
                                    }
                                    let parts: Vec<&str> = line.split_whitespace().collect();
                                    if !parts.is_empty() {
                                        available.push(parts[0].to_string());
                                    }
                                }
                                let mut satisfying = Vec::new();
                                for v in available {
                                    if req.satisfies(&v) {
                                        satisfying.push(v);
                                    }
                                }
                                satisfying.sort_by(|a, b| {
                                    if crate::package::semver::is_newer_semver(a, b) {
                                        std::cmp::Ordering::Greater
                                    } else if crate::package::semver::is_newer_semver(b, a) {
                                        std::cmp::Ordering::Less
                                    } else {
                                        std::cmp::Ordering::Equal
                                    }
                                });
                                if let Some(highest) = satisfying
                                    .pop()
                                    .filter(|h| crate::package::semver::is_newer_semver(h, req_str))
                                {
                                    println!(
                                        "Updating registry package '{}': {req_str} -> {highest}",
                                        dep_name
                                    );

                                    let mut new_val_formatted = highest;
                                    if spans[1].text.starts_with('"')
                                        && spans[1].text.ends_with('"')
                                    {
                                        new_val_formatted = format!("\"{}\"", new_val_formatted);
                                    }
                                    let mut new_code = code[..spans[1].start].to_string();
                                    new_code.push_str(&new_val_formatted);
                                    new_code.push_str(&code[spans[1].end..]);

                                    if let Some(comm) = comment {
                                        new_code.push_str(comm);
                                    }
                                    *line = new_code;
                                }
                            }
                        }
                    }
                }
            } else if spans.len() == 3 {
                // If the third token is quoted, it is an exact Git ref and must be skipped
                if spans[2].text.starts_with('"') && spans[2].text.ends_with('"') {
                    continue;
                }
                let req_str = spans[2].text.trim_matches('"');
                let url = spans[1].text.trim_matches('"');
                if let Ok(req) = crate::project::dependency::VersionReq::parse(req_str) {
                    match req {
                        crate::project::dependency::VersionReq::MajorWildcard { .. }
                        | crate::project::dependency::VersionReq::MinorWildcard { .. } => {
                            // Wildcard requirements are skipped.
                            continue;
                        }
                        crate::project::dependency::VersionReq::Exact { .. } => {
                            let res = git_ls_remote(url);
                            if let Ok(tags) = res {
                                let mut satisfying = Vec::new();
                                for ref_name in tags {
                                    if let Some(tag_name) = ref_name
                                        .strip_prefix("refs/tags/")
                                        .filter(|t| req.satisfies(t))
                                    {
                                        satisfying.push(tag_name.to_string());
                                    }
                                }
                                satisfying.sort_by(|a, b| {
                                    if crate::package::semver::is_newer_semver(a, b) {
                                        std::cmp::Ordering::Greater
                                    } else if crate::package::semver::is_newer_semver(b, a) {
                                        std::cmp::Ordering::Less
                                    } else {
                                        std::cmp::Ordering::Equal
                                    }
                                });
                                if let Some(highest) = satisfying
                                    .pop()
                                    .filter(|h| crate::package::semver::is_newer_semver(h, req_str))
                                {
                                    println!(
                                        "Updating git package '{}': {req_str} -> {highest}",
                                        dep_name
                                    );

                                    let clean_highest =
                                        highest.strip_prefix('v').unwrap_or(&highest);
                                    let new_val_formatted = clean_highest.to_string();
                                    let mut new_code = code[..spans[2].start].to_string();
                                    new_code.push_str(&new_val_formatted);
                                    new_code.push_str(&code[spans[2].end..]);

                                    if let Some(comm) = comment {
                                        new_code.push_str(comm);
                                    }
                                    *line = new_code;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fs::write(&project.root_file, lines.join("\n") + "\n")
        .map_err(|e| DealerError::io(&project.root_file, e))?;
    resolve_deps(project, state)?;
    Ok(())
}

pub(crate) fn run_update_packages(
    project: &ProjectRoot,
    package: Option<&str>,
    state: &DealerState,
) -> DealerResult<()> {
    run_update_packages_internal(
        project,
        package,
        state,
        |url| fetch_url_string(url).map_err(DealerError::Package),
        |url| crate::support::git::list_remote_tags(url).map_err(DealerError::Package),
        |proj, st| {
            resolve_dependencies(proj, st)?;
            Ok(())
        },
    )
}
