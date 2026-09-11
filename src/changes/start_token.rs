use crate::common::hub_helper;
use crate::common::json_output;
use crate::hub::Hub;
use serde::Serialize;
use std::error;
use std::fmt::Display;
use std::fmt::Formatter;

pub struct Config {
    pub json: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StartTokenJson {
    start_page_token: String,
}

pub async fn start_token(config: Config) -> Result<(), Error> {
    let hub = hub_helper::get_hub().await.map_err(Error::Hub)?;

    let token = get_start_page_token(&hub)
        .await
        .map_err(Error::GetStartPageToken)?;

    if config.json {
        json_output::print_json(&StartTokenJson {
            start_page_token: token,
        });
    } else {
        println!("{}", token);
    }

    Ok(())
}

/// Returns the token that `changes list` needs to report every change made
/// from now on.
pub async fn get_start_page_token(hub: &Hub) -> Result<String, google_drive3::Error> {
    let (_, token) = hub
        .changes()
        .get_start_page_token()
        .supports_all_drives(true)
        .add_scope(google_drive3::api::Scope::Full)
        .doit()
        .await?;

    Ok(token.start_page_token.unwrap_or_default())
}

#[derive(Debug)]
pub enum Error {
    Hub(hub_helper::Error),
    GetStartPageToken(google_drive3::Error),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Hub(err) => write!(f, "{}", err),
            Error::GetStartPageToken(err) => {
                write!(f, "Failed to get the changes start page token: {}", err)
            }
        }
    }
}
