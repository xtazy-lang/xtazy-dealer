use crate::error::{DealerError, DealerResult};
use crate::project::dependency::{ParsedDependency, parse_dependency_line, tokenize_line};

#[derive(Debug, Clone)]
pub(crate) struct ProjectDeclaration {
    pub(crate) is_app: bool,
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) xtazy_pin: Option<String>,
    pub(crate) dependencies: Vec<ParsedDependency>,
}

pub(crate) fn parse_project_file(content: &str) -> DealerResult<ProjectDeclaration> {
    let lines: Vec<&str> = content.lines().collect();

    // Validate that every non-empty line (including comment-only lines) has no spaces in leading indentation
    for (line_num, line) in lines.iter().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let first_non_ws = line.find(|c: char| !c.is_whitespace()).unwrap();
        let leading = &line[..first_non_ws];
        if leading.contains(' ') {
            return Err(DealerError::Project(format!(
                "line {}: invalid indentation: structural indentation must use tabs only, spaces are not allowed",
                line_num + 1
            )));
        }
    }

    // Find first non-empty line to get the app/package declaration
    let mut decl_idx = None;
    for (i, line) in lines.iter().enumerate() {
        if !line.trim().is_empty() {
            decl_idx = Some(i);
            break;
        }
    }

    let decl_idx = decl_idx.ok_or_else(|| DealerError::Project("empty file".to_string()))?;
    let decl_line = lines[decl_idx].trim();
    let decl_tokens = tokenize_line(decl_line);
    if decl_tokens.len() < 3 {
        return Err(DealerError::Project(
            "invalid project declaration: project declaration requires a version".to_string(),
        ));
    }
    let is_app = match decl_tokens[0].as_str() {
        "app" => true,
        "package" => false,
        _ => {
            return Err(DealerError::Project(
                "project file must declare 'app' or 'package' on the first line".to_string(),
            ));
        }
    };
    let name = decl_tokens[1].clone();
    let version = decl_tokens[2].clone();

    // Find body indent
    let mut body_indent = None;
    for line in &lines[decl_idx + 1..] {
        let code = strip_comments(line);
        if code.trim().is_empty() {
            continue;
        }
        body_indent = Some(count_leading_spaces_or_tabs(line));
        break;
    }

    let mut dealer_headers = Vec::new();
    for (i, line) in lines.iter().enumerate().skip(decl_idx + 1) {
        let code = strip_comments(line);
        if code.trim().is_empty() {
            continue;
        }
        let indent = count_leading_spaces_or_tabs(line);
        let trimmed = code.trim();
        if trimmed == "dealer" || (trimmed.starts_with("dealer ") && trimmed.contains("xtazy")) {
            if indent == 0 {
                return Err(DealerError::Project(
                    "dealer block must be indented as a root child block".to_string(),
                ));
            }
            if body_indent.filter(|&bi| indent == bi).is_some() {
                dealer_headers.push(i);
            }
        }
    }

    if dealer_headers.len() > 1 {
        return Err(DealerError::Project(
            "duplicate top-level dealer block found".to_string(),
        ));
    }

    let mut xtazy_pin = None;
    let mut dependencies = Vec::new();

    if let Some(&h_idx) = dealer_headers.first() {
        let h_indent = body_indent.unwrap();
        let code = strip_comments(lines[h_idx]);
        let trimmed = code.trim();
        let tokens = tokenize_line(trimmed);
        if tokens.len() == 3 && tokens[1] == "xtazy" {
            xtazy_pin = Some(tokens[2].clone());
        }

        for line in &lines[h_idx + 1..] {
            let code = strip_comments(line);
            if code.trim().is_empty() {
                continue;
            }
            let indent = count_leading_spaces_or_tabs(line);
            if indent <= h_indent {
                break;
            }
            let trimmed = code.trim();
            if let Some(dep) = parse_dependency_line(trimmed) {
                dependencies.push(dep);
            } else {
                return Err(DealerError::Project(format!(
                    "invalid dependency declaration in dealer block: '{}'",
                    line.trim()
                )));
            }
        }
    }

    Ok(ProjectDeclaration {
        is_app,
        name,
        version,
        xtazy_pin,
        dependencies,
    })
}

pub(crate) fn count_leading_spaces_or_tabs(s: &str) -> usize {
    let mut count = 0;
    for ch in s.chars() {
        if ch == '\t' {
            count += 1;
        } else {
            break;
        }
    }
    count
}

pub(crate) fn strip_comments(line: &str) -> &str {
    let mut in_quotes = false;
    let mut comment_byte_idx = None;
    let bytes = line.as_bytes();
    for (char_idx, ch) in line.char_indices() {
        if ch == '"' {
            in_quotes = !in_quotes;
        } else if !in_quotes
            && (ch == '#'
                || (ch == '/' && char_idx + 1 < bytes.len() && bytes[char_idx + 1] == b'/'))
        {
            comment_byte_idx = Some(char_idx);
            break;
        }
    }
    if let Some(idx) = comment_byte_idx {
        line[..idx].trim_end()
    } else {
        line
    }
}
