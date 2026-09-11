use clap::{Parser, Subcommand};
use gdrive::about;
use gdrive::account;
use gdrive::app_config;
use gdrive::changes;
use gdrive::common::delegate::ChunkSize;
use gdrive::common::exit_code::ExitCode;
use gdrive::common::key_value;
use gdrive::common::permission;
use gdrive::drives;
use gdrive::files;
use gdrive::files::list::ListQuery;
use gdrive::files::list::ListSortOrder;
use gdrive::permissions;
use gdrive::revisions;
use gdrive::version;
use mime::Mime;
use std::error::Error;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None, disable_version_flag = true)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Use this account for this command only, without changing the current account. Can also be set with the GDRIVE_ACCOUNT environment variable
    #[arg(long, global = true, value_name = "ACCOUNT_NAME")]
    account: Option<String>,
}

#[derive(Subcommand)]
enum Command {
    /// Print information about gdrive
    About,

    /// Commands for managing accounts
    Account {
        #[command(subcommand)]
        command: AccountCommand,
    },

    /// Commands for managing drives
    Drives {
        #[command(subcommand)]
        command: DriveCommand,
    },

    /// Commands for managing files
    Files {
        #[command(subcommand)]
        command: FileCommand,
    },

    /// Commands for managing file permissions
    Permissions {
        #[command(subcommand)]
        command: PermissionCommand,
    },

    /// Print version information
    Version,
}

#[derive(Subcommand)]
enum AccountCommand {
    /// Add an account
    Add,

    /// List all accounts
    List,

    /// Print current account
    Current,

    /// Switch to a different account
    Switch {
        /// Account name
        account_name: String,
    },

    /// Remove an account
    Remove {
        /// Account name
        account_name: String,
    },

    /// Export account, this will create a zip file of the account which can be imported
    Export {
        /// Account name
        account_name: String,
    },

    /// Import account that was created with the export command
    Import {
        /// Path to archive
        file_path: PathBuf,
    },
}

#[derive(Subcommand)]
enum DriveCommand {
    /// List drives
    List {
        /// Don't print header
        #[arg(long)]
        skip_header: bool,

        /// Field separator
        #[arg(long, default_value_t = String::from("\t"))]
        field_separator: String,
    },
}

#[derive(Subcommand)]
enum FileCommand {
    /// Print file info
    Info {
        /// File id
        file_id: String,

        /// Display size in bytes
        #[arg(long, default_value_t = false)]
        size_in_bytes: bool,

        /// Print the file as json, including md5, head revision and app properties
        #[arg(long)]
        json: bool,
    },

    /// List files
    List {
        /// Max files to list
        #[arg(long, default_value_t = 30)]
        max: usize,

        /// Query. See https://developers.google.com/drive/search-parameters
        #[arg(long, default_value_t = ListQuery::default())]
        query: ListQuery,

        /// Order by. See https://developers.google.com/drive/api/v3/reference/files/list
        #[arg(long, default_value_t = ListSortOrder::default())]
        order_by: ListSortOrder,

        /// List files in a specific folder
        #[arg(long, value_name = "DIRECTORY_ID")]
        parent: Option<String>,

        /// List files on a shared drive
        #[arg(long, value_name = "DRIVE_ID")]
        drive: Option<String>,

        /// Don't print header
        #[arg(long)]
        skip_header: bool,

        /// Show full file name without truncating
        #[arg(long)]
        full_name: bool,

        /// Field separator
        #[arg(long, default_value_t = String::from("\t"))]
        field_separator: String,

        /// Print the files as a json array
        #[arg(long)]
        json: bool,
    },

    /// Download file
    Download {
        /// File id
        file_id: String,

        /// Overwrite existing files and folders
        #[arg(long)]
        overwrite: bool,

        /// Follow shortcut and download target file (does not work with recursive download)
        #[arg(long)]
        follow_shortcuts: bool,

        /// Download directories
        #[arg(long)]
        recursive: bool,

        /// Path where the file/directory should be downloaded to
        #[arg(long, value_name = "PATH")]
        destination: Option<PathBuf>,

        /// Write file to stdout
        #[arg(long)]
        stdout: bool,
    },

