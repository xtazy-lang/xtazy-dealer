use crate::error::{DealerError, DealerResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedDependency {
    pub(crate) name: String,
    pub(crate) source: DependencySource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DependencySource {
    Registry { req: VersionReq },
    Git { url: String, rev: GitRev },
    LocalPath { path: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum GitRev {
    Req(VersionReq),
    Branch(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct VersionBounds {
    pub(crate) min: (u64, u64, u64),
    pub(crate) max: (u64, u64, u64),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum VersionReq {
    Exact { major: u64, minor: u64, patch: u64 },
    MajorWildcard { major: u64 },
    MinorWildcard { major: u64, minor: u64 },
}

impl VersionReq {
    pub(crate) fn parse(s: &str) -> DealerResult<Self> {
        let clean_s = s.strip_prefix('v').unwrap_or(s);
        let parts: Vec<&str> = clean_s.split('.').collect();
        if parts.len() != 3 {
            return Err(DealerError::Project(format!(
                "invalid version requirement: {}",
                s
            )));
        }
        let major = parts[0]
            .parse::<u64>()
            .map_err(|_| DealerError::Project(format!("invalid major version in {}", s)))?;
        if parts[1].eq_ignore_ascii_case("x") {
            if !parts[2].eq_ignore_ascii_case("x") {
                return Err(DealerError::Project(format!(
                    "invalid wildcard version requirement: {}",
                    s
                )));
            }
            Ok(VersionReq::MajorWildcard { major })
        } else {
            let minor = parts[1]
                .parse::<u64>()
                .map_err(|_| DealerError::Project(format!("invalid minor version in {}", s)))?;
            if parts[2].eq_ignore_ascii_case("x") {
                Ok(VersionReq::MinorWildcard { major, minor })
            } else {
                let patch = parts[2]
                    .parse::<u64>()
                    .map_err(|_| DealerError::Project(format!("invalid patch version in {}", s)))?;
                Ok(VersionReq::Exact {
                    major,
                    minor,
                    patch,
                })
            }
        }
    }

    pub(crate) fn min_bound(&self) -> (u64, u64, u64) {
        match self {
            VersionReq::Exact {
                major,
                minor,
                patch,
            } => (*major, *minor, *patch),
            VersionReq::MajorWildcard { major } => (*major, 0, 0),
            VersionReq::MinorWildcard { major, minor } => (*major, *minor, 0),
        }
    }

    pub(crate) fn max_bound(&self) -> (u64, u64, u64) {
        match self {
            VersionReq::Exact { major, .. } => (*major + 1, 0, 0),
            VersionReq::MajorWildcard { major } => (*major + 1, 0, 0),
            VersionReq::MinorWildcard { major, minor } => (*major, *minor + 1, 0),
        }
    }

    pub(crate) fn merge_all(reqs: &[VersionReq]) -> Option<VersionBounds> {
        if reqs.is_empty() {
            return None;
        }
        let mut merged_min = reqs[0].min_bound();
        let mut merged_max = reqs[0].max_bound();
        for req in &reqs[1..] {
            let min = req.min_bound();
            let max = req.max_bound();
            merged_min = std::cmp::max(merged_min, min);
            merged_max = std::cmp::min(merged_max, max);
        }
        if merged_min < merged_max {
            Some(VersionBounds {
                min: merged_min,
                max: merged_max,
            })
        } else {
            None
        }
    }

    pub(crate) fn satisfies(&self, version_str: &str) -> bool {
        let Some((major, minor, patch, _)) = crate::package::semver::parse_semver(version_str)
        else {
            return false;
        };
        let min = self.min_bound();
        let max = self.max_bound();
        let val = (major, minor, patch);
        val >= min && val < max
    }
}

impl std::fmt::Display for VersionReq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VersionReq::Exact {
                major,
                minor,
                patch,
            } => write!(f, "{}.{}.{}", major, minor, patch),
            VersionReq::MajorWildcard { major } => write!(f, "{}.x.x", major),
            VersionReq::MinorWildcard { major, minor } => write!(f, "{}.{}.x", major, minor),
        }
    }
}

pub(crate) fn tokenize_line(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    for ch in line.chars() {
        if ch == '"' {
            in_quotes = !in_quotes;
            current.push(ch);
        } else if ch.is_whitespace() && !in_quotes {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
        } else {
            current.push(ch);
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

pub(crate) fn parse_dependency_line(line: &str) -> Option<ParsedDependency> {
    let tokens = tokenize_line(line);
    if tokens.is_empty() {
        return None;
    }
    let name = tokens[0].clone();
    if tokens.len() == 2 {
        let arg = tokens[1].clone();
        if arg.starts_with('"') && arg.ends_with('"') {
            let path = arg[1..arg.len() - 1].to_string();
            Some(ParsedDependency {
                name,
                source: DependencySource::LocalPath { path },
            })
        } else {
            let req = VersionReq::parse(&arg).ok()?;
            Some(ParsedDependency {
                name,
                source: DependencySource::Registry { req },
            })
        }
    } else if tokens.len() == 3 {
        let url_quoted = tokens[1].clone();
        let ref_quoted = tokens[2].clone();
        if url_quoted.starts_with('"') && url_quoted.ends_with('"') {
            let url = url_quoted[1..url_quoted.len() - 1].to_string();
            if ref_quoted.starts_with('"') && ref_quoted.ends_with('"') {
                let branch = ref_quoted[1..ref_quoted.len() - 1].to_string();
                Some(ParsedDependency {
                    name,
                    source: DependencySource::Git {
                        url,
                        rev: GitRev::Branch(branch),
                    },
                })
            } else {
                let req = VersionReq::parse(&ref_quoted).ok()?;
                Some(ParsedDependency {
                    name,
                    source: DependencySource::Git {
                        url,
                        rev: GitRev::Req(req),
                    },
                })
            }
        } else {
            None
        }
    } else {
        None
    }
}
