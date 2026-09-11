use crate::common::hub_helper;
use crate::common::json_output;
use crate::common::json_output::FileJson;
use crate::common::json_output::FILE_FIELDS;
use crate::hub::Hub;
use std::error;
use std::fmt::Display;
use std::fmt::Formatter;

pub struct Config {
    pub file_id: String,
    /// `true` moves the file to the trash, `false` restores it.
    pub trashed: bool,
    pub json: bool,
}

/// Moves a file to the trash or restores it. Unlike `files delete`, Drive
/// keeps a trashed file for 30 days and it can be restored.
pub async fn trash(config: Config) -> Result<(), Error> {
    let hub = hub_helper::get_hub().await.map_err(Error::Hub)?;

    let file = set_trashed(&hub, &config.file_id, config.trashed)
        .await
        .map_err(Error::Update)?;

    if config.json {
        json_output::print_json(&FileJson::from(&file));
    } else if config.trashed {
        println!("Moved '{}' to the trash", file.name.unwrap_or_default());
    } else {
        println!(
            "Restored '{}' from the trash",
            file.name.unwrap_or_default()
        );
    }

    Ok(())
}

pub async fn set_trashed(
    hub: &Hub,
    file_id: &str,
    trashed: bool,
) -> Result<google_drive3::api::File, google_drive3::Error> {
    let patch = google_drive3::api::File {
        trashed: Some(trashed),
        ..google_drive3::api::File::default()
    };

    let (_, file) = hub
        .files()
        .update(patch, file_id)
        .param("fields", FILE_FIELDS)
        .supports_all_drives(true)
        .add_scope(google_drive3::api::Scope::Full)
        .doit_without_upload()
        .await?;

    Ok(file)
}

#[derive(Debug)]
pub enum Error {
    Hub(hub_helper::Error),
    Update(google_drive3::Error),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Hub(err) => write!(f, "{}", err),
            Error::Update(err) => write!(f, "Failed to update the file: {}", err),
        }
    }
}
