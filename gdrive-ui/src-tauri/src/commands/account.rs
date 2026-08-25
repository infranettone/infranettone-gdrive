use crate::error::UiError;
use crate::state::OauthState;
use gdrive::account;
use gdrive::app_config;
use gdrive::app_config::AppConfig;
use gdrive::hub;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

/// Event names the wizard listens to during the OAuth flow.
pub const EVENT_URL: &str = "oauth://url";
pub const EVENT_DONE: &str = "oauth://done";
pub const EVENT_ERROR: &str = "oauth://error";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountSummary {
    pub name: String,
    pub is_current: bool,
    pub path: PathBuf,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretDto {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddedAccountDto {
    pub email: String,
    pub base_path: String,
}

#[tauri::command]
pub fn account_list() -> Result<Vec<AccountSummary>, UiError> {
    let names = app_config::list_accounts().map_err(UiError::from_display)?;
    let current = current_account_name();
    let base_path = AppConfig::default_base_path().map_err(UiError::from_display)?;

    Ok(names
        .into_iter()
        .map(|name| AccountSummary {
            is_current: current.as_deref() == Some(name.as_str()),
            path: base_path.join(&name),
            name,
        })
        .collect())
}

#[tauri::command]
pub fn account_current() -> Option<String> {
    current_account_name()
}

#[tauri::command]
pub fn account_switch(account_name: String) -> Result<(), UiError> {
    account::switch(account::switch::Config { account_name }).map_err(UiError::from_display)
}

#[tauri::command]
pub fn account_remove(account_name: String) -> Result<(), UiError> {
    account::remove(account::remove::Config { account_name }).map_err(UiError::from_display)
}

#[tauri::command]
pub fn account_export(account_name: String) -> Result<(), UiError> {
    account::export(account::export::Config { account_name }).map_err(UiError::from_display)
}

#[tauri::command]
pub fn account_import(archive_path: PathBuf) -> Result<(), UiError> {
    account::import(account::import::Config { archive_path }).map_err(UiError::from_display)
}

/// Read a `client_secret_*.json` downloaded from Google Cloud Console and
/// return the credentials, so the wizard can fill both fields without the user
/// copying anything by hand.
#[tauri::command]
pub fn parse_credentials_file(path: PathBuf) -> Result<SecretDto, UiError> {
    let content = std::fs::read_to_string(&path).map_err(|err| {
        UiError::new(
            "read_file",
            format!("Could not read {}: {}", path.display(), err),
            Some("Pick the .json file that Google Cloud Console downloaded when you created the OAuth client."),
        )
    })?;

    parse_credentials_content(&content)
}

/// Same as [`parse_credentials_file`], for content dropped into the window
/// where the frontend already has the text.
#[tauri::command]
pub fn parse_credentials_content(content: &str) -> Result<SecretDto, UiError> {
    let secret = app_config::parse_google_credentials_json(content).map_err(|err| {
        UiError::new(
            "bad_credentials_file",
            err.to_string(),
            Some("Expected the credentials json from Google Cloud Console, containing an \"installed\" section with client_id and client_secret."),
        )
    })?;

    Ok(SecretDto {
        client_id: secret.client_id,
        client_secret: secret.client_secret,
    })
}

/// Whether a string looks like a Google OAuth client id, so the wizard can
/// flag a paste mistake before starting a doomed flow.
#[tauri::command]
pub fn validate_client_id(client_id: &str) -> bool {
    app_config::looks_like_client_id(client_id)
}

/// Whether the OAuth redirect port is available.
#[tauri::command]
pub fn redirect_port_free() -> bool {
    hub::redirect_port_is_free()
}

#[tauri::command]
pub fn redirect_port() -> u16 {
    hub::REDIRECT_PORT
}

/// Start the OAuth flow. Returns as soon as the flow is running; progress is
/// reported through the `oauth://*` events because the flow blocks until the
/// user finishes in the browser.
#[tauri::command]
pub fn account_add_start(
    app: AppHandle,
    state: State<'_, OauthState>,
    client_id: String,
    client_secret: String,
) -> Result<(), UiError> {
    if !hub::redirect_port_is_free() {
        return Err(UiError::new(
            "port_in_use",
            format!("Port {} is already in use", hub::REDIRECT_PORT),
            Some("Close any other running gdrive instance and try again — Google must be able to redirect back to this port."),
        ));
    }

    let secret = app_config::Secret {
        client_id: client_id.trim().to_string(),
        client_secret: client_secret.trim().to_string(),
    };

    let url_app = app.clone();
    let presenter: hub::UrlPresenter = Arc::new(move |url: String| {
        let _ = url_app.emit(EVENT_URL, url);
    });

    let handle = tauri::async_runtime::spawn(async move {
        match account::add::add_with_secret(secret, presenter).await {
            Ok(added) => {
                let _ = app.emit(
                    EVENT_DONE,
                    AddedAccountDto {
                        email: added.email,
                        base_path: added.base_path.to_string_lossy().to_string(),
                    },
                );
            }
            Err(err) => {
                let _ = app.emit(EVENT_ERROR, UiError::from_display(err));
            }
        }
    });

    state.replace(handle);

    Ok(())
}

/// Abort a running OAuth flow. Dropping the task releases the redirect port,
/// so the user can immediately retry.
#[tauri::command]
pub fn account_add_cancel(state: State<'_, OauthState>) {
    state.cancel();
}

fn current_account_name() -> Option<String> {
    AppConfig::load_account_config().ok().map(|c| c.current)
}