    /// Upload file
    Upload {
        /// Path of file to upload
        file_path: Option<PathBuf>,

        /// Force mime type [default: auto-detect]
        #[arg(long, value_name = "MIME_TYPE")]
        mime: Option<Mime>,

        /// Upload to an existing directory
        #[arg(long, value_name = "DIRECTORY_ID")]
        parent: Option<Vec<String>>,

        /// Upload directories. Note that this will always create a new directory on drive and will not update existing directories with the same name
        #[arg(long)]
        recursive: bool,

        /// Set chunk size in MB, must be a power of two.
        #[arg(long, value_name = "1|2|4|8|16|32|64|128|256|512|1024|4096|8192", default_value_t = ChunkSize::default())]
        chunk_size: ChunkSize,

        /// Print errors occuring during chunk upload
        #[arg(long, value_name = "", default_value_t = false)]
        print_chunk_errors: bool,

        /// Print details about each chunk
        #[arg(long, value_name = "", default_value_t = false)]
        print_chunk_info: bool,

        /// Print only id of file/folder
        #[arg(long, default_value_t = false)]
        print_only_id: bool,

        /// Set an app property on the uploaded file, as KEY=VALUE. Can be repeated. Not applied with --recursive
        #[arg(long = "app-property", value_name = "KEY=VALUE", value_parser = key_value::parse_app_property)]
        app_properties: Vec<(String, String)>,

        /// Print the uploaded file as json
        #[arg(long)]
        json: bool,
    },

    /// Update file. This will create a new version of the file. The older versions will typically be kept for 30 days.
    Update {
        /// File id of the file you want ot update
        file_id: String,

        /// Path of file to upload
        file_path: Option<PathBuf>,

        /// Force mime type [default: auto-detect]
        #[arg(long, value_name = "MIME_TYPE")]
        mime: Option<Mime>,

        /// Set chunk size in MB, must be a power of two.
        #[arg(long, value_name = "1|2|4|8|16|32|64|128|256|512|1024|4096|8192", default_value_t = ChunkSize::default())]
        chunk_size: ChunkSize,

        /// Print errors occuring during chunk upload
        #[arg(long, value_name = "", default_value_t = false)]
        print_chunk_errors: bool,

        /// Print details about each chunk
        #[arg(long, value_name = "", default_value_t = false)]
        print_chunk_info: bool,

        /// Keep the new revision forever. Otherwise Drive purges it 30 days after newer content is uploaded
        #[arg(long)]
        keep_revision_forever: bool,

        /// Set an app property on the file, as KEY=VALUE. Can be repeated
        #[arg(long = "app-property", value_name = "KEY=VALUE", value_parser = key_value::parse_app_property)]
        app_properties: Vec<(String, String)>,

        /// Only update if the file's md5 on Drive is this one, otherwise exit with code 7 without uploading. The check runs right before the upload, so verify the revisions afterwards if a concurrent writer must never be missed
        #[arg(long, value_name = "MD5")]
        if_md5: Option<String>,

        /// Print the updated file as json
        #[arg(long)]
        json: bool,
    },

    /// Delete file
    Delete {
        /// File id
        file_id: String,

        /// Delete directory and all it's content
        #[arg(long)]
        recursive: bool,
    },

    /// Move a file to the trash. Unlike delete, Drive keeps it for 30 days and untrash restores it
    Trash {
        /// File id
        file_id: String,

        /// Print the file as json
        #[arg(long)]
        json: bool,
    },

    /// Restore a file from the trash
    Untrash {
        /// File id
        file_id: String,

        /// Print the file as json
        #[arg(long)]
        json: bool,
    },

