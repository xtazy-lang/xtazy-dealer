use std::fs;

use crate::error::{DealerError, DealerResult};
use crate::package::edit::{find_dealer_block_range, get_token_spans, split_code_and_comment};
use crate::package::semver::is_newer_semver;
use crate::project::ProjectRoot;
use crate::support::net::fetch_url_string;

pub(crate) fn run_outdated_packages(project: &ProjectRoot) -> DealerResult<()> {
    let content = fs::read_to_string(&project.root_file)
        .map_err(|e| DealerError::io(&project.root_file, e))?;
    let lines: Vec<&str> = content.lines().collect();

    let Some((_h_idx, start_idx, end_idx)) = find_dealer_block_range(&lines)
        .map_err(|e| DealerError::Project(format!("failed to parse project structure: {e}")))?
    else {
        return Ok(());
    };

    for &line in lines.iter().take(end_idx + 1).skip(start_idx) {
        if line.trim().is_empty() {
            continue;
        }
        let (code, _) = split_code_and_comment(line);
        let spans = get_token_spans(code);
        if spans.is_empty() {
            continue;
        }
        let dep_name = &spans[0].text;
        if spans.len() == 2 {
            let req_str = spans[1].text.trim_matches('"');
            if let Ok(req) = crate::project::dependency::VersionReq::parse(req_str) {
                match req {
                    crate::project::dependency::VersionReq::MajorWildcard { .. }
                    | crate::project::dependency::VersionReq::MinorWildcard { .. } => {
                        // Wildcard requirements are skipped by dealer outdated.
                        continue;
                    }
                    crate::project::dependency::VersionReq::Exact { .. } => {
                        let versions_url = crate::constants::web::package_versions_url(dep_name);
                        if let Ok(content) = fetch_url_string(&versions_url) {
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
                                if is_newer_semver(a, b) {
                                    std::cmp::Ordering::Greater
                                } else if is_newer_semver(b, a) {
                                    std::cmp::Ordering::Less
                                } else {
                                    std::cmp::Ordering::Equal
                                }
                            });
                            if let Some(highest) =
                                satisfying.pop().filter(|h| is_newer_semver(h, req_str))
                            {
                                println!(
                                    "{} (registry) is outdated: {req_str} -> {highest}",
                                    dep_name
                                );
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
                        // Wildcard requirements are skipped by dealer outdated.
                        continue;
                    }
                    crate::project::dependency::VersionReq::Exact { .. } => {
                        let res = crate::support::git::list_remote_tags(url);
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
                                if is_newer_semver(a, b) {
                                    std::cmp::Ordering::Greater
                                } else if is_newer_semver(b, a) {
                                    std::cmp::Ordering::Less
                                } else {
                                    std::cmp::Ordering::Equal
                                }
                            });
                            if let Some(highest) =
                                satisfying.pop().filter(|h| is_newer_semver(h, req_str))
                            {
                                println!("{} (git) is outdated: {req_str} -> {highest}", dep_name);
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
