//! Exit codes for the commands that programs script, so a caller can tell a
//! missing file from a network blip or a failed precondition without parsing
//! error messages.

use crate::changes;
use crate::common::hub_helper;
use crate::files;
use crate::revisions;

pub const GENERIC: i32 = 1;
// 2 is taken by clap for invalid command line arguments.
pub const NOT_FOUND: i32 = 3;
/// No usable account, expired or revoked credentials, or no permission.
pub const AUTH: i32 = 4;
/// The request did not reach Drive or the connection dropped. Retryable.
pub const NETWORK: i32 = 5;
/// Drive asked to slow down. Retryable after a backoff.
pub const RATE_LIMITED: i32 = 6;
/// A condition such as `files update --if-md5` did not hold; nothing changed.
pub const PRECONDITION_FAILED: i32 = 7;
/// Downloaded content did not match the md5 Drive reported.
pub const MD5_MISMATCH: i32 = 8;
/// Drive returned a 5xx error. Retryable.
pub const SERVER_ERROR: i32 = 9;
/// The local destination exists and `--overwrite` was not given.
pub const FILE_EXISTS: i32 = 10;

pub trait ExitCode {
    fn exit_code(&self) -> i32;
}

pub fn for_http_status(status: u16) -> i32 {
    match status {
        401 | 403 => AUTH,
        404 => NOT_FOUND,
        409 | 412 => PRECONDITION_FAILED,
        429 => RATE_LIMITED,
        500..=599 => SERVER_ERROR,
        _ => GENERIC,
    }
}

/// Classifies a Drive json error body. Drive reports rate limits as 403, so
/// the reason has to be checked before the status code.
pub fn for_error_body(body: &serde_json::Value) -> i32 {
    let error = &body["error"];

    let rate_limited = error["errors"]
        .as_array()
        .map(|errors| {
            errors.iter().any(|err| {
                matches!(
                    err["reason"].as_str(),
                    Some(
                        "rateLimitExceeded" | "userRateLimitExceeded" | "sharingRateLimitExceeded"
                    )
                )
            })
        })
        .unwrap_or(false);

    if rate_limited {
        return RATE_LIMITED;
    }

    error["code"]
        .as_u64()
        .and_then(|code| u16::try_from(code).ok())
        .map(for_http_status)
        .unwrap_or(GENERIC)
}

pub fn for_drive_error(err: &google_drive3::Error) -> i32 {
    match err {
        google_drive3::Error::HttpError(_) | google_drive3::Error::Io(_) => NETWORK,
        google_drive3::Error::MissingToken(_) | google_drive3::Error::MissingAPIKey => AUTH,
        google_drive3::Error::BadRequest(body) => for_error_body(body),
        google_drive3::Error::Failure(response) => for_http_status(response.status().as_u16()),
        _ => GENERIC,
    }
}

impl ExitCode for hub_helper::Error {
    fn exit_code(&self) -> i32 {
        AUTH
    }
}

impl ExitCode for files::info::Error {
    fn exit_code(&self) -> i32 {
        match self {
            Self::Hub(err) => err.exit_code(),
            Self::GetFile(err) => for_drive_error(err),
        }
    }
}

impl ExitCode for files::list::Error {
    fn exit_code(&self) -> i32 {
        match self {
            Self::Hub(err) => err.exit_code(),
            Self::ListFiles(err) => for_drive_error(err),
        }
    }
}

impl ExitCode for files::update::Error {
    fn exit_code(&self) -> i32 {
        match self {
            Self::Hub(err) => err.exit_code(),
            Self::GetFile(err) | Self::Update(err) => for_drive_error(err),
            Self::PreconditionFailed { .. } => PRECONDITION_FAILED,
            Self::FileInfo(_) | Self::OpenFile(_, _) => GENERIC,
        }
    }
}

impl ExitCode for files::upload::Error {
    fn exit_code(&self) -> i32 {
        match self {
            Self::Hub(err) => err.exit_code(),
            Self::Upload(err) | Self::Mkdir(err) => for_drive_error(err),
            _ => GENERIC,
        }
    }
}

impl ExitCode for files::download::Error {
    fn exit_code(&self) -> i32 {
        match self {
            Self::Hub(err) => err.exit_code(),
            Self::GetFile(err) | Self::DownloadFile(err) => for_drive_error(err),
            Self::FileExists(_) => FILE_EXISTS,
            Self::Md5Mismatch { .. } => MD5_MISMATCH,
            Self::ReadChunk(_) => NETWORK,
            _ => GENERIC,
        }
    }
}

impl ExitCode for files::delete::Error {
    fn exit_code(&self) -> i32 {
        match self {
            Self::Hub(err) => err.exit_code(),
            Self::GetFile(err) | Self::DeleteFile(err) => for_drive_error(err),
            Self::IsDirectory(_) => GENERIC,
        }
    }
}

impl ExitCode for files::mkdir::Error {
    fn exit_code(&self) -> i32 {
        match self {
            Self::Hub(err) => err.exit_code(),
            Self::CreateDirectory(err) => for_drive_error(err),
        }
    }
}

impl ExitCode for revisions::list::Error {
    fn exit_code(&self) -> i32 {
        match self {
            Self::Hub(err) => err.exit_code(),
            Self::ListRevisions(err) => for_drive_error(err),
        }
    }
}

impl ExitCode for revisions::download::Error {
    fn exit_code(&self) -> i32 {
        match self {
            Self::Hub(err) => err.exit_code(),
            Self::GetRevision(err) | Self::DownloadRevision(err) => for_drive_error(err),
            Self::FileExists(_) => FILE_EXISTS,
            Self::Save(err) => err.exit_code(),
        }
    }
}

impl ExitCode for revisions::keep::Error {
    fn exit_code(&self) -> i32 {
        match self {
            Self::Hub(err) => err.exit_code(),
            Self::UpdateRevision(err) => for_drive_error(err),
        }
    }
}

impl ExitCode for changes::start_token::Error {
    fn exit_code(&self) -> i32 {
        match self {
            Self::Hub(err) => err.exit_code(),
            Self::GetStartPageToken(err) => for_drive_error(err),
        }
    }
}

impl ExitCode for changes::list::Error {
    fn exit_code(&self) -> i32 {
        match self {
            Self::Hub(err) => err.exit_code(),
            Self::ListChanges(err) => for_drive_error(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn maps_http_status_codes() {
        assert_eq!(for_http_status(404), NOT_FOUND);
        assert_eq!(for_http_status(401), AUTH);
        assert_eq!(for_http_status(412), PRECONDITION_FAILED);
        assert_eq!(for_http_status(429), RATE_LIMITED);
        assert_eq!(for_http_status(503), SERVER_ERROR);
        assert_eq!(for_http_status(400), GENERIC);
    }

    #[test]
    fn rate_limit_reason_wins_over_forbidden_status() {
        let body = json!({
            "error": {
                "code": 403,
                "errors": [{ "reason": "userRateLimitExceeded" }]
            }
        });

        assert_eq!(for_error_body(&body), RATE_LIMITED);
    }

    #[test]
    fn uses_the_code_of_the_error_body() {
        let body = json!({ "error": { "code": 404, "errors": [{ "reason": "notFound" }] } });
        assert_eq!(for_error_body(&body), NOT_FOUND);
    }

    #[test]
    fn unknown_error_body_is_generic() {
        assert_eq!(for_error_body(&json!({})), GENERIC);
    }
}
