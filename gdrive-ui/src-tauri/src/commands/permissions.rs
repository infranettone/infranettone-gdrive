use crate::error::UiError;
use gdrive::common::delegate::UploadDelegateConfig;
use gdrive::common::hub_helper;
use gdrive::common::permission;
use gdrive::permissions;
use serde::Serialize;
use std::str::FromStr;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionRow {
    pub id: String,
    pub role: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub email_address: Option<String>,
    pub domain: Option<String>,
    pub discoverable: Option<bool>,
}

#[tauri::command]
pub async fn permissions_list(file_id: String) -> Result<Vec<PermissionRow>, UiError> {
    let hub = hub_helper::get_hub().await.map_err(UiError::from_display)?;

    let found =
        permissions::list::list_permissions(&hub, UploadDelegateConfig::default(), &file_id)
            .await
            .map_err(UiError::from_display)?;

    Ok(found
        .into_iter()
        .map(|p| PermissionRow {
            id: p.id.unwrap_or_default(),
            role: p.role.unwrap_or_default(),
            type_: p.type_.unwrap_or_default(),
            email_address: p.email_address,
            domain: p.domain,
            discoverable: p.allow_file_discovery,
        })
        .collect())
}

#[tauri::command]
pub async fn permissions_share(
    file_id: String,
    role: String,
    permission_type: String,
    discoverable: bool,
    email: Option<String>,
    domain: Option<String>,
) -> Result<(), UiError> {
    let role =
        permission::Role::from_str(&role).map_err(|err| UiError::new("bad_role", err, None))?;

    let type_ = permission::Type::from_str(&permission_type)
        .map_err(|err| UiError::new("bad_type", err, None))?;

    permissions::share(permissions::share::Config {
        file_id,
        role,
        type_,
        discoverable,
        email: email.filter(|s| !s.trim().is_empty()),
        domain: domain.filter(|s| !s.trim().is_empty()),
    })
    .await
    .map_err(UiError::from_display)
}

/// Revoke access. With no `permissionId` and `all` false this revokes the
/// `anyone` permission, matching the CLI default.
#[tauri::command]
pub async fn permissions_revoke(
    file_id: String,
    all: bool,
    permission_id: Option<String>,
) -> Result<(), UiError> {
    let action = if all {
        permissions::revoke::RevokeAction::AllExceptOwner
    } else if let Some(id) = permission_id.filter(|s| !s.trim().is_empty()) {
        permissions::revoke::RevokeAction::Id(id)
    } else {
        permissions::revoke::RevokeAction::Anyone
    };

    permissions::revoke(permissions::revoke::Config { file_id, action })
        .await
        .map_err(UiError::from_display)
}
