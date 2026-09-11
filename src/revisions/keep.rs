use crate::common::hub_helper;
use crate::common::json_output;
use crate::common::json_output::RevisionJson;
use crate::common::json_output::REVISION_FIELDS;
use crate::hub::Hub;
use std::error;
use std::fmt::Display;
use std::fmt::Formatter;

pub struct Config {
    pub file_id: String,
    pub revision_id: String,
    pub keep_forever: bool,
    pub json: bool,
}

pub async fn keep(config: Config) -> Result<(), Error> {
    let hub = hub_helper::get_hub().await.map_err(Error::Hub)?;

    let revision = set_keep_forever(
        &hub,
        &config.file_id,
        &config.revision_id,
        config.keep_forever,
    )
    .await
    .map_err(Error::UpdateRevision)?;

    if config.json {
        json_output::print_json(&RevisionJson::from(&revision));
    } else {
        println!(
            "Revision {} keepForever: {}",
            revision.id.unwrap_or_default(),
            revision.keep_forever.unwrap_or(false)
        );
    }

    Ok(())
}

/// Marks a revision to be kept forever, or lets Drive purge it again 30 days
/// after newer content is uploaded. Drive allows at most 200 kept revisions
/// per file.
pub async fn set_keep_forever(
    hub: &Hub,
    file_id: &str,
    revision_id: &str,
    keep_forever: bool,
) -> Result<google_drive3::api::Revision, google_drive3::Error> {
    let patch = google_drive3::api::Revision {
        keep_forever: Some(keep_forever),
        ..google_drive3::api::Revision::default()
    };

    let (_, revision) = hub
        .revisions()
        .update(patch, file_id, revision_id)
        .param("fields", REVISION_FIELDS)
        .add_scope(google_drive3::api::Scope::Full)
        .doit()
        .await?;

    Ok(revision)
}

#[derive(Debug)]
pub enum Error {
    Hub(hub_helper::Error),
    UpdateRevision(google_drive3::Error),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Hub(err) => write!(f, "{}", err),
            Error::UpdateRevision(err) => write!(f, "Failed to update revision: {}", err),
        }
    }
}
