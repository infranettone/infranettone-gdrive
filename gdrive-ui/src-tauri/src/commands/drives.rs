use crate::error::UiError;
use gdrive::common::delegate::UploadDelegateConfig;
use gdrive::common::hub_helper;
use gdrive::drives;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveRow {
    pub id: String,
    pub name: String,
}

#[tauri::command]
pub async fn drives_list() -> Result<Vec<DriveRow>, UiError> {
    let hub = hub_helper::get_hub().await.map_err(UiError::from_display)?;

    let found = drives::list::list_drives(&hub, UploadDelegateConfig::default())
        .await
        .map_err(UiError::from_display)?;

    Ok(found
        .into_iter()
        .map(|drive| DriveRow {
            id: drive.id.unwrap_or_default(),
            name: drive.name.unwrap_or_default(),
        })
        .collect())
}
