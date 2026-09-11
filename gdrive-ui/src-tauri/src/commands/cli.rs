//! Installing the bundled `gdrive` CLI onto the user's PATH.
//!
//! The desktop app ships the CLI as a Tauri sidecar, so the installer already
//! puts the binary on disk — but next to the app binary, which is only on the
//! PATH by accident (the .deb and .rpm land in `/usr/bin`, an AppImage, a .app
//! bundle and an .msi do not). These commands copy it somewhere the shell will
//! find it, without ever needing root: a per-user bin directory.

use crate::error::UiError;
use serde::Serialize;
use std::path::{Path, PathBuf};

/// The file name of the CLI on this platform.
#[cfg(windows)]
const BIN_NAME: &str = "gdrive.exe";
#[cfg(not(windows))]
const BIN_NAME: &str = "gdrive";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CliStatus {
    /// Whether the app was built with the CLI sidecar. False only for a
    /// development build that skipped it.
    pub bundled: bool,
    /// The per-user directory `cli_install` copies into.
    pub target_dir: PathBuf,
    /// The full path of the installed copy.
    pub target_path: PathBuf,
    /// Whether `target_dir` is on the PATH this app inherited.
    pub on_path: bool,
    /// What `gdrive` currently resolves to on the PATH, if anything.
    pub resolved: Option<PathBuf>,
    /// True when running `gdrive` already reaches a copy of the bundled CLI —
    /// either because the package installed it somewhere on the PATH, or
    /// because `cli_install` has been run before.
    pub installed: bool,
    /// Whether the copy at `target_path` exists — the only one this app is
    /// entitled to remove.
    pub user_copy: bool,
    /// `gdrive version`'s first line, when a resolved binary answers.
    pub version: Option<String>,
}

#[tauri::command]
pub fn cli_status() -> Result<CliStatus, UiError> {
    let target_dir = install_dir()?;
    let target_path = target_dir.join(BIN_NAME);
    let resolved = which(BIN_NAME);
    let user_copy = target_path.is_file();

    // Installed means "the shell finds one", wherever it came from: our own
    // copy, or the package manager having dropped the sidecar in /usr/bin.
    let installed = resolved.is_some() || user_copy;

    Ok(CliStatus {
        bundled: sidecar_path().is_ok(),
        on_path: path_entries().contains(&target_dir),
        version: resolved.as_deref().and_then(cli_version),
        resolved,
        installed,
        user_copy,
        target_dir,
        target_path,
    })
}

/// Copy the bundled CLI into the per-user bin directory, replacing whatever
/// copy is already there.
#[tauri::command]
pub fn cli_install() -> Result<CliStatus, UiError> {
    let sidecar = sidecar_path()?;
    let target_dir = install_dir()?;
    let target_path = target_dir.join(BIN_NAME);

    std::fs::create_dir_all(&target_dir).map_err(|err| {
        UiError::new(
            "cli_install_failed",
            format!("Could not create {}: {err}", target_dir.display()),
            Some("Check that you have write access to that directory."),
        )
    })?;

    // Copy to a temporary name and rename over the target: replacing a binary
    // in place fails on Windows while it is running, and would leave a
    // half-written file behind on any platform if the copy were interrupted.
    let staging = target_dir.join(format!("{BIN_NAME}.new"));
    let _ = std::fs::remove_file(&staging);

    std::fs::copy(&sidecar, &staging).map_err(|err| {
        UiError::new(
            "cli_install_failed",
            format!("Could not copy the CLI to {}: {err}", staging.display()),
            Some(
                "Check that you have write access to that directory and that there is free space.",
            ),
        )
    })?;

    if let Err(err) = make_executable(&staging) {
        let _ = std::fs::remove_file(&staging);
        return Err(UiError::new(
            "cli_install_failed",
            format!("Could not make {} executable: {err}", staging.display()),
            None,
        ));
    }

    std::fs::rename(&staging, &target_path).map_err(|err| {
        let _ = std::fs::remove_file(&staging);
        UiError::new(
            "cli_install_failed",
            format!(
                "Could not install the CLI to {}: {err}",
                target_path.display()
            ),
            Some("If a gdrive command is running, close it and try again."),
        )
    })?;

    cli_status()
}

/// Remove the copy this app installed. Never touches a `gdrive` that lives
/// anywhere else — a system package, or one the user installed by hand.
#[tauri::command]
pub fn cli_uninstall() -> Result<CliStatus, UiError> {
    let target_path = install_dir()?.join(BIN_NAME);

    match std::fs::remove_file(&target_path) {
        Ok(()) => {}
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => {
            return Err(UiError::new(
                "cli_uninstall_failed",
                format!("Could not remove {}: {err}", target_path.display()),
                None,
            ))
        }
    }

    cli_status()
}

/// The sidecar the bundler placed next to the app executable.
fn sidecar_path() -> Result<PathBuf, UiError> {
    let exe = std::env::current_exe().map_err(|err| {
        UiError::new(
            "cli_not_bundled",
            format!("Could not locate the running executable: {err}"),
            None,
        )
    })?;

    let candidate = exe
        .parent()
        .map(|dir| dir.join(BIN_NAME))
        .filter(|path| path.is_file());

    candidate.ok_or_else(|| {
        UiError::new(
            "cli_not_bundled",
            "This build does not ship the gdrive CLI.",
            Some("Download the CLI archive from the release page, or build the sidecar before bundling (see gdrive-ui/README.md)."),
        )
    })
}

/// Where a user-owned binary goes on this platform. Deliberately per-user:
/// writing to /usr/local/bin or Program Files would need elevation, and the
/// app has no business asking for it.
fn install_dir() -> Result<PathBuf, UiError> {
    let home = home::home_dir().ok_or_else(|| {
        UiError::new(
            "no_home",
            "Could not determine your home directory.",
            Some("Make sure HOME (or USERPROFILE on Windows) is set."),
        )
    })?;

    if cfg!(windows) {
        // The convention Windows itself uses for per-user programs.
        let local = std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| home.join("AppData").join("Local"));
        Ok(local.join("Programs").join("gdrive").join("bin"))
    } else {
        Ok(home.join(".local").join("bin"))
    }
}

fn path_entries() -> Vec<PathBuf> {
    std::env::var_os("PATH")
        .map(|path| std::env::split_paths(&path).collect())
        .unwrap_or_default()
}

/// Minimal PATH lookup. `Command` would find the binary for us, but we need
/// the resolved path itself to tell the user which copy answers.
fn which(name: &str) -> Option<PathBuf> {
    path_entries()
        .into_iter()
        .map(|dir| dir.join(name))
        .find(|candidate| candidate.is_file())
}

fn cli_version(path: &Path) -> Option<String> {
    let output = std::process::Command::new(path)
        .arg("version")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(String::from)
}

#[cfg(unix)]
fn make_executable(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_dir_is_under_the_home_directory() {
        let dir = install_dir().expect("a home directory");
        let home = home::home_dir().expect("a home directory");
        assert!(
            dir.starts_with(&home) || cfg!(windows),
            "{} should live under {}",
            dir.display(),
            home.display()
        );
        assert!(dir.ends_with("bin"));
    }

    #[test]
    fn which_finds_nothing_for_an_implausible_name() {
        assert!(which("gdrive-definitely-not-a-real-binary").is_none());
    }
}
