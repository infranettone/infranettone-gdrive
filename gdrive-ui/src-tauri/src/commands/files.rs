use crate::error::UiError;
use gdrive::common::delegate::ChunkSize;
use gdrive::common::delegate::UploadDelegateConfig;
use gdrive::common::drive_file;
use gdrive::common::hub_helper;
use gdrive::files;
use gdrive::files::info::DisplayConfig;
use gdrive::files::list::ListFilesConfig;
use gdrive::files::list::ListQuery;
use gdrive::files::list::ListSortOrder;
use serde::Serialize;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileRow {
    pub id: String,
    pub name: String,
    /// One of `folder`, `regular`, `shortcut`, `document`.
    pub file_type: String,
    pub mime_type: Option<String>,
    pub size: Option<i64>,
    pub size_human: Option<String>,
    pub created_time: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldRow {
    pub name: String,
    pub value: String,
}

/// List files. `folderId` and `driveId` narrow the listing; passing a raw
/// `query` mirrors the CLI's `--query`.
#[tauri::command]
pub async fn files_list(
    folder_id: Option<String>,
    drive_id: Option<String>,
    query: Option<String>,
    max: Option<usize>,
    order_by: Option<String>,
) -> Result<Vec<FileRow>, UiError> {
    let hub = hub_helper::get_hub().await.map_err(UiError::from_display)?;

    // Same precedence as the CLI: parent, then drive, then a raw query.
    let list_query = match (folder_id, drive_id, query) {
        (Some(folder_id), _, _) => ListQuery::FilesInFolder { folder_id },
        (_, Some(drive_id), _) => ListQuery::FilesOnDrive { drive_id },
        (_, _, Some(q)) if !q.trim().is_empty() => ListQuery::Custom(q),
        _ => ListQuery::RootNotTrashed,
    };

    let order_by = order_by
        .filter(|s| !s.trim().is_empty())
        .map(ListSortOrder::Custom)
        .unwrap_or_default();

    let found = files::list::list_files(
        &hub,
        &ListFilesConfig {
            query: list_query,
            order_by,
            max_files: max.unwrap_or(100),
        },
    )
    .await
    .map_err(UiError::from_display)?;

    Ok(found.into_iter().map(to_row).collect())
}

#[tauri::command]
pub async fn files_info(file_id: String, size_in_bytes: bool) -> Result<Vec<FieldRow>, UiError> {
    let hub = hub_helper::get_hub().await.map_err(UiError::from_display)?;

    let file = files::info::get_file(&hub, &file_id)
        .await
        .map_err(UiError::from_display)?;

    let fields = files::info::prepare_fields(&file, &DisplayConfig { size_in_bytes });

    Ok(fields
        .into_iter()
        .filter_map(|field| {
            field.value.map(|value| FieldRow {
                name: field.name,
                value,
            })
        })
        .collect())
}

#[tauri::command]
pub async fn files_mkdir(name: String, parents: Option<Vec<String>>) -> Result<FileRow, UiError> {
    let hub = hub_helper::get_hub().await.map_err(UiError::from_display)?;

    let file = files::mkdir::create_directory(
        &hub,
        &files::mkdir::Config {
            id: None,
            name,
            parents,
            print_only_id: true,
        },
        UploadDelegateConfig::default(),
    )
    .await
    .map_err(UiError::from_display)?;

    Ok(to_row(file))
}

#[tauri::command]
pub async fn files_rename(file_id: String, name: String) -> Result<(), UiError> {
    files::rename(files::rename::Config { file_id, name })
        .await
        .map_err(UiError::from_display)
}

#[tauri::command]
pub async fn files_move(file_id: String, folder_id: String) -> Result<(), UiError> {
    files::mv(files::mv::Config {
        file_id,
        to_folder_id: folder_id,
    })
    .await
    .map_err(UiError::from_display)
}

#[tauri::command]
pub async fn files_copy(file_id: String, folder_id: String) -> Result<(), UiError> {
    files::copy(files::copy::Config {
        file_id,
        to_folder_id: folder_id,
    })
    .await
    .map_err(UiError::from_display)
}

#[tauri::command]
pub async fn files_delete(file_id: String, recursive: bool) -> Result<(), UiError> {
    files::delete(files::delete::Config {
        file_id,
        delete_directories: recursive,
    })
    .await
    .map_err(UiError::from_display)
}

#[tauri::command]
pub async fn files_upload(
    file_path: PathBuf,
    parents: Option<Vec<String>>,
    recursive: bool,
    chunk_size_mb: Option<u64>,
) -> Result<(), UiError> {
    files::upload(files::upload::Config {
        file_path: Some(file_path),
        mime_type: None,
        parents,
        chunk_size: chunk_size(chunk_size_mb)?,
        print_chunk_errors: false,
        print_chunk_info: false,
        upload_directories: recursive,
        print_only_id: false,
    })
    .await
    .map_err(UiError::from_display)
}

#[tauri::command]
pub async fn files_download(
    file_id: String,
    destination: PathBuf,
    overwrite: bool,
    recursive: bool,
    follow_shortcuts: bool,
) -> Result<(), UiError> {
    let existing_file_action = if overwrite {
        files::download::ExistingFileAction::Overwrite
    } else {
        files::download::ExistingFileAction::Abort
    };

    files::download(files::download::Config {
        file_id,
        existing_file_action,
        follow_shortcuts,
        download_directories: recursive,
        destination: files::download::Destination::Path(destination),
    })
    .await
    .map_err(UiError::from_display)
}

#[tauri::command]
pub async fn files_export(
    file_id: String,
    file_path: PathBuf,
    overwrite: bool,
) -> Result<(), UiError> {
    let existing_file_action = if overwrite {
        files::export::ExistingFileAction::Overwrite
    } else {
        files::export::ExistingFileAction::Abort
    };

    files::export(files::export::Config {
        file_id,
        file_path,
        existing_file_action,
    })
    .await
    .map_err(UiError::from_display)
}

#[tauri::command]
pub async fn files_import(file_path: PathBuf, parents: Option<Vec<String>>) -> Result<(), UiError> {
    files::import(files::import::Config {
        file_path,
        parents,
        print_only_id: false,
    })
    .await
    .map_err(UiError::from_display)
}

fn chunk_size(megabytes: Option<u64>) -> Result<ChunkSize, UiError> {
    match megabytes {
        None => Ok(ChunkSize::default()),
        Some(mb) => ChunkSize::from_str(&mb.to_string()).map_err(|err| {
            UiError::new(
                "bad_chunk_size",
                err,
                Some("The chunk size must be a power of two, in megabytes (1, 2, 4, 8, …)."),
            )
        }),
    }
}

fn to_row(file: google_drive3::api::File) -> FileRow {
    let file_type = if drive_file::is_directory(&file) {
        "folder"
    } else if drive_file::is_binary(&file) {
        "regular"
    } else if drive_file::is_shortcut(&file) {
        "shortcut"
    } else {
        "document"
    };

    FileRow {
        id: file.id.clone().unwrap_or_default(),
        name: file.name.clone().unwrap_or_default(),
        file_type: file_type.to_string(),
        mime_type: file.mime_type.clone(),
        size: file.size,
        size_human: file
            .size
            .map(|bytes| files::info::format_bytes(bytes, &DisplayConfig::default())),
        created_time: file.created_time.map(files::info::format_date_time),
    }
}
