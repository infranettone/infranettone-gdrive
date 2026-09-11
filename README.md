# gdrive

<img src="https://user-images.githubusercontent.com/720405/210108089-32b7a259-b384-49c3-a2d3-fe07a42791e2.png" width="100">

## Overview

gdrive is a command line application for interacting with Google Drive. This is the successor of [gdrive2](https://github.com/prasmussen/gdrive), though at the moment only the most basic functionality is implemented.

There is also a **desktop app** that shares the same accounts and config as the CLI, and adds a guided wizard for the one genuinely fiddly part: creating your Google OAuth credentials. See [Desktop app](#desktop-app).

## Community

Join our [discord server](https://discord.gg/5fyVwp8559) to discuss everything gdrive.

## Sponsor

Help keep this project alive. By sponsoring the [gdrive tier](https://github.com/sponsors/prasmussen)
you will help support:

- Keeping up with api changes
- Development of new features
- Fixing and answering of issues
- Writing of guides and docs

## Getting started

### Requirements

- Google OAuth Client credentials, see [docs](/docs/create_google_api_credentials.md)

  These are credentials you create yourself, in your own Google Cloud project — gdrive ships with none, so nobody else can reach your files and you don't share anyone's quota. Creating them takes about 5 minutes. If you'd rather be walked through it, the [desktop app](#desktop-app) opens each Google Cloud screen for you and fills the credentials in from the json file Google hands you.

### Install binary

- Download `gdrive_linux-x64.tar.gz` (or `gdrive_windows-x64.zip`) from [the release section](https://github.com/glotlabs/gdrive/releases)
- Unpack and put the binary somewhere in your PATH (i.e. `/usr/local/bin`, or `%LOCALAPPDATA%\Programs\gdrive\bin` on Windows)
- The Linux binary is statically linked (musl), so it runs on any Linux distro
- Releases ship the CLI for Linux and Windows, and the desktop app for Linux only; see [Other platforms](#other-platforms)

### Add google account to gdrive

- Run `gdrive account add`
- This will prompt you for your google Client ID and Client Secret (see [Requirements](#requirements))
- Next you will be presented with an url
- Follow the url and give approval for gdrive to access your Drive
- You will be redirected to `http://localhost:8085` (gdrive starts a temporary web server) which completes the setup
- Gdrive is now ready to use!

### Using gdrive on a remote server

Part of the flow for adding an account to gdrive requires your web browser to access `localhost:8085` on the machine that runs gdrive.
This makes it tricky to set up accounts on remote servers. The suggested workaround is to add the account on your local machine and import it on the remote server:
1. [local] Run `gdrive account add` 
2. [local] Run `gdrive account export <ACCOUNT_NAME>`
3. [local] Copy the exported archive to the remote server
4. [remote] Run `gdrive account import <ARCHIVE_PATH>`

### Credentials
Gdrive saves your account credentials and tokens under `$HOME/.config/gdrive3/`.
You don't usually need to use these files directly, but if someone gets access to them, they will also be able to access your Google Drive. Keep them safe.

### Gdrive on virtual machines in the cloud
There are some issues communicating with the Drive API from certain cloud providers.
For example on an AWS instance the api returns a lot of `429 Too Many Requests` / `503 Service Unavailable` / `502 Bad Gateway` errors while uploading.
While the same file uploads without any errors from a Linode instance.
Gdrive has retry logic built in for these errors, but it can slow down the upload significantly.
To check if you are affected by these errors you can run the `upload` command with these flags: `--print-chunk-errors` `--print-chunk-info`.

## Scripting gdrive

These options make gdrive safe to drive from another program, such as the [KeePass sync agent](https://github.com/infranettone/infranettone-keepass-gdrive-sync-agent).

### Json output

`files info`, `files list`, `files upload`, `files update`, `files mkdir`, `files trash|untrash`, `files revisions list|keep` and `files changes list|start-token` accept `--json`. Files always include `md5Checksum`, `headRevisionId`, `version`, `trashed`, `appProperties` and `lastModifyingUser`; times are RFC 3339 in UTC.

```sh
gdrive files info <FILE_ID> --json
```

### Choosing the account per command

`--account <NAME>` (or the `GDRIVE_ACCOUNT` environment variable) uses that account for one command without reading or changing the account selected with `account switch`.

### Safe updates

- `files update --if-md5 <MD5>` only uploads when the file's md5 on Drive is still the one you expect, and exits with code 7 otherwise. Drive has no conditional upload, so a writer that commits between the check and the upload still wins: check `files revisions list` afterwards if that matters.
- `--keep-revision-forever` stops Drive from purging the new revision 30 days after newer content is uploaded (Drive allows 200 kept revisions per file).
- `--app-property KEY=VALUE` (repeatable, also on `files upload`) tags the file, for example with the device that wrote it.

### Revisions

```sh
gdrive files revisions list <FILE_ID> [--json]
gdrive files revisions download <FILE_ID> <REVISION_ID> --destination <FILE_PATH> [--overwrite]
gdrive files revisions keep <FILE_ID> <REVISION_ID> [--unset]
```

Downloads are checked against the revision's md5 before they replace the destination.

### Changes

```sh
gdrive files changes start-token
gdrive files changes list <PAGE_TOKEN> --json
```

`changes list` returns `newStartPageToken` once it has caught up (store it for the next call), or `nextPageToken` when `--max` stopped it early.

### Trash

```sh
gdrive files trash <FILE_ID>
gdrive files untrash <FILE_ID>
```

Unlike `files delete`, which removes a file for good, a trashed file stays in Drive's trash for 30 days and `files untrash` brings it back. `files info` reports `trashed` for it.

### Exit codes

`files info|list|download|upload|update|delete|trash|untrash|mkdir`, `files revisions` and `files changes` exit with:

| Code | Meaning |
|---|---|
| 0 | Success |
| 1 | Other error |
| 2 | Invalid command line arguments |
| 3 | File or revision not found |
| 4 | No usable account, invalid credentials or no permission |
| 5 | Network error (retryable) |
| 6 | Rate limited by Drive (retry after a backoff) |
| 7 | Precondition failed, e.g. `--if-md5` did not match; nothing changed |
| 8 | Downloaded content did not match its md5 |
| 9 | Drive server error (retryable) |
| 10 | Destination file exists and `--overwrite` was not given |

## Desktop app

A cross-platform desktop app (Linux, macOS, Windows) built with [Tauri](https://tauri.app). It reads and writes the same `$HOME/.config/gdrive3/` that the CLI uses, so an account added in either one immediately works in the other.

### Download

Installers for Linux x64 are attached to each [release](https://github.com/glotlabs/gdrive/releases): `.deb`, `.rpm` and `.AppImage`. The app itself is cross-platform, but releases currently build Linux only — see [Other platforms](#other-platforms).

The installers now ship the `gdrive` CLI too, but bundling it is not the same as putting it on your PATH — where the binary lands depends on the package format. The `.deb` installs it as `/usr/bin/gdrive`, so there it just works. For the other formats, open the app's **Terminal** screen: it tells you whether a terminal can already find `gdrive`, and *Install the command* copies it to `~/.local/bin` (or `%LOCALAPPDATA%\Programs\gdrive\bin` on Windows), naming the line to add to your PATH if that directory isn't on it. No elevation is needed, and uninstalling only ever removes the app's own copy.

It is the same binary as the standalone archives above and uses the same accounts, so those archives are only for wanting the CLI without the app.

Updater signing is wired into the release workflow but inert: it only activates when the `TAURI_SIGNING_PRIVATE_KEY` secret is set.

### The "Add Google account" wizard

The app's main reason to exist. Instead of pointing you at a 28-step document, it walks through the [Requirements](#requirements) in six screens:

1. **Create a Google Cloud project** — opens the project creator; you paste the Project ID back so every later link goes straight to the right screen.
2. **Enable the Google Drive API** — one button, one click.
3. **Configure the consent screen** — including a copy button for the two scopes gdrive needs, a reminder to add yourself as a test user, and a prominent warning to **Publish app** (otherwise Google expires your token after 7 days).
4. **Create the OAuth client ID** — as type *Desktop app*, and download the json.
5. **Enter your credentials** — drag the `client_secret_*.json` onto the window and both fields fill in, or paste them by hand. The Client ID format is validated, and port 8085 is checked before starting.
6. **Authorize** — the consent url opens in your browser and is also shown with a copy button. Cancelling frees the port immediately.

Failures come back with a remedy rather than a raw OAuth error: `invalid_client` points you back at step 4, `access_denied` at the test-user and publish settings in step 3, a busy port tells you to close the other gdrive instance.

Progress through the Google Cloud steps is remembered if you close the app midway. Your client secret is never written to browser storage — only to `$HOME/.config/gdrive3/<account>/secret.json`, which is `chmod 0600` on unix.

The app is available in English and Spanish.

### Building it yourself

See [gdrive-ui/README.md](gdrive-ui/README.md).

## Other platforms

The code is cross-platform, and both the CLI and the desktop app have built and shipped for macOS (arm64 and x64) and Windows x64. To keep CI and releases fast, the pipelines currently build **Linux**, plus the **CLI for Windows** (the KeePass sync agent needs it there). Bringing the other platforms back is deferred until someone needs them. When that happens:

- The last workflows that built every platform are at commit `5b6e1f6` — `git show 5b6e1f6:.github/workflows/release.yaml` (and `ci.yaml`). Restore the `matrix` of the `cli` and `desktop` jobs from there; the Windows CLI already has its own `cli-windows` jobs.
- macOS jobs must run on `macos-14` (Apple Silicon). The Intel `.dmg` is cross-compiled with `--target x86_64-apple-darwin`: GitHub has retired its Intel runners, and a job asking for one waits forever.
- Cross builds pass the triple to the sidecar script: `npm run sidecar -- <triple>`.
- macOS Gatekeeper and Windows SmartScreen warn on unsigned binaries. Apple signing needs the `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_PASSWORD` and `APPLE_TEAM_ID` secrets, promoted into the env the same way `TAURI_SIGNING_PRIVATE_KEY` is.
- The app's Terminal screen already has a Windows code path (it installs into `%LOCALAPPDATA%\Programs\gdrive\bin`), but it has never been exercised on a real machine.

## Releasing

Publishing is a version bump. There is exactly one place to change it:

```toml
# Cargo.toml
[workspace.package]
version = "3.9.2"
```

The CLI crate, the desktop app crate, `tauri.conf.json` and the bundle
filenames all derive from it.

Merge that to `main` and [`.github/workflows/release.yaml`](.github/workflows/release.yaml) does the rest: it tags `v<version>`, opens a draft release, builds every artifact in parallel, uploads them, and publishes the release once they have all succeeded. The release is only undrafted when every platform succeeded, so a half-built release is never visible.

The workflow skips as soon as it sees that version already published, so ordinary pushes to `main` cost one cheap job. If a build fails, the tag and the draft stay behind and re-running the workflow resumes from there.
