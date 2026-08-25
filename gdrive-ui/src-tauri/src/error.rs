use serde::Serialize;

/// An error shaped for the UI: a stable `code` the frontend can branch on, a
/// human message, and an actionable `hint` where we know one.
///
/// The whole point is that a user who mistyped a client secret sees "check the
/// credentials from step 4" instead of a raw OAuth error body.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiError {
    pub code: String,
    pub message: String,
    pub hint: Option<String>,
}

impl UiError {
    pub fn new(code: &str, message: impl Into<String>, hint: Option<&str>) -> Self {
        UiError {
            code: code.to_string(),
            message: message.into(),
            hint: hint.map(String::from),
        }
    }

    /// Wrap any gdrive error, upgrading the ones we recognize into a coded
    /// error with a remedy.
    pub fn from_display(err: impl std::fmt::Display) -> Self {
        let message = err.to_string();
        classify(&message)
    }
}

fn classify(message: &str) -> UiError {
    let haystack = message.to_lowercase();

    if haystack.contains("invalid_client") || haystack.contains("unauthorized_client") {
        return UiError::new(
            "invalid_client",
            message,
            Some("Google rejected the client id / client secret. Make sure you copied them from the OAuth client of type \"Desktop app\" you created in step 4, with no leading or trailing spaces."),
        );
    }

    if haystack.contains("access_denied") {
        return UiError::new(
            "access_denied",
            message,
            Some("Google refused the consent. Either you dismissed the consent screen, or your email is not listed as a test user and the app has not been published (step 3)."),
        );
    }

    if haystack.contains("invalid_grant") {
        return UiError::new(
            "invalid_grant",
            message,
            Some("The stored token is no longer valid. This usually happens when the OAuth consent screen was left in \"Testing\" mode, which expires tokens after 7 days. Publish the app and add the account again."),
        );
    }

    if haystack.contains("address already in use") || haystack.contains("addrinuse") {
        return UiError::new(
            "port_in_use",
            message,
            Some("Port 8085 is taken. Close any other running gdrive instance (or whatever is listening on that port) and try again."),
        );
    }

    if haystack.contains("no account has been selected") {
        return UiError::new(
            "no_account",
            message,
            Some("Add a Google account first, or pick one from the Accounts screen."),
        );
    }

    if haystack.contains("dns")
        || haystack.contains("failed to lookup")
        || haystack.contains("connection refused")
        || haystack.contains("network")
    {
        return UiError::new(
            "network",
            message,
            Some("Could not reach Google. Check your internet connection or proxy settings and retry."),
        );
    }

    UiError::new("error", message, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_known_oauth_failures() {
        assert_eq!(
            classify("Bad Request: invalid_client").code,
            "invalid_client"
        );
        assert_eq!(classify("error: access_denied").code, "access_denied");
        assert_eq!(
            classify("Address already in use (os error 98)").code,
            "port_in_use"
        );
    }

    #[test]
    fn unknown_failures_fall_back_without_a_hint() {
        let err = classify("something exploded");
        assert_eq!(err.code, "error");
        assert!(err.hint.is_none());
    }
}
