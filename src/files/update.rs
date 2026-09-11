use crate::common::delegate::BackoffConfig;
use crate::common::delegate::ChunkSize;
use crate::common::delegate::UploadDelegate;
use crate::common::delegate::UploadDelegateConfig;
use crate::common::file_helper;
use crate::common::file_info;
use crate::common::file_info::FileInfo;
use crate::common::hub_helper;
use crate::common::json_output;
use crate::common::json_output::FileJson;
use crate::common::json_output::FILE_FIELDS;
use crate::common::key_value;
use crate::files;
use crate::files::info;
use crate::files::info::DisplayConfig;
use crate::hub::Hub;
use mime::Mime;
use std::collections::HashMap;
use std::error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::io;
use std::path::PathBuf;
use std::time::Duration;

pub struct Config {
    pub file_id: String,
    pub file_path: Option<PathBuf>,
    pub mime_type: Option<Mime>,
    pub chunk_size: ChunkSize,
    pub print_chunk_errors: bool,
    pub print_chunk_info: bool,
    pub keep_revision_forever: bool,
    pub app_properties: Vec<(String, String)>,
    /// Only update when the file's md5 on Drive is this one.
    pub if_md5: Option<String>,
    pub json: bool,
}

/// Options for the new revision that `update_file` creates.
#[derive(Debug, Clone, Default)]
pub struct UpdateOptions {
    pub keep_revision_forever: bool,
    /// Set on the file; `None` leaves the existing app properties untouched.
    pub app_properties: Option<HashMap<String, String>>,
}

pub async fn update(config: Config) -> Result<(), Error> {
    let hub = hub_helper::get_hub().await.map_err(Error::Hub)?;

    let delegate_config = UploadDelegateConfig {
        chunk_size: config.chunk_size,
        backoff_config: BackoffConfig {
            max_retries: 20,
            min_sleep: Duration::from_secs(1),
            max_sleep: Duration::from_secs(60),
        },
        print_chunk_errors: config.print_chunk_errors,
        print_chunk_info: config.print_chunk_info,
    };

    let (file, file_path) = file_helper::open_file(&config.file_path).map_err(|err| {
        Error::OpenFile(
            config.file_path.unwrap_or_else(|| PathBuf::from("<stdin>")),
            err,
        )
    })?;

    let drive_file = info::get_file(&hub, &config.file_id)
        .await
        .map_err(Error::GetFile)?;

    err_if_md5_mismatch(config.if_md5.as_deref(), &drive_file)?;

    let file_info = FileInfo::from_file(
        &file,
        &file_info::Config {
            file_path: file_path.clone(),
            mime_type: config.mime_type,
            parents: drive_file.parents.clone(),
        },
    )
    .map_err(Error::FileInfo)?;

    let reader = std::io::BufReader::new(file);

    if !config.json {
        println!("Updating {} with {}", config.file_id, file_path.display());
    }

    let options = UpdateOptions {
        keep_revision_forever: config.keep_revision_forever,
        app_properties: key_value::to_app_properties(&config.app_properties),
    };

    let file = update_file(
        &hub,
        reader,
        &config.file_id,
        file_info,
        delegate_config,
        &options,
    )
    .await
    .map_err(Error::Update)?;

    if config.json {
        json_output::print_json(&FileJson::from(&file));
    } else {
        println!("File successfully updated");
        let fields = files::info::prepare_fields(&file, &DisplayConfig::default());
        files::info::print_fields(&fields);
    }

    Ok(())
}

pub async fn update_file<RS>(
    hub: &Hub,
    src_file: RS,
    file_id: &str,
    file_info: FileInfo,
    delegate_config: UploadDelegateConfig,
    options: &UpdateOptions,
) -> Result<google_drive3::api::File, google_drive3::Error>
where
    RS: google_drive3::client::ReadSeek,
{
    let dst_file = google_drive3::api::File {
        name: Some(file_info.name),
        app_properties: options.app_properties.clone(),
        ..google_drive3::api::File::default()
    };

    let mut delegate = UploadDelegate::new(delegate_config);

    let req = hub
        .files()
        .update(dst_file, file_id)
        .param("fields", FILE_FIELDS)
        .keep_revision_forever(options.keep_revision_forever)
        .add_scope(google_drive3::api::Scope::Full)
        .delegate(&mut delegate)
        .supports_all_drives(true);

    let (_, file) = if file_info.size > 0 {
        req.upload_resumable(src_file, file_info.mime_type).await?
    } else {
        req.upload(src_file, file_info.mime_type).await?
    };

    Ok(file)
}

