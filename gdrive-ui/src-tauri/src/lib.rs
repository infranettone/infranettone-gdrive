pub mod commands;
pub mod error;
pub mod state;

use state::OauthState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(OauthState::default())
        .invoke_handler(tauri::generate_handler![
            commands::account::account_list,
            commands::account::account_current,
            commands::account::account_switch,
            commands::account::account_remove,
            commands::account::account_export,
            commands::account::account_import,
            commands::account::parse_credentials_file,
            commands::account::parse_credentials_content,
            commands::account::validate_client_id,
            commands::account::redirect_port_free,
            commands::account::redirect_port,
            commands::account::account_add_start,
            commands::account::account_add_cancel,
            commands::drives::drives_list,
            commands::files::files_list,
            commands::files::files_info,
            commands::files::files_mkdir,
            commands::files::files_rename,
            commands::files::files_move,
            commands::files::files_copy,
            commands::files::files_delete,
            commands::files::files_upload,
            commands::files::files_download,
            commands::files::files_export,
            commands::files::files_import,
            commands::permissions::permissions_list,
            commands::permissions::permissions_share,
            commands::permissions::permissions_revoke,
        ])
        .run(tauri::generate_context!())
        .expect("error while running gdrive-ui");
}