    /// Create directory
    Mkdir {
        /// Name
        name: String,

        /// Create in an existing directory
        #[arg(long, value_name = "DIRECTORY_ID")]
        parent: Option<Vec<String>>,

        /// Print only id of folder
        #[arg(long, default_value_t = false)]
        print_only_id: bool,

        /// Print the created directory as json
        #[arg(long)]
        json: bool,
    },

    /// Rename file/directory
    Rename {
        /// Id of file or directory
        file_id: String,

        /// New name
        name: String,
    },

    /// Move file/directory
    Move {
        /// Id of file or directory to move
        file_id: String,

        /// Id of folder to move to
        folder_id: String,
    },

    /// Copy file
    Copy {
        /// Id of file or directory to move
        file_id: String,

        /// Id of folder to copy to
        folder_id: String,
    },

    /// Import file as a google document/spreadsheet/presentation.
    /// Example of file types that can be imported: doc, docx, odt, pdf, html, xls, xlsx, csv, ods, ppt, pptx, odp
    Import {
        /// Path to file
        file_path: PathBuf,

        /// Upload to an existing directory
        #[arg(long, value_name = "DIRECTORY_ID")]
        parent: Option<Vec<String>>,

        /// Print only id of file
        #[arg(long, default_value_t = false)]
        print_only_id: bool,
    },

    /// Export google document to file
    Export {
        /// File id
        file_id: String,

        /// File path to export to. The file extension will determine the export format
        file_path: PathBuf,

        /// Overwrite existing files
        #[arg(long)]
        overwrite: bool,
    },

    /// Commands for managing file revisions
    Revisions {
        #[command(subcommand)]
        command: RevisionCommand,
    },

    /// Commands for listing changes to files
    Changes {
        #[command(subcommand)]
        command: ChangeCommand,
    },
}

#[derive(Subcommand)]
enum RevisionCommand {
    /// List the revisions of a file, oldest first
    List {
        /// File id
        file_id: String,

        /// Print the revisions as a json array
        #[arg(long)]
        json: bool,

        /// Don't print header
        #[arg(long)]
        skip_header: bool,

        /// Field separator
        #[arg(long, default_value_t = String::from("\t"))]
        field_separator: String,
    },