pub async fn update_metadata(
    hub: &Hub,
    delegate_config: UploadDelegateConfig,
    patch_file: PatchFile,
) -> Result<google_drive3::api::File, google_drive3::Error> {
    let mut delegate = UploadDelegate::new(delegate_config);

    let (_, file) = hub
        .files()
        .update(patch_file.file, &patch_file.id)
        .param("fields", FILE_FIELDS)
        .add_scope(google_drive3::api::Scope::Full)
        .delegate(&mut delegate)
        .supports_all_drives(true)
        .doit_without_upload()
        .await?;

    Ok(file)
}

/// Guards against overwriting content that changed on Drive since the caller
/// last saw it. Drive has no conditional upload, so a writer that commits in
/// between this check and the upload still wins: callers that need strict
/// guarantees must also verify the revision history afterwards.
fn err_if_md5_mismatch(
    expected_md5: Option<&str>,
    drive_file: &google_drive3::api::File,
) -> Result<(), Error> {
    let Some(expected) = expected_md5 else {
        return Ok(());
    };

    let actual = drive_file.md5_checksum.clone().unwrap_or_default();

    if actual.eq_ignore_ascii_case(expected.trim()) {
        Ok(())
    } else {
        Err(Error::PreconditionFailed {
            expected: expected.to_string(),
            actual,
        })
    }
}

#[derive(Debug)]
pub enum Error {
    Hub(hub_helper::Error),
    FileInfo(file_info::Error),
    OpenFile(PathBuf, io::Error),
    GetFile(google_drive3::Error),
    Update(google_drive3::Error),
    PreconditionFailed { expected: String, actual: String },
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Hub(err) => write!(f, "{}", err),
            Error::FileInfo(err) => write!(f, "{}", err),
            Error::OpenFile(path, err) => {
                write!(f, "Failed to open file '{}': {}", path.display(), err)
            }
            Error::GetFile(err) => write!(f, "Failed to get file: {}", err),
            Error::Update(err) => write!(f, "Failed to update file: {}", err),
            Error::PreconditionFailed { expected, actual } => write!(
                f,
                "The file changed on Drive, expected md5 {} but it is {}. Nothing was updated",
                expected,
                if actual.is_empty() { "<none>" } else { actual }
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PatchFile {
    id: String,
    file: google_drive3::api::File,
}

impl PatchFile {
    pub fn new(id: String) -> Self {
        Self {
            id,
            file: google_drive3::api::File::default(),
        }
    }

    pub fn with_name(&self, name: &str) -> Self {
        Self {
            file: google_drive3::api::File {
                name: Some(name.to_string()),
                ..self.file.clone()
            },
            ..self.clone()
        }
    }

    pub fn id(&self) -> String {
        self.id.clone()
    }

    pub fn file(&self) -> google_drive3::api::File {
        self.file.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn drive_file_with_md5(md5: Option<&str>) -> google_drive3::api::File {
        google_drive3::api::File {
            md5_checksum: md5.map(|s| s.to_string()),
            ..google_drive3::api::File::default()
        }
    }

    #[test]
    fn no_expected_md5_always_passes() {
        assert!(err_if_md5_mismatch(None, &drive_file_with_md5(Some("abc"))).is_ok());
    }

    #[test]
    fn matching_md5_passes_ignoring_case() {
        assert!(err_if_md5_mismatch(Some("ABC"), &drive_file_with_md5(Some("abc"))).is_ok());
    }

    #[test]
    fn different_md5_fails() {
        let result = err_if_md5_mismatch(Some("abc"), &drive_file_with_md5(Some("def")));
        assert!(matches!(result, Err(Error::PreconditionFailed { .. })));
    }

    #[test]
    fn missing_md5_on_drive_fails() {
        let result = err_if_md5_mismatch(Some("abc"), &drive_file_with_md5(None));
        assert!(matches!(result, Err(Error::PreconditionFailed { .. })));
    }
}
