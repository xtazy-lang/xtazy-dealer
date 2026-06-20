use crate::error::DealerError;

pub(crate) fn not_implemented(feature: &str) -> String {
    DealerError::NotImplemented {
        feature: feature.to_string(),
    }
    .to_string()
}

pub(crate) fn check_passed() -> &'static str {
    "Xtazy project check passed"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_implemented_is_consistent() {
        assert_eq!(
            not_implemented("self update"),
            "self update is recognized but not implemented yet"
        );
    }
}
