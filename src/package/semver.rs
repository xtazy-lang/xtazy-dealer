pub(crate) fn parse_semver(s: &str) -> Option<(u64, u64, u64, String)> {
    let s = s.strip_prefix('v').unwrap_or(s);
    let (num_part, prerelease) = match s.split_once('-') {
        Some((n, p)) => (n, p.to_string()),
        None => (s, String::new()),
    };
    let parts: Vec<&str> = num_part.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let major = parts[0].parse::<u64>().ok()?;
    let minor = parts[1].parse::<u64>().ok()?;
    let patch = parts[2].parse::<u64>().ok()?;
    Some((major, minor, patch, prerelease))
}

pub(crate) fn is_newer_semver(candidate: &str, current: &str) -> bool {
    let Some(cand_val) = parse_semver(candidate) else {
        return false;
    };
    let Some(curr_val) = parse_semver(current) else {
        return false;
    };

    let cand_num = (cand_val.0, cand_val.1, cand_val.2);
    let curr_num = (curr_val.0, curr_val.1, curr_val.2);

    if cand_num > curr_num {
        return true;
    }
    if cand_num < curr_num {
        return false;
    }

    match (cand_val.3.is_empty(), curr_val.3.is_empty()) {
        (true, false) => true,
        (false, true) => false,
        _ => cand_val.3 > curr_val.3,
    }
}
