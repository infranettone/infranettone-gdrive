# gdrive-ui

The cross-platform desktop app for [gdrive](../README.md), built with [Tauri v2](https://tauri.app) (Rust backend, React + TypeScript frontend).

It links the `gdrive` crate directly as a library rather than shelling out to the CLI, so it gets typed results and structured errors instead of parsed table output. Accounts live in the same `$HOME/.config/gdrive3/`, so the CLI and the app are interchangeable.

## Layout

```
gdrive-ui/
  src/               frontend
    lib/api.ts       typed wrappers around invoke() and the oauth:// events
    lib/i18n.ts      every user-visible string, in English and Spanish
    routes/          Accounts, Files, Drives, Permissions
    wizard/          the guided "Add Google account" flow
  src-tauri/         Rust backend
    src/commands/    one module per CLI noun
    src/error.rs     UiError: a code, a message, and a remedy
    src/state.rs     holds the in-flight OAuth task so it can be cancelled
```

## Prerequisites

- Rust (stable) and Node 20+
- **Linux only** — the system webview and its build dependencies:

  ```sh
  sudo apt-get install -y \
    libwebkit2gtk-4.1-dev librsvg2-dev patchelf \
    build-essential curl wget file libssl-dev libayatana-appindicator3-dev libxdo-dev
  ```

  (`4.1`, not `4.0` — Tauri v2 requires it.) macOS and Windows need nothing beyond Xcode command line tools / MSVC build tools.

## Develop

```sh
cd gdrive-ui
npm install
npm run tauri dev
```

## Build installers

```sh
npm run tauri build
```

Bundles land in `../target/release/bundle/`. CI does this for all platforms in [`.github/workflows/release.yaml`](../.github/workflows/release.yaml).

The app's version is not set here: `src-tauri/Cargo.toml` inherits it from `[workspace.package]` in the root `Cargo.toml`, `tauri.conf.json` has no `version` key so Tauri falls back to the crate's, and `package.json` carries none at all. See [Releasing](../README.md#releasing).

## Checks

```sh
npm run typecheck          # frontend
npm run build              # frontend; also required before the Rust build,
                           # because generate_context! reads dist/
cargo clippy -p gdrive-ui --all-targets -- -D warnings
cargo test -p gdrive-ui
```

## Notes

- The OAuth flow blocks in Rust until Google redirects to `localhost:8085`, so it is driven by events (`oauth://url`, `oauth://done`, `oauth://error`) rather than a command's return value. Only one flow can run at a time — the port is fixed.
- Leaving the authorize screen aborts the task, which releases the port. Anything that starts a flow must also be able to cancel it.
- The `@tauri-apps/plugin-*` npm packages are pinned to exact versions that match the `tauri-plugin-*` crates in `Cargo.lock`. The Tauri CLI refuses to build on a major/minor mismatch, so bump both sides together — and note that the newest plugin crates currently pull a `zbus` that needs a newer rustc than 1.98.
- The client secret lives only in React state and, once the flow succeeds, in `secret.json`. Wizard progress in `localStorage` deliberately excludes it.
