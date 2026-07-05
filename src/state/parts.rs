#[derive(Debug, Clone)]
pub(crate) struct XtazyParts {
    pub(crate) _xtazy_version: String,
    pub(crate) piko_version: String,
    pub(crate) piko_hash: String,
    pub(crate) rusttime_version: String,
    pub(crate) rusttime_hash: String,
    pub(crate) std_version: String,
    pub(crate) std_hash: String,
    pub(crate) rust_version: String,
    pub(crate) rust_hash: String,
}

pub(crate) fn parse_xtazy_parts(content: &str) -> Result<XtazyParts, String> {
    let lines: Vec<&str> = content
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();
    if lines.len() != 5 {
        return Err(format!("expected exactly 5 lines, found {}", lines.len()));
    }

    // Line 0: xtazy <version>
    let p0: Vec<&str> = lines[0].split_whitespace().collect();
    if p0.len() != 2 || p0[0] != crate::constants::components::COMPONENT_XTAZY {
        return Err(format!(
            "line 1 must start with '{}'",
            crate::constants::components::COMPONENT_XTAZY
        ));
    }
    let xtazy_version = p0[1].to_string();

    // Line 1: piko <version> sha256:<hash>
    let p1: Vec<&str> = lines[1].split_whitespace().collect();
    if p1.len() != 3 || p1[0] != crate::constants::components::COMPONENT_PIKO {
        return Err(format!(
            "line 2 must start with '{}'",
            crate::constants::components::COMPONENT_PIKO
        ));
    }
    let piko_version = p1[1].to_string();
    let piko_hash = p1[2].to_string();

    // Line 2: rusttime <version> sha256:<hash>
    let p2: Vec<&str> = lines[2].split_whitespace().collect();
    if p2.len() != 3 || p2[0] != crate::constants::components::COMPONENT_RUSTTIME {
        return Err(format!(
            "line 3 must start with '{}'",
            crate::constants::components::COMPONENT_RUSTTIME
        ));
    }
    let rusttime_version = p2[1].to_string();
    let rusttime_hash = p2[2].to_string();

    // Line 3: std <version> sha256:<hash>
    let p3: Vec<&str> = lines[3].split_whitespace().collect();
    if p3.len() != 3 || p3[0] != crate::constants::components::COMPONENT_STD {
        return Err(format!(
            "line 4 must start with '{}'",
            crate::constants::components::COMPONENT_STD
        ));
    }
    let std_version = p3[1].to_string();
    let std_hash = p3[2].to_string();

    // Line 4: rust <version> sha256:<hash>
    let p4: Vec<&str> = lines[4].split_whitespace().collect();
    if p4.len() != 3 || p4[0] != crate::constants::components::COMPONENT_RUST {
        return Err(format!(
            "line 5 must start with '{}'",
            crate::constants::components::COMPONENT_RUST
        ));
    }
    let rust_version = p4[1].to_string();
    let rust_hash = p4[2].to_string();

    Ok(XtazyParts {
        _xtazy_version: xtazy_version,
        piko_version,
        piko_hash,
        rusttime_version,
        rusttime_hash,
        std_version,
        std_hash,
        rust_version,
        rust_hash,
    })
}
