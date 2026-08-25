import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

/** Mirrors `UiError` in src-tauri/src/error.rs. */
export interface UiError {
  code: string;
  message: string;
  hint: string | null;
}

/** Anything that crosses the bridge can reject with a non-UiError, so normalise. */
export function toUiError(err: unknown): UiError {
  if (err && typeof err === "object" && "code" in err && "message" in err) {
    return err as UiError;
  }
  return { code: "error", message: String(err), hint: null };
}

export interface AccountSummary {
  name: string;
  isCurrent: boolean;
  path: string;
}

export interface Secret {
  clientId: string;
  clientSecret: string;
}

export interface AddedAccount {
  email: string;
  basePath: string;
}

export interface FileRow {
  id: string;
  name: string;
  fileType: "folder" | "regular" | "shortcut" | "document";
  mimeType: string | null;
  size: number | null;
  sizeHuman: string | null;
  createdTime: string | null;
}

export interface FieldRow {
  name: string;
  value: string;
}

export interface DriveRow {
  id: string;
  name: string;
}

export interface PermissionRow {
  id: string;
  role: string;
  type: string;
  emailAddress: string | null;
  domain: string | null;
  discoverable: boolean | null;
}

// --- accounts -------------------------------------------------------------

export const accountList = () => invoke<AccountSummary[]>("account_list");
export const accountCurrent = () => invoke<string | null>("account_current");
export const accountSwitch = (accountName: string) =>
  invoke<void>("account_switch", { accountName });
export const accountRemove = (accountName: string) =>
  invoke<void>("account_remove", { accountName });
export const accountExport = (accountName: string) =>
  invoke<void>("account_export", { accountName });
export const accountImport = (archivePath: string) =>
  invoke<void>("account_import", { archivePath });

// --- the add-account wizard ----------------------------------------------

export const parseCredentialsFile = (path: string) =>
  invoke<Secret>("parse_credentials_file", { path });
export const parseCredentialsContent = (content: string) =>
  invoke<Secret>("parse_credentials_content", { content });
export const validateClientId = (clientId: string) =>
  invoke<boolean>("validate_client_id", { clientId });
export const redirectPortFree = () => invoke<boolean>("redirect_port_free");
export const redirectPort = () => invoke<number>("redirect_port");
export const accountAddStart = (clientId: string, clientSecret: string) =>
  invoke<void>("account_add_start", { clientId, clientSecret });
export const accountAddCancel = () => invoke<void>("account_add_cancel");

/**
 * Subscribe to the OAuth flow events. Returns a disposer that removes every
 * listener, so a component can register them in a single effect.
 */
export async function onOauthEvents(handlers: {
  onUrl: (url: string) => void;
  onDone: (account: AddedAccount) => void;
  onError: (err: UiError) => void;
}): Promise<UnlistenFn> {
  const unlisteners = await Promise.all([
    listen<string>("oauth://url", (e) => handlers.onUrl(e.payload)),
    listen<AddedAccount>("oauth://done", (e) => handlers.onDone(e.payload)),
    listen<UiError>("oauth://error", (e) => handlers.onError(e.payload)),
  ]);

  return () => unlisteners.forEach((un) => un());
}

// --- files ----------------------------------------------------------------

export const filesList = (args: {
  folderId?: string | null;
  driveId?: string | null;
  query?: string | null;
  max?: number;
  orderBy?: string | null;
}) => invoke<FileRow[]>("files_list", args);

export const filesInfo = (fileId: string, sizeInBytes = false) =>
  invoke<FieldRow[]>("files_info", { fileId, sizeInBytes });
export const filesMkdir = (name: string, parents?: string[]) =>
  invoke<FileRow>("files_mkdir", { name, parents });
export const filesRename = (fileId: string, name: string) =>
  invoke<void>("files_rename", { fileId, name });
export const filesMove = (fileId: string, folderId: string) =>
  invoke<void>("files_move", { fileId, folderId });
export const filesCopy = (fileId: string, folderId: string) =>
  invoke<void>("files_copy", { fileId, folderId });
export const filesDelete = (fileId: string, recursive: boolean) =>
  invoke<void>("files_delete", { fileId, recursive });
export const filesUpload = (args: {
  filePath: string;
  parents?: string[];
  recursive: boolean;
  chunkSizeMb?: number;
}) => invoke<void>("files_upload", args);
export const filesDownload = (args: {
  fileId: string;
  destination: string;
  overwrite: boolean;
  recursive: boolean;
  followShortcuts: boolean;
}) => invoke<void>("files_download", args);
export const filesExport = (
  fileId: string,
  filePath: string,
  overwrite: boolean,
) => invoke<void>("files_export", { fileId, filePath, overwrite });
export const filesImport = (filePath: string, parents?: string[]) =>
  invoke<void>("files_import", { filePath, parents });

// --- drives & permissions -------------------------------------------------

export const drivesList = () => invoke<DriveRow[]>("drives_list");
export const permissionsList = (fileId: string) =>
  invoke<PermissionRow[]>("permissions_list", { fileId });
export const permissionsShare = (args: {
  fileId: string;
  role: string;
  /** `anyone` | `user` | `group` | `domain` */
  permissionType: string;
  discoverable: boolean;
  email?: string | null;
  domain?: string | null;
}) => invoke<void>("permissions_share", args);
export const permissionsRevoke = (
  fileId: string,
  all: boolean,
  permissionId?: string,
) => invoke<void>("permissions_revoke", { fileId, all, permissionId });
