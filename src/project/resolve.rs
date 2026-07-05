use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{DealerError, DealerResult};
use crate::project::dealer_block::parse_project_file;
use crate::project::dependency::{DependencySource, GitRev, ParsedDependency, VersionReq};
use crate::project::root::ProjectRoot;
use crate::state::DealerState;
use crate::support::net::fetch_url_string;

pub(crate) fn resolve_xtazy_version(
    project: &ProjectRoot,
    state: &DealerState,
) -> DealerResult<String> {
    let content = fs::read_to_string(&project.root_file)
        .map_err(|e| DealerError::io(&project.root_file, e))?;
    let decl = parse_project_file(&content)?;

    if let Some(pin) = decl.xtazy_pin {
        return Ok(pin);
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let last_check = state.last_xtazy_update_check().unwrap_or(0);
    let last_check_ok = now - last_check < 3600;
    let cached_ver = if last_check_ok {
        state.latest_xtazy_version()
    } else {
        None
    };
    if let Some(cached) = cached_ver {
        return Ok(cached);
    }

    let latest_url = crate::constants::web::XTAZY_LATEST_URL;
    match fetch_url_string(latest_url) {
        Ok(latest) => {
            let ver = latest.trim().to_string();
            state.set_latest_xtazy_version(&ver).ok();
            state.set_last_xtazy_update_check(now).ok();
            Ok(ver)
        }
        Err(err) => {
            if let Some(cached) = state.latest_xtazy_version() {
                Ok(cached)
            } else {
                Err(DealerError::Backend(format!(
                    "failed to fetch latest xtazy version and no cache available: {err}"
                )))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DependencyMergeState {
    Registry { reqs: Vec<VersionReq> },
    GitNumeric { url: String, reqs: Vec<VersionReq> },
    GitRef { url: String, ref_quoted: String },
    LocalPath { canonical_path: PathBuf },
}

fn merge_dependency(
    name: &str,
    new_dep: &ParsedDependency,
    existing: Option<&DependencyMergeState>,
    base_dir: &Path,
) -> DealerResult<DependencyMergeState> {
    match existing {
        None => match &new_dep.source {
            DependencySource::Registry { req } => Ok(DependencyMergeState::Registry {
                reqs: vec![req.clone()],
            }),
            DependencySource::Git { url, rev } => match rev {
                GitRev::Req(req) => Ok(DependencyMergeState::GitNumeric {
                    url: url.clone(),
                    reqs: vec![req.clone()],
                }),
                GitRev::Branch(branch) => Ok(DependencyMergeState::GitRef {
                    url: url.clone(),
                    ref_quoted: branch.clone(),
                }),
            },
            DependencySource::LocalPath { path } => {
                let canonical =
                    crate::package::local::resolve_local_dependency(base_dir, name, path)?;
                Ok(DependencyMergeState::LocalPath {
                    canonical_path: canonical,
                })
            }
        },
        Some(state) => match (state, &new_dep.source) {
            (DependencyMergeState::Registry { reqs }, DependencySource::Registry { req }) => {
                if reqs.contains(req) {
                    return Ok(DependencyMergeState::Registry { reqs: reqs.clone() });
                }
                let mut next_reqs = reqs.clone();
                next_reqs.push(req.clone());
                if VersionReq::merge_all(&next_reqs).is_none() {
                    return Err(DealerError::PackageResolution(format!(
                        "Conflict: incompatible version requirements for package '{}'",
                        name
                    )));
                }
                Ok(DependencyMergeState::Registry { reqs: next_reqs })
            }
            (
                DependencyMergeState::GitNumeric { url, reqs },
                DependencySource::Git {
                    url: new_url,
                    rev: GitRev::Req(req),
                },
            ) => {
                if url != new_url {
                    return Err(DealerError::PackageResolution(format!(
                        "Conflict: package '{}' has different Git URLs: '{}' and '{}'",
                        name, url, new_url
                    )));
                }
                if reqs.contains(req) {
                    return Ok(DependencyMergeState::GitNumeric {
                        url: url.clone(),
                        reqs: reqs.clone(),
                    });
                }
                let mut next_reqs = reqs.clone();
                next_reqs.push(req.clone());
                if VersionReq::merge_all(&next_reqs).is_none() {
                    return Err(DealerError::PackageResolution(format!(
                        "Conflict: incompatible Git version requirements for package '{}'",
                        name
                    )));
                }
                Ok(DependencyMergeState::GitNumeric {
                    url: url.clone(),
                    reqs: next_reqs,
                })
            }
            (
                DependencyMergeState::GitRef { url, ref_quoted },
                DependencySource::Git {
                    url: new_url,
                    rev: GitRev::Branch(new_ref),
                },
            ) => {
                if url != new_url || ref_quoted != new_ref {
                    return Err(DealerError::PackageResolution(format!(
                        "Conflict: package '{}' has conflicting Git branch/ref references",
                        name
                    )));
                }
                Ok(DependencyMergeState::GitRef {
                    url: url.clone(),
                    ref_quoted: ref_quoted.clone(),
                })
            }
            (
                DependencyMergeState::LocalPath { canonical_path },
                DependencySource::LocalPath { path },
            ) => {
                let canonical =
                    crate::package::local::resolve_local_dependency(base_dir, name, path)?;
                if canonical_path != &canonical {
                    return Err(DealerError::PackageResolution(format!(
                        "Conflict: package '{}' resolves to different local paths: '{}' and '{}'",
                        name,
                        canonical_path.display(),
                        canonical.display()
                    )));
                }
                Ok(DependencyMergeState::LocalPath {
                    canonical_path: canonical,
                })
            }
            _ => Err(DealerError::PackageResolution(format!(
                "Conflict: package '{}' is declared with different source kinds",
                name
            ))),
        },
    }
}

fn select_highest_satisfying_registry_version(
    name: &str,
    reqs: &[VersionReq],
) -> DealerResult<String> {
    let entries = crate::package::registry::fetch_registry_versions(name)?;

    let mut satisfying_versions = Vec::new();
    for entry in entries {
        let is_ok = reqs.iter().all(|req| req.satisfies(&entry.version));
        if is_ok {
            satisfying_versions.push(entry.version);
        }
    }

    if satisfying_versions.is_empty() {
        return Err(DealerError::PackageResolution(format!(
            "No registry version found for package '{}' satisfying requirements: {:?}",
            name, reqs
        )));
    }

    satisfying_versions.sort_by(|a, b| {
        if crate::package::semver::is_newer_semver(a, b) {
            std::cmp::Ordering::Greater
        } else if crate::package::semver::is_newer_semver(b, a) {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Equal
        }
    });

    Ok(satisfying_versions.pop().unwrap())
}

fn select_highest_satisfying_git_version(
    name: &str,
    url: &str,
    reqs: &[VersionReq],
) -> DealerResult<String> {
    let tags = crate::support::git::list_remote_tags(url).map_err(|e| {
        DealerError::Package(format!(
            "Failed to list remote tags for {name} from {url}: {e}"
        ))
    })?;

    let mut satisfying_versions = Vec::new();
    for tag in tags {
        if let Some(stripped) = tag.strip_prefix("refs/tags/") {
            let is_ok = reqs.iter().all(|req| req.satisfies(stripped));
            if is_ok {
                satisfying_versions.push(stripped.to_string());
            }
        }
    }

    if satisfying_versions.is_empty() {
        return Err(DealerError::PackageResolution(format!(
            "No Git tag found for package '{}' satisfying requirements: {:?}",
            name, reqs
        )));
    }

    satisfying_versions.sort_by(|a, b| {
        if crate::package::semver::is_newer_semver(a, b) {
            std::cmp::Ordering::Greater
        } else if crate::package::semver::is_newer_semver(b, a) {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Equal
        }
    });

    Ok(satisfying_versions.pop().unwrap())
}

struct ResolveContext<'a> {
    state: &'a DealerState,
    merge_states: &'a mut HashMap<String, DependencyMergeState>,
    current_resolved: &'a mut HashMap<String, PathBuf>,
    path_stack: &'a mut Vec<String>,
    changed: &'a mut bool,
    parsed_cache: &'a mut HashMap<PathBuf, crate::project::dealer_block::ProjectDeclaration>,
}

pub(crate) fn resolve_dependencies(
    project: &ProjectRoot,
    state: &DealerState,
) -> DealerResult<HashMap<String, PathBuf>> {
    let mut merge_states = HashMap::new();
    let mut resolved_paths = HashMap::new();
    let mut parsed_cache = HashMap::new();

    let content = fs::read_to_string(&project.root_file)
        .map_err(|e| DealerError::io(&project.root_file, e))?;
    let decl = parse_project_file(&content)?;

    loop {
        let mut changed = false;
        let mut current_resolved = HashMap::new();
        let mut path_stack = Vec::new();

        {
            let mut ctx = ResolveContext {
                state,
                merge_states: &mut merge_states,
                current_resolved: &mut current_resolved,
                path_stack: &mut path_stack,
                changed: &mut changed,
                parsed_cache: &mut parsed_cache,
            };

            resolve_recursive_fixed_point(&decl.dependencies, &project.root_dir, &mut ctx)?;
        }

        if !changed && current_resolved == resolved_paths {
            resolved_paths = current_resolved;
            break;
        }
        resolved_paths = current_resolved;
    }

    Ok(resolved_paths)
}

fn resolve_recursive_fixed_point(
    dependencies: &[ParsedDependency],
    base_dir: &Path,
    ctx: &mut ResolveContext,
) -> DealerResult<()> {
    for dep in dependencies {
        if ctx.path_stack.contains(&dep.name) {
            return Err(DealerError::PackageResolution(format!(
                "Circular dependency detected: {:?}",
                ctx.path_stack
            )));
        }

        let existing = ctx.merge_states.get(&dep.name).cloned();
        let new_state = merge_dependency(&dep.name, dep, existing.as_ref(), base_dir)?;
        if existing.is_none() || existing.as_ref().unwrap() != &new_state {
            ctx.merge_states.insert(dep.name.clone(), new_state.clone());
            *ctx.changed = true;
        }

        let dep_path = match &new_state {
            DependencyMergeState::LocalPath { canonical_path } => canonical_path.clone(),
            DependencyMergeState::GitRef { url, ref_quoted } => {
                crate::package::git::resolve_git_dependency(
                    ctx.state,
                    &dep.name,
                    url,
                    &crate::support::git::ResolvedGitRev::Branch(ref_quoted.clone()),
                )?
            }
            DependencyMergeState::Registry { reqs } => {
                let selected_version = select_highest_satisfying_registry_version(&dep.name, reqs)?;
                crate::package::registry::resolve_registry_dependency(
                    ctx.state,
                    &dep.name,
                    &selected_version,
                )?
            }
            DependencyMergeState::GitNumeric { url, reqs } => {
                let selected_tag = select_highest_satisfying_git_version(&dep.name, url, reqs)?;
                crate::package::git::resolve_git_dependency(
                    ctx.state,
                    &dep.name,
                    url,
                    &crate::support::git::ResolvedGitRev::Tag(selected_tag),
                )?
            }
        };

        let already_resolved = ctx.current_resolved.get(&dep.name).cloned();
        if already_resolved.is_none() || already_resolved.as_ref().unwrap() != &dep_path {
            let package_x = dep_path.join("package.x");
            if !package_x.is_file() {
                return Err(DealerError::Package(format!(
                    "Dependency '{}' at '{}' is invalid: missing package.x",
                    dep.name,
                    dep_path.display()
                )));
            }

            let dep_decl = if let Some(cached) = ctx.parsed_cache.get(&package_x) {
                cached.clone()
            } else {
                let dep_content =
                    fs::read_to_string(&package_x).map_err(|e| DealerError::io(&package_x, e))?;
                let parsed = parse_project_file(&dep_content)?;
                ctx.parsed_cache.insert(package_x.clone(), parsed.clone());
                parsed
            };

            if dep_decl.is_app {
                return Err(DealerError::PackageResolution(format!(
                    "Dependency '{}' resolved to '{}', which is an app, not a package",
                    dep.name,
                    dep_path.display()
                )));
            }

            if dep_decl.name != dep.name {
                return Err(DealerError::PackageResolution(format!(
                    "Package name mismatch: declared dependency is '{}', but resolved package claims name '{}'",
                    dep.name, dep_decl.name
                )));
            }

            ctx.current_resolved
                .insert(dep.name.clone(), dep_path.clone());

            ctx.path_stack.push(dep.name.clone());
            resolve_recursive_fixed_point(&dep_decl.dependencies, &dep_path, ctx)?;
            ctx.path_stack.pop();
        }
    }
    Ok(())
}
