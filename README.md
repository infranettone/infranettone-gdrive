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

- Download the latest binary from [the release section](https://github.com/glotlabs/gdrive/releases)
- Unpack and put the binary somewhere in your PATH (i.e. `/usr/local/bin` on linux and macos)
- Note that the binary is not code signed and will cause a warning on windows and macos when running. This will be fixed later, but for now you can find a workaround via your favorite search engine.

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

## Desktop app

A cross-platform desktop app (Linux, macOS, Windows) built with [Tauri](https://tauri.app). It reads and writes the same `$HOME/.config/gdrive3/` that the CLI uses, so an account added in either one immediately works in the other.

### Download

Installers are attached to each [release](https://github.com/glotlabs/gdrive/releases):

| Platform | Files |
| --- | --- |
| Linux x64 | `.AppImage`, `.deb`, `.rpm` |
| macOS arm64 (Apple Silicon) | `.dmg` |
| macOS x64 (Intel) | `.dmg` |
| Windows x64 | `.msi`, `.exe` |

Both macOS builds are produced on Apple Silicon runners — GitHub has retired its Intel ones — with the x64 `.dmg` cross-compiled.

Like the CLI binaries, these are **not code signed**. macOS Gatekeeper and Windows SmartScreen will warn on first launch; on macOS, right-click the app and choose *Open*. Signing is wired into the release workflow but inert: it only activates for the secrets that are actually set (`APPLE_CERTIFICATE`, `APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID`, and `TAURI_SIGNING_PRIVATE_KEY` for the updater).

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
