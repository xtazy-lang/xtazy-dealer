pub(crate) fn sanitize_package_name(name: &str) -> String {
    let mut clean_name = String::new();
    let mut previous_was_separator = false;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            clean_name.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !previous_was_separator && !clean_name.is_empty() {
            clean_name.push('_');
            previous_was_separator = true;
        }
    }
    while clean_name.ends_with('_') {
        clean_name.pop();
    }
    if clean_name.is_empty() {
        clean_name.push_str("xtazy_project");
    }
    if clean_name.starts_with(|c: char| c.is_ascii_digit()) {
        clean_name = format!("xtazy_{}", clean_name);
    }
    clean_name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitizes_dash_and_leading_digit() {
        assert_eq!(sanitize_package_name("my-app"), "my_app");
        assert_eq!(sanitize_package_name("123-app"), "xtazy_123_app");
    }

    #[test]
    fn sanitizes_spaces_punctuation_and_empty_names() {
        assert_eq!(sanitize_package_name("My App!"), "my_app");
        assert_eq!(sanitize_package_name("hello.world"), "hello_world");
        assert_eq!(sanitize_package_name("___"), "xtazy_project");
    }
}
