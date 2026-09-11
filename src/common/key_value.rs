use std::collections::HashMap;

/// Drive limits an app property to 124 bytes, key and value combined.
const MAX_APP_PROPERTY_BYTES: usize = 124;

/// Parses a `key=value` command line argument, as taken by `--app-property`.
pub fn parse_app_property(s: &str) -> Result<(String, String), String> {
    let (key, value) = s
        .split_once('=')
        .ok_or_else(|| format!("expected key=value, got '{}'", s))?;

    let key = key.trim();

    if key.is_empty() {
        return Err(format!("the key is empty in '{}'", s));
    }

    if key.len() + value.len() > MAX_APP_PROPERTY_BYTES {
        return Err(format!(
            "'{}' is too long, key and value can be at most {} bytes combined",
            s, MAX_APP_PROPERTY_BYTES
        ));
    }

    Ok((key.to_string(), value.to_string()))
}

/// Collects parsed app properties, or `None` when there are none so the
/// request leaves the file's existing properties untouched.
pub fn to_app_properties(pairs: &[(String, String)]) -> Option<HashMap<String, String>> {
    if pairs.is_empty() {
        None
    } else {
        Some(pairs.iter().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_key_and_value() {
        assert_eq!(
            parse_app_property("device=laptop"),
            Ok(("device".to_string(), "laptop".to_string()))
        );
    }

    #[test]
    fn keeps_equals_signs_in_the_value() {
        assert_eq!(
            parse_app_property("base=a=b"),
            Ok(("base".to_string(), "a=b".to_string()))
        );
    }

    #[test]
    fn allows_an_empty_value() {
        assert_eq!(
            parse_app_property("device="),
            Ok(("device".to_string(), String::new()))
        );
    }

    #[test]
    fn rejects_missing_separator_and_empty_key() {
        assert!(parse_app_property("device").is_err());
        assert!(parse_app_property("=laptop").is_err());
    }

    #[test]
    fn rejects_properties_over_the_drive_limit() {
        let long_value = "x".repeat(MAX_APP_PROPERTY_BYTES);
        assert!(parse_app_property(&format!("k={}", long_value)).is_err());
    }

    #[test]
    fn no_pairs_means_no_properties() {
        assert_eq!(to_app_properties(&[]), None);
    }
}
