use crate::common::hub_helper;
use crate::common::json_output::REVISION_FIELDS;
use crate::files;
use crate::files::download::ExistingFileAction;
use crate::hub::Hub;
use google_drive3::hyper;
use std::error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::path::PathBuf;

pub struct Config {
    pub file_id: String,
    pub revision_id: String,
    pub destination: Destination,
    pub existing_file_action: ExistingFileAction,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Destination {
    /// Exact path of the file to write, not a directory.
    File(PathBuf),
    Stdout,
}

pub async fn download(config: Config) -> Result<(), Error> {
    let hub = hub_helper::get_hub().await.map_err(Error::Hub)?;

    let revision = get_revision(&hub, &config.file_id, &config.revision_id)
        .await
        .map_err(Error::GetRevision)?;

    if let Destination::File(path) = &config.destination {
        if path.exists() && config.existing_file_action == ExistingFileAction::Abort {
            return Err(Error::FileExists(path.clone()));
        }
    }

    let body = download_revision(&hub, &config.file_id, &config.revision_id)
        .await
        .map_err(Error::DownloadRevision)?;

    match &config.destination {
        Destination::Stdout => {
            // fmt
            files::download::save_body_to_stdout(body)
                .await
                .map_err(Error::Save)?
        }

        Destination::File(path) => {
            // The md5 is checked before the file is moved into place, so a
            // corrupted download never replaces an existing file.
            files::download::save_body_to_file(body, path, revision.md5_checksum.clone())
                .await
                .map_err(Error::Save)?;

            println!(
                "Downloaded revision {} to {}",
                config.revision_id,
                path.display()
            );
        }
    }

    Ok(())
}

pub async fn get_revision(
    hub: &Hub,
    file_id: &str,
    revision_id: &str,
) -> Result<google_drive3::api::Revision, google_drive3::Error> {
    let (_, revision) = hub
        .revisions()
        .get(file_id, revision_id)
        .param("fields", REVISION_FIELDS)
        .add_scope(google_drive3::api::Scope::Full)
        .doit()
        .await?;

    Ok(revision)
}

pub async fn download_revision(
    hub: &Hub,
    file_id: &str,
    revision_id: &str,
) -> Result<hyper::Body, google_drive3::Error> {
    let (response, _) = hub
        .revisions()
        .get(file_id, revision_id)
        .param("alt", "media")
        .add_scope(google_drive3::api::Scope::Full)
        .doit()
        .await?;

    Ok(response.into_body())
}

#[derive(Debug)]
pub enum Error {
    Hub(hub_helper::Error),
    GetRevision(google_drive3::Error),
    DownloadRevision(google_drive3::Error),
    FileExists(PathBuf),
    Save(files::download::Error),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Hub(err) => write!(f, "{}", err),
            Error::GetRevision(err) => write!(f, "Failed to get revision: {}", err),
            Error::DownloadRevision(err) => write!(f, "Failed to download revision: {}", err),
            Error::FileExists(path) => write!(
                f,
                "File '{}' already exists, use --overwrite to overwrite it",
                path.display()
            ),
            Error::Save(err) => write!(f, "{}", err),
        }
    }
}
