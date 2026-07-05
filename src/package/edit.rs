use crate::error::{DealerError, DealerResult};

pub(crate) struct TokenSpan {
    pub(crate) text: String,
    pub(crate) start: usize,
    pub(crate) end: usize,
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

pub(crate) fn find_dealer_block_range(
    lines: &[&str],
) -> DealerResult<Option<(usize, usize, usize)>> {
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

    let mut decl_idx = None;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("app ") || trimmed.starts_with("package ") {
            decl_idx = Some(i);
            break;
        }
    }
    let Some(d_idx) = decl_idx else {
        return Ok(None);
    };

    let mut body_indent = None;
    for line in &lines[d_idx + 1..] {
        let code = split_code_and_comment(line).0;
        if code.trim().is_empty() {
            continue;
        }
        body_indent = Some(count_leading_spaces_or_tabs(line));
        break;
    }

    let b_indent = match body_indent {
        Some(bi) => bi,
        None => return Ok(None),
    };

    let mut dealer_headers = Vec::new();
    for (i, line) in lines.iter().enumerate().skip(d_idx + 1) {
        let code = split_code_and_comment(line).0;
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
            if indent == b_indent {
                dealer_headers.push((i, indent));
            }
        }
    }

    if dealer_headers.is_empty() {
        return Ok(None);
    }
    if dealer_headers.len() > 1 {
        return Err(DealerError::Project(
            "duplicate top-level dealer block found".to_string(),
        ));
    }

    let (h_idx, h_indent) = dealer_headers[0];
    let mut end_idx = h_idx;

    for (i, line) in lines.iter().enumerate().skip(h_idx + 1) {
        let code = split_code_and_comment(line).0;
        if code.trim().is_empty() {
            continue;
        }
        let indent = count_leading_spaces_or_tabs(line);
        if indent > h_indent {
            end_idx = i;
        } else {
            break;
        }
    }

    Ok(Some((h_idx, h_idx + 1, end_idx)))
}

pub(crate) fn get_token_spans(s: &str) -> Vec<TokenSpan> {
    let mut spans = Vec::new();
    let mut current = String::new();
    let mut start_idx = None;
    let mut in_quotes = false;

    for (idx, ch) in s.char_indices() {
        if ch == '"' {
            if !in_quotes {
                in_quotes = true;
                start_idx = Some(idx);
                current.push(ch);
            } else {
                in_quotes = false;
                current.push(ch);
                if let Some(start) = start_idx {
                    spans.push(TokenSpan {
                        text: current.clone(),
                        start,
                        end: idx + 1,
                    });
                }
                current.clear();
                start_idx = None;
            }
        } else if ch.is_whitespace() && !in_quotes {
            if !current.is_empty() {
                if let Some(start) = start_idx {
                    spans.push(TokenSpan {
                        text: current.clone(),
                        start,
                        end: idx,
                    });
                }
                current.clear();
                start_idx = None;
            }
        } else {
            if current.is_empty() {
                start_idx = Some(idx);
            }
            current.push(ch);
        }
    }
    if let Some(start) = start_idx.filter(|_| !current.is_empty()) {
        spans.push(TokenSpan {
            text: current.clone(),
            start,
            end: s.len(),
        });
    }
    spans
}

pub(crate) fn split_code_and_comment(line: &str) -> (&str, Option<&str>) {
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
        (&line[..idx], Some(&line[idx..]))
    } else {
        (line, None)
    }
}