    /// Download a revision of a file. Its md5 is verified before the file is written
    Download {
        /// File id
        file_id: String,

        /// Revision id
        revision_id: String,

        /// Path of the file to write
        #[arg(
            long,
            value_name = "FILE_PATH",
            required_unless_present = "stdout",
            conflicts_with = "stdout"
        )]
        destination: Option<PathBuf>,

        /// Overwrite the destination file if it exists
        #[arg(long)]
        overwrite: bool,

        /// Write the revision to stdout
        #[arg(long)]
        stdout: bool,
    },

    /// Keep a revision forever. Drive allows this on at most 200 revisions per file
    Keep {
        /// File id
        file_id: String,

        /// Revision id
        revision_id: String,

        /// Stop keeping the revision forever, so Drive purges it 30 days after newer content is uploaded
        #[arg(long)]
        unset: bool,

        /// Print the revision as json
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum ChangeCommand {
    /// Print the page token that lists the changes made from now on
    StartToken {
        /// Print the token as json
        #[arg(long)]
        json: bool,
    },

    /// List the changes made since a page token
    List {
        /// Page token from `changes start-token` or from a previous `changes list`
        page_token: String,

        /// Max changes to list
        #[arg(long, default_value_t = 1000)]
        max: usize,

        /// Print the changes and the next page tokens as json
        #[arg(long)]
        json: bool,

        /// Don't print header
        #[arg(long)]
        skip_header: bool,

        /// Field separator
        #[arg(long, default_value_t = String::from("\t"))]
        field_separator: String,
    },
}

#[derive(Subcommand)]
enum PermissionCommand {
    /// Grant permission to file
    Share {
        /// File id
        file_id: String,

        /// The role granted by this permission. Allowed values are: owner, organizer, fileOrganizer, writer, commenter, reader
        #[arg(long, default_value_t = permission::Role::default())]
        role: permission::Role,

        /// The type of the grantee. Valid values are: user, group, domain, anyone
        #[arg(long, default_value_t = permission::Type::default())]
        type_: permission::Type,

        /// Email address. Required for user and group type
        #[arg(long)]
        email: Option<String>,

        /// Domain. Required for domain type
        #[arg(long)]
        domain: Option<String>,

        /// Whether the permission allows the file to be discovered through search. This is only applicable for permissions of type domain or anyone
        #[arg(long)]
        discoverable: bool,
    },

    /// List permissions for a file
    List {
        /// File id
        file_id: String,

        /// Don't print header
        #[arg(long)]
        skip_header: bool,

        /// Field separator
        #[arg(long, default_value_t = String::from("\t"))]
        field_separator: String,
    },

    /// Revoke permissions for a file. If no other options are specified, the 'anyone' permission will be revoked
    Revoke {
        /// File id
        file_id: String,

        /// Revoke all permissions (except owner)
        #[arg(long)]
        all: bool,

        /// Revoke specific permission
        #[arg(long, value_name = "PERMISSION_ID")]
        id: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if let Some(account_name) = &cli.account {
        // Read by `AppConfig::load_current_account`. Set before any other
        // thread exists, so no reader races with the write.
        std::env::set_var(app_config::ACCOUNT_ENV_VAR, account_name);
    }

    match cli.command {
        Command::About => {
            // fmt
            about::about()
        }

        Command::Account { command } => {
            // fmt
            match command {
                AccountCommand::Add => {
                    // fmt
                    account::add().await.unwrap_or_else(handle_error)
                }

                AccountCommand::List => {
                    // fmt
                    account::list().unwrap_or_else(handle_error)
                }

                AccountCommand::Current => {
                    // fmt
                    account::current().unwrap_or_else(handle_error)
                }

                AccountCommand::Switch { account_name } => {
                    // fmt
                    account::switch(account::switch::Config { account_name })
                        .unwrap_or_else(handle_error)
                }

                AccountCommand::Remove { account_name } => {
                    // fmt
                    account::remove(account::remove::Config { account_name })
                        .unwrap_or_else(handle_error)
                }

                AccountCommand::Export { account_name } => {
                    // fmt
                    account::export(account::export::Config { account_name })
                        .unwrap_or_else(handle_error)
                }

                AccountCommand::Import { file_path } => {
                    // fmt
                    account::import(account::import::Config {
                        archive_path: file_path,
                    })
                    .unwrap_or_else(handle_error)
                }
            }
        }

        Command::Drives { command } => {
            // fmt
            match command {
                DriveCommand::List {
                    skip_header,
                    field_separator,
                } => drives::list(drives::list::Config {
                    skip_header,
                    field_separator,
                })
                .await
                .unwrap_or_else(handle_error),
            }
        }

        Command::Files { command } => {
            match command {
                FileCommand::Info {
                    file_id,
                    size_in_bytes,
                    json,
                } => {
                    // fmt
                    files::info(files::info::Config {
                        file_id,
                        size_in_bytes,
                        json,
                    })
                    .await
                    .unwrap_or_else(handle_error_with_code)
                }

                FileCommand::List {
                    max,
                    query,
                    order_by,
                    parent,
                    drive,
                    skip_header,
                    full_name,
                    field_separator,
                    json,
                } => {
                    let parent_query =
                        parent.map(|folder_id| ListQuery::FilesInFolder { folder_id });

                    let drive_query = drive.map(|drive_id| ListQuery::FilesOnDrive { drive_id });

                    let q = parent_query.or(drive_query).unwrap_or(query);

                    files::list(files::list::Config {
                        query: q,
                        order_by,
                        max_files: max,
                        skip_header,
                        truncate_name: !full_name,
                        field_separator,
                        json,
                    })
                    .await
                    .unwrap_or_else(handle_error_with_code)
                }

                FileCommand::Download {
                    file_id,
                    overwrite,
                    follow_shortcuts,
                    recursive,
                    destination,
                    stdout,
                } => {
                    let existing_file_action = if overwrite {
                        files::download::ExistingFileAction::Overwrite
                    } else {
                        files::download::ExistingFileAction::Abort
                    };

                    let dst = if stdout {
                        files::download::Destination::Stdout
                    } else if let Some(path) = destination {
                        files::download::Destination::Path(path)
                    } else {
                        files::download::Destination::CurrentDir
                    };

                    files::download(files::download::Config {
                        file_id,
                        existing_file_action,
                        follow_shortcuts,
                        download_directories: recursive,
                        destination: dst,
                    })
                    .await
                    .unwrap_or_else(handle_error_with_code)
                }

                FileCommand::Upload {
                    file_path,
                    mime,
                    parent,
                    recursive,
                    chunk_size,
                    print_chunk_errors,
                    print_chunk_info,
                    print_only_id,
                    app_properties,
                    json,
                } => {
                    // fmt
                    files::upload(files::upload::Config {
                        file_path,
                        mime_type: mime,
                        parents: parent,
                        chunk_size,
                        print_chunk_errors,
                        print_chunk_info,
                        upload_directories: recursive,
                        print_only_id,
                        app_properties,
                        json,
                    })
                    .await
                    .unwrap_or_else(handle_error_with_code)
                }

                FileCommand::Update {
                    file_id,
                    file_path,
                    mime,
                    chunk_size,
                    print_chunk_errors,
                    print_chunk_info,
                    keep_revision_forever,
                    app_properties,
                    if_md5,
                    json,
                } => {
                    // fmt
                    files::update(files::update::Config {
                        file_id,
                        file_path,
                        mime_type: mime,
                        chunk_size,
                        print_chunk_errors,
                        print_chunk_info,
                        keep_revision_forever,
                        app_properties,
                        if_md5,
                        json,
                    })
                    .await
                    .unwrap_or_else(handle_error_with_code)
                }

                FileCommand::Delete { file_id, recursive } => {
                    // fmt
                    files::delete(files::delete::Config {
                        file_id,
                        delete_directories: recursive,
                    })
                    .await
                    .unwrap_or_else(handle_error_with_code)
                }

                FileCommand::Trash { file_id, json } => files::trash(files::trash::Config {
                    file_id,
                    trashed: true,
                    json,
                })
                .await
                .unwrap_or_else(handle_error_with_code),

                FileCommand::Untrash { file_id, json } => files::trash(files::trash::Config {
                    file_id,
                    trashed: false,
                    json,
                })
                .await
                .unwrap_or_else(handle_error_with_code),

                FileCommand::Mkdir {
                    name,
                    parent,
                    print_only_id,
                    json,
                } => {
                    // fmt
                    files::mkdir(
                        files::mkdir::Config {
                            id: None,
                            name,
                            parents: parent,
                            print_only_id,
                        },
                        json,
                    )
                    .await
                    .unwrap_or_else(handle_error_with_code)
                }

                FileCommand::Rename { file_id, name } => {
                    // fmt
                    files::rename(files::rename::Config { file_id, name })
                        .await
                        .unwrap_or_else(handle_error)
                }

                FileCommand::Move { file_id, folder_id } => {
                    // fmt
                    files::mv(files::mv::Config {
                        file_id,
                        to_folder_id: folder_id,
                    })
                    .await
                    .unwrap_or_else(handle_error)
                }

                FileCommand::Copy { file_id, folder_id } => {
                    // fmt
                    files::copy(files::copy::Config {
                        file_id,
                        to_folder_id: folder_id,
                    })
                    .await
                    .unwrap_or_else(handle_error)
                }

                FileCommand::Import {
                    file_path,
                    parent,
                    print_only_id,
                } => {
                    // fmt
                    files::import(files::import::Config {
                        file_path,
                        parents: parent,
                        print_only_id,
                    })
                    .await
                    .unwrap_or_else(handle_error)
                }

                FileCommand::Export {
                    file_id,
                    file_path,
                    overwrite,
                } => {
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
                    .unwrap_or_else(handle_error)
                }

                FileCommand::Revisions { command } => match command {
                    RevisionCommand::List {
                        file_id,
                        json,
                        skip_header,
                        field_separator,
                    } => revisions::list(revisions::list::Config {
                        file_id,
                        json,
                        skip_header,
                        field_separator,
                    })
                    .await
                    .unwrap_or_else(handle_error_with_code),

                    RevisionCommand::Download {
                        file_id,
                        revision_id,
                        destination,
                        overwrite,
                        stdout,
                    } => {
                        // clap requires --destination unless --stdout is given.
                        let destination = match destination {
                            Some(path) if !stdout => revisions::download::Destination::File(path),
                            _ => revisions::download::Destination::Stdout,
                        };

                        let existing_file_action = if overwrite {
                            files::download::ExistingFileAction::Overwrite
                        } else {
                            files::download::ExistingFileAction::Abort
                        };

                        revisions::download(revisions::download::Config {
                            file_id,
                            revision_id,
                            destination,
                            existing_file_action,
                        })
                        .await
                        .unwrap_or_else(handle_error_with_code)
                    }

                    RevisionCommand::Keep {
                        file_id,
                        revision_id,
                        unset,
                        json,
                    } => revisions::keep(revisions::keep::Config {
                        file_id,
                        revision_id,
                        keep_forever: !unset,
                        json,
                    })
                    .await
                    .unwrap_or_else(handle_error_with_code),
                },

                FileCommand::Changes { command } => match command {
                    ChangeCommand::StartToken { json } => {
                        changes::start_token(changes::start_token::Config { json })
                            .await
                            .unwrap_or_else(handle_error_with_code)
                    }

                    ChangeCommand::List {
                        page_token,
                        max,
                        json,
                        skip_header,
                        field_separator,
                    } => changes::list(changes::list::Config {
                        page_token,
                        max_changes: max,
                        json,
                        skip_header,
                        field_separator,
                    })
                    .await
                    .unwrap_or_else(handle_error_with_code),
                },
            }
        }

        Command::Permissions { command } => {
            match command {
                PermissionCommand::Share {
                    file_id,
                    role,
                    type_,
                    discoverable,
                    email,
                    domain,
                } => {
                    // fmt
                    permissions::share(permissions::share::Config {
                        file_id,
                        role,
                        type_,
                        discoverable,
                        email,
                        domain,
                    })
                    .await
                    .unwrap_or_else(handle_error)
                }

                PermissionCommand::List {
                    file_id,
                    skip_header,
                    field_separator,
                } => {
                    // fmt
                    permissions::list(permissions::list::Config {
                        file_id,
                        skip_header,
                        field_separator,
                    })
                    .await
                    .unwrap_or_else(handle_error)
                }

                PermissionCommand::Revoke { file_id, all, id } => {
                    let action = if all {
                        permissions::revoke::RevokeAction::AllExceptOwner
                    } else if id.is_some() {
                        permissions::revoke::RevokeAction::Id(id.unwrap_or_default())
                    } else {
                        permissions::revoke::RevokeAction::Anyone
                    };

                    permissions::revoke(permissions::revoke::Config { file_id, action })
                        .await
                        .unwrap_or_else(handle_error)
                }
            }
        }

        Command::Version => {
            // fmt
            version::version()
        }
    }
}

fn handle_error(err: impl Error) {
    eprintln!("Error: {}", err);
    std::process::exit(1);
}

/// Like `handle_error`, but exits with a code that tells scripts what kind of
/// failure happened. See `gdrive::common::exit_code`.
fn handle_error_with_code<E: Error + ExitCode>(err: E) {
    eprintln!("Error: {}", err);
    std::process::exit(err.exit_code());
}
