use crate::common::hub_helper;
use crate::common::json_output;
use crate::common::json_output::RevisionJson;
use crate::common::json_output::REVISION_FIELDS;
use crate::common::table;
use crate::common::table::Table;
use crate::files;
use crate::files::info::DisplayConfig;
use crate::hub::Hub;
use std::error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::io;

const MAX_PAGE_SIZE: i32 = 1000;

pub struct Config {
    pub file_id: String,
    pub json: bool,
    pub skip_header: bool,
    pub field_separator: String,
}

pub async fn list(config: Config) -> Result<(), Error> {
    let hub = hub_helper::get_hub().await.map_err(Error::Hub)?;

    let revisions = list_revisions(&hub, &config.file_id)
        .await
        .map_err(Error::ListRevisions)?;

    if config.json {
        let revisions: Vec<RevisionJson> = revisions.iter().map(RevisionJson::from).collect();
        json_output::print_json(&revisions);
        return Ok(());
    }

    let values: Vec<[String; 6]> = revisions
        .into_iter()
        .map(|revision| {
            [
                revision.id.unwrap_or_default(),
                revision
                    .modified_time
                    .map(files::info::format_date_time)
                    .unwrap_or_default(),
                revision
                    .size
                    .map(|bytes| files::info::format_bytes(bytes, &DisplayConfig::default()))
                    .unwrap_or_default(),
                revision.md5_checksum.unwrap_or_default(),
                files::info::format_bool(revision.keep_forever.unwrap_or(false)),
                revision
                    .last_modifying_user
                    .and_then(|user| user.display_name)
                    .unwrap_or_default(),
            ]
        })
        .collect();

    let table = Table {
        header: ["Id", "Modified", "Size", "MD5", "KeepForever", "ModifiedBy"],
        values,
    };

    let _ = table::write(
        io::stdout(),
        table,
        &table::DisplayConfig {
            skip_header: config.skip_header,
            separator: config.field_separator,
        },
    );

    Ok(())
}

/// Lists every revision of a file, oldest first.
pub async fn list_revisions(
    hub: &Hub,
    file_id: &str,
) -> Result<Vec<google_drive3::api::Revision>, google_drive3::Error> {
    let fields = format!("revisions({}),nextPageToken", REVISION_FIELDS);
    let mut collected: Vec<google_drive3::api::Revision> = vec![];
    let mut next_page_token: Option<String> = None;

    loop {
        let mut req = hub.revisions().list(file_id).page_size(MAX_PAGE_SIZE);

        if let Some(token) = &next_page_token {
            req = req.page_token(token);
        }

        let (_, revision_list) = req
            .param("fields", &fields)
            .add_scope(google_drive3::api::Scope::Full)
            .doit()
            .await?;

        if let Some(mut revisions) = revision_list.revisions {
            collected.append(&mut revisions);
        }

        next_page_token = revision_list.next_page_token;

        if next_page_token.is_none() {
            break;
        }
    }

    Ok(collected)
}

#[derive(Debug)]
pub enum Error {
    Hub(hub_helper::Error),
    ListRevisions(google_drive3::Error),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Hub(err) => write!(f, "{}", err),
            Error::ListRevisions(err) => write!(f, "Failed to list revisions: {}", err),
        }
    }
}
