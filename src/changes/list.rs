use crate::common::hub_helper;
use crate::common::json_output;
use crate::common::json_output::ChangeJson;
use crate::common::json_output::FILE_FIELDS;
use crate::common::table;
use crate::common::table::Table;
use crate::files;
use crate::hub::Hub;
use serde::Serialize;
use std::cmp::min;
use std::error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::io;

const MAX_PAGE_SIZE: usize = 1000;

pub struct Config {
    pub page_token: String,
    pub max_changes: usize,
    pub json: bool,
    pub skip_header: bool,
    pub field_separator: String,
}

pub struct ChangesPage {
    pub changes: Vec<google_drive3::api::Change>,
    /// Set once every change up to now has been listed: store it and pass it
    /// to the next `changes list` to only get newer changes.
    pub new_start_page_token: Option<String>,
    /// Set when `max_changes` stopped the listing early: pass it to the next
    /// `changes list` to continue where this one stopped.
    pub next_page_token: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ChangesPageJson {
    changes: Vec<ChangeJson>,
    new_start_page_token: Option<String>,
    next_page_token: Option<String>,
}

pub async fn list(config: Config) -> Result<(), Error> {
    let hub = hub_helper::get_hub().await.map_err(Error::Hub)?;

    let page = list_changes(&hub, &config.page_token, config.max_changes)
        .await
        .map_err(Error::ListChanges)?;

    if config.json {
        json_output::print_json(&ChangesPageJson {
            changes: page.changes.iter().map(ChangeJson::from).collect(),
            new_start_page_token: page.new_start_page_token,
            next_page_token: page.next_page_token,
        });
        return Ok(());
    }

    let values: Vec<[String; 4]> = page
        .changes
        .into_iter()
        .map(|change| {
            [
                change
                    .time
                    .map(files::info::format_date_time)
                    .unwrap_or_default(),
                change.file_id.unwrap_or_default(),
                files::info::format_bool(change.removed.unwrap_or(false)),
                change.file.and_then(|file| file.name).unwrap_or_default(),
            ]
        })
        .collect();

    let table = Table {
        header: ["Time", "FileId", "Removed", "Name"],
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

    if let Some(token) = page.next_page_token {
        eprintln!(
            "More changes available, continue with page token: {}",
            token
        );
    } else if let Some(token) = page.new_start_page_token {
        eprintln!("No more changes, next start page token: {}", token);
    }

    Ok(())
}

pub async fn list_changes(
    hub: &Hub,
    page_token: &str,
    max_changes: usize,
) -> Result<ChangesPage, google_drive3::Error> {
    let fields = format!(
        "changes(fileId,removed,time,changeType,file({})),newStartPageToken,nextPageToken",
        FILE_FIELDS
    );

    let mut collected: Vec<google_drive3::api::Change> = vec![];
    let mut token = page_token.to_string();

    loop {
        let remaining = max_changes.saturating_sub(collected.len()).max(1);
        let page_size = min(MAX_PAGE_SIZE, remaining);

        let (_, change_list) = hub
            .changes()
            .list(&token)
            .page_size(page_size as i32)
            .include_removed(true)
            .include_items_from_all_drives(true)
            .supports_all_drives(true)
            .spaces("drive")
            .param("fields", &fields)
            .add_scope(google_drive3::api::Scope::Full)
            .doit()
            .await?;

        if let Some(mut changes) = change_list.changes {
            collected.append(&mut changes);
        }

        if change_list.new_start_page_token.is_some() {
            return Ok(ChangesPage {
                changes: collected,
                new_start_page_token: change_list.new_start_page_token,
                next_page_token: None,
            });
        }

        match change_list.next_page_token {
            Some(next) if collected.len() >= max_changes => {
                return Ok(ChangesPage {
                    changes: collected,
                    new_start_page_token: None,
                    next_page_token: Some(next),
                });
            }

            Some(next) => token = next,

            None => {
                return Ok(ChangesPage {
                    changes: collected,
                    new_start_page_token: None,
                    next_page_token: None,
                });
            }
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Hub(hub_helper::Error),
    ListChanges(google_drive3::Error),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Hub(err) => write!(f, "{}", err),
            Error::ListChanges(err) => write!(f, "Failed to list changes: {}", err),
        }
    }
}
