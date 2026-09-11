use google_drive3::api::Change;
use google_drive3::api::File;
use google_drive3::api::Revision;
use google_drive3::api::User;
use serde::Serialize;
use std::collections::HashMap;

/// Fields requested for every file a command prints.
///
/// Besides what the tables show, this includes what a program needs to detect
/// changes safely from `--json` output: the md5, the head revision, the
/// version counter, who modified the file and its app properties.
pub const FILE_FIELDS: &str = "id,name,size,createdTime,modifiedTime,md5Checksum,mimeType,parents,shared,description,webContentLink,webViewLink,headRevisionId,version,trashed,appProperties,lastModifyingUser(displayName,emailAddress)";

/// Fields requested for every revision a command prints.
pub const REVISION_FIELDS: &str = "id,md5Checksum,modifiedTime,keepForever,size,mimeType,originalFilename,lastModifyingUser(displayName,emailAddress)";

/// Prints a value as pretty json on stdout.
pub fn print_json<T: Serialize>(value: &T) {
    match serde_json::to_string_pretty(value) {
        Ok(json) => println!("{}", json),
        Err(err) => eprintln!("Warning: Failed to serialize output as json: {}", err),
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserJson {
    pub display_name: Option<String>,
    pub email_address: Option<String>,
}

impl From<&User> for UserJson {
    fn from(user: &User) -> Self {
        UserJson {
            display_name: user.display_name.clone(),
            email_address: user.email_address.clone(),
        }
    }
}

/// Stable json shape of a Drive file. Times are RFC 3339 in UTC, as returned
/// by the server, so they can be compared across machines.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileJson {
    pub id: Option<String>,
    pub name: Option<String>,
    pub mime_type: Option<String>,
    pub size: Option<i64>,
    pub md5_checksum: Option<String>,
    pub created_time: Option<String>,
    pub modified_time: Option<String>,
    pub parents: Vec<String>,
    pub head_revision_id: Option<String>,
    pub version: Option<i64>,
    pub trashed: bool,
    pub app_properties: HashMap<String, String>,
    pub last_modifying_user: Option<UserJson>,
}

impl From<&File> for FileJson {
    fn from(file: &File) -> Self {
        FileJson {
            id: file.id.clone(),
            name: file.name.clone(),
            mime_type: file.mime_type.clone(),
            size: file.size,
            md5_checksum: file.md5_checksum.clone(),
            created_time: file.created_time.map(|time| time.to_rfc3339()),
            modified_time: file.modified_time.map(|time| time.to_rfc3339()),
            parents: file.parents.clone().unwrap_or_default(),
            head_revision_id: file.head_revision_id.clone(),
            version: file.version,
            trashed: file.trashed.unwrap_or(false),
            app_properties: file.app_properties.clone().unwrap_or_default(),
            last_modifying_user: file.last_modifying_user.as_ref().map(UserJson::from),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RevisionJson {
    pub id: Option<String>,
    pub md5_checksum: Option<String>,
    pub modified_time: Option<String>,
    pub keep_forever: bool,
    pub size: Option<i64>,
    pub mime_type: Option<String>,
    pub original_filename: Option<String>,
    pub last_modifying_user: Option<UserJson>,
}

impl From<&Revision> for RevisionJson {
    fn from(revision: &Revision) -> Self {
        RevisionJson {
            id: revision.id.clone(),
            md5_checksum: revision.md5_checksum.clone(),
            modified_time: revision.modified_time.map(|time| time.to_rfc3339()),
            keep_forever: revision.keep_forever.unwrap_or(false),
            size: revision.size,
            mime_type: revision.mime_type.clone(),
            original_filename: revision.original_filename.clone(),
            last_modifying_user: revision.last_modifying_user.as_ref().map(UserJson::from),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeJson {
    pub file_id: Option<String>,
    pub removed: bool,
    pub time: Option<String>,
    pub change_type: Option<String>,
    pub file: Option<FileJson>,
}

impl From<&Change> for ChangeJson {
    fn from(change: &Change) -> Self {
        ChangeJson {
            file_id: change.file_id.clone(),
            removed: change.removed.unwrap_or(false),
            time: change.time.map(|time| time.to_rfc3339()),
            change_type: change.change_type.clone(),
            file: change.file.as_ref().map(FileJson::from),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_json_uses_camel_case_and_defaults() {
        let file = File {
            id: Some("abc".to_string()),
            head_revision_id: Some("rev1".to_string()),
            md5_checksum: Some("d41d8cd98f00b204e9800998ecf8427e".to_string()),
            ..File::default()
        };

        let json = serde_json::to_value(FileJson::from(&file)).unwrap();

        assert_eq!(json["id"], "abc");
        assert_eq!(json["headRevisionId"], "rev1");
        assert_eq!(json["md5Checksum"], "d41d8cd98f00b204e9800998ecf8427e");
        assert_eq!(json["trashed"], false);
        assert_eq!(json["parents"], serde_json::json!([]));
        assert_eq!(json["appProperties"], serde_json::json!({}));
    }

    #[test]
    fn revision_json_defaults_keep_forever_to_false() {
        let revision = Revision {
            id: Some("rev1".to_string()),
            ..Revision::default()
        };

        let json = serde_json::to_value(RevisionJson::from(&revision)).unwrap();

        assert_eq!(json["id"], "rev1");
        assert_eq!(json["keepForever"], false);
    }
}
