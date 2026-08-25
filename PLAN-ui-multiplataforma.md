# Plan: UI multiplataforma para `gdrive` (Tauri v2) con asistente guiado de alta de cuenta

## Context

`infranettone-gdrive` es hoy un CLI en Rust puro (`gdrive` 3.9.1, crate binario, clap 4, `google-drive3` desde el branch git `prasmussen/google-apis-rs#resumable-fix`). No existe ninguna UI.

El punto de fricción real de la herramienta es el onboarding: `gdrive account add` ([src/account/add.rs](src/account/add.rs)) pide por stdin un Client ID y un Client Secret que el usuario sólo puede obtener siguiendo a mano las 28 pasos de [docs/create_google_api_credentials.md](docs/create_google_api_credentials.md) en Google Cloud Console. La sección "Requirements" del README se limita a enlazar ese documento. Quien no es desarrollador se pierde ahí y nunca llega a usar la herramienta.

El objetivo es una aplicación de escritorio multiplataforma (Linux/macOS/Windows) que:
1. Guíe paso a paso la creación de credenciales OAuth (los "Requirements") **dentro** de la app, con enlaces directos a cada pantalla de la consola e importación del `client_secret_*.json` que Google descarga.
2. Complete el flujo de consentimiento OAuth y deje la cuenta registrada exactamente en el mismo `~/.config/gdrive3/` que usa el CLI (interoperables).
3. Exponga el resto de operaciones (cuentas, unidades, ficheros, permisos) en una UI usable.
4. Se compile y publique automáticamente desde GitHub Actions para las tres plataformas.

**Decisiones ya tomadas con el usuario:** Tauri v2; refactor a `lib + bin` para llamar la lógica en proceso; wizard con guía in-app **y** import de `credentials.json`.

---

## Fase 1 — Refactor a crate `lib` + `bin` (sin cambios de comportamiento del CLI)

Hoy [src/main.rs](src/main.rs) declara todos los `mod` y contiene el `Cli` de clap. Se separa:

- **Nuevo `src/lib.rs`**: mueve las declaraciones `mod` de `main.rs` y las hace `pub mod`:
  `pub mod about; pub mod account; pub mod app_config; pub mod common; pub mod drives; pub mod files; pub mod hub; pub mod permissions; pub mod version;`
  Los `use crate::…` internos siguen resolviendo igual dentro de la lib, así que **no hay que tocar los módulos existentes** salvo elevar a `pub` lo que sea `pub(crate)`.
- **`src/main.rs`** queda como bin fino: conserva el `enum Cli` de clap, el `match` de despacho y `handle_error` (main.rs:724-727), cambiando `crate::x` por `gdrive::x`.
- **`Cargo.toml`**: añadir
  ```toml
  [lib]
  name = "gdrive"
  path = "src/lib.rs"

  [[bin]]
  name = "gdrive"
  path = "src/main.rs"
  ```
  y convertir el proyecto en workspace con miembros `["." , "gdrive-ui/src-tauri"]` (o mover el crate CLI a `crates/gdrive/`; se opta por lo primero para minimizar el diff y no romper las workflows de release existentes).

**Criterio de aceptación:** `cargo build --release` sigue produciendo el mismo binario y `gdrive files list` funciona igual.

## Fase 2 — Hacer el flujo OAuth pilotable desde una GUI

El obstáculo: [src/hub.rs:105-112](src/hub.rs#L105-L112) `present_user_url` hace `println!` de la URL de consentimiento; y [src/account/add.rs](src/account/add.rs) mezcla prompts de stdin con la lógica. La GUI necesita **recibir** esa URL, no verla en stdout.

- **`src/hub.rs`**: parametrizar el delegate. Añadir
  ```rust
  pub type UrlPresenter = Arc<dyn Fn(String) + Send + Sync>;
  struct AuthDelegate { presenter: UrlPresenter }
  impl Auth {
      pub async fn new(config, tokens_path) -> …            // igual que hoy: presenter = println!
      pub async fn new_with_presenter(config, tokens_path, presenter: UrlPresenter) -> …
  }
  ```
  Se mantiene `InstalledFlowReturnMethod::HTTPPortRedirect(8085)` (hub.rs:66) — el servidor temporal lo levanta `yup-oauth2`, la GUI no necesita implementarlo.
- **`src/account/add.rs`**: extraer el núcleo sin I/O de terminal:
  ```rust
  pub async fn add_with_secret(secret: app_config::Secret, presenter: UrlPresenter)
      -> Result<AddedAccount, Error>   // { email, base_path }
  ```
  que hace tempdir → `Auth::new_with_presenter` → `auth.token(SCOPES)` → `about.get(fields=user)` → `app_config::add_account` → `switch_account` (misma secuencia de add.rs:17-58). `pub async fn add()` pasa a ser el wrapper que hace los `println!` y el `secret_prompt()` y delega en esa función. **Cero cambios visibles en el CLI.**
- Exportar las constantes de scopes (`SCOPES: [&str;2]`) para reutilizarlas.
- **`app_config`**: añadir `pub fn parse_google_credentials_json(content: &str) -> Result<Secret, Error>` que acepte el JSON descargado de Google Cloud (`{"installed":{"client_id":…,"client_secret":…}}` y también la variante `{"web":{…}}`) y devuelva el `Secret` existente (app_config.rs:192-196). Esto habilita el drag&drop del wizard.
- **`app_config`**: añadir `pub fn account_summary(name) -> AccountInfo` / reutilizar `list_accounts()` (app_config.rs:37-52) tal cual para la pantalla de cuentas.

## Fase 3 — Aplicación Tauri v2 (`gdrive-ui/`)

Estructura:

```
gdrive-ui/
  package.json           # vite + react + typescript
  vite.config.ts
  index.html
  src/
    main.tsx
    App.tsx
    lib/api.ts           # wrappers tipados de invoke() + listen()
    routes/
      Accounts.tsx
      Files.tsx          # listado, upload, download, mkdir, rename, move, copy, delete
      Drives.tsx
      Permissions.tsx
    wizard/
      AddAccountWizard.tsx
      steps/*.tsx        # ver desglose abajo
      guide.ts           # contenido de los pasos (i18n es/en)
  src-tauri/
    Cargo.toml           # depende de gdrive = { path = "../.." }, tauri 2, tauri-plugin-{opener,dialog,fs}
    tauri.conf.json
    capabilities/default.json
    icons/
    src/main.rs, commands/{account,files,drives,permissions}.rs, state.rs
```

**Backend Tauri** (`src-tauri/src/commands/`): comandos `#[tauri::command] async` que llaman la lib. Cada error se mapea a un struct serializable `{ code, message, hint }` en lugar de un `String` opaco, para que la UI pueda dar remedios ("el Client ID no está autorizado", "el puerto 8085 está ocupado").

Comandos principales:
- `account_list`, `account_current`, `account_switch`, `account_remove`, `account_export`, `account_import`
- `parse_credentials_file(path) -> Secret` (usa el parser de la Fase 2 + `tauri-plugin-dialog` para el selector)
- `account_add_start(client_id, client_secret) -> ()` — lanza la tarea; el presenter emite el evento `oauth://url` con la URL de consentimiento; al terminar emite `oauth://done { email }` u `oauth://error { … }`
- `account_add_cancel()` — aborta el `JoinHandle` (el listener 8085 se libera al hacer drop)
- `port_8085_free() -> bool` — chequeo previo (`TcpListener::bind`) para avisar antes de empezar
- `files_list`, `files_info`, `files_upload`, `files_download`, `files_mkdir`, `files_rename`, `files_move`, `files_copy`, `files_delete`, `files_export`, `files_import`
- `drives_list`, `permissions_{list,share,revoke}`

Las subidas/bajadas emiten eventos de progreso por fichero; el `--chunk-size` del CLI se expone como ajuste avanzado.

## Fase 4 — El asistente guiado "Add Google Account" (núcleo del encargo)

Wizard de pantalla completa, con barra de progreso lateral y estado persistido (si el usuario cierra la app a mitad, vuelve al mismo paso). Traduce los 28 pasos de `docs/create_google_api_credentials.md` a **6 pasos accionables**, cada uno con: explicación breve, botón "Abrir en el navegador" con la URL exacta (vía `tauri-plugin-opener`), captura de referencia embebida (las mismas imágenes del doc, descargadas a `gdrive-ui/src/wizard/assets/` para que funcione offline) y un check "ya lo he hecho".

| Paso | Contenido | URL que abre el botón |
|---|---|---|
| 0. Bienvenida | Explica qué son las credenciales OAuth, por qué las crea el propio usuario (la app no lleva credenciales embebidas) y que tarda ~5 min. Ofrece atajo **"Ya tengo credenciales"** → salta al paso 4. | — |
| 1. Proyecto | Crear/seleccionar proyecto en Google Cloud. Campo para pegar el **Project ID** (se reutiliza en pasos siguientes y en el nombre de la app del consent screen, como pide el doc paso 9). | `https://console.cloud.google.com/projectcreate` |
| 2. Habilitar Drive API | Un clic en "Enable". Con Project ID conocido el enlace va directo. | `https://console.cloud.google.com/apis/library/drive.googleapis.com?project=<ID>` |
| 3. Pantalla de consentimiento | External → datos de contacto → **scopes** `.../auth/drive` y `.../auth/drive.metadata.readonly` (botón "copiar scopes") → **añadir el propio email como test user** → **Publish app** (crítico: si no, el token caduca a los 7 días; se marca con un aviso destacado). | `https://console.cloud.google.com/apis/credentials/consent?project=<ID>` |
| 4. Crear el Client ID | Tipo **Desktop app**. Al terminar, Google muestra/descarga el JSON. | `https://console.cloud.google.com/apis/credentials/oauthclient?project=<ID>` |
| 5. Introducir credenciales | **Dos vías:** (a) **drag & drop / selector del `client_secret_*.json`** → autorrellena ambos campos vía `parse_credentials_file`; (b) pegado manual. Validación en vivo: el Client ID debe casar `^\d+-[a-z0-9]+\.apps\.googleusercontent\.com$` y advertir si el tipo no parece "Desktop"; el secret no vacío. El secret se muestra enmascarado con botón de revelar. Comprueba además que el puerto 8085 esté libre. | — |
| 6. Autorizar | Llama `account_add_start`. Al recibir `oauth://url` abre el navegador automáticamente **y** muestra la URL con botón "copiar" (fallback para entornos sin navegador). Spinner "Esperando la autorización en el navegador…", con explicación de la pantalla "Google no ha verificado esta app → Avanzado → Ir a …" que verá si no publicó la app. Botón cancelar. | — |
| 7. Listo | Muestra el email detectado, la ruta `~/.config/gdrive3/<email>/`, el aviso de seguridad de add.rs:54, y CTA a "Ver mis archivos". | — |

Manejo de errores específicos, con remedio en pantalla en vez de un volcado de error:
- `invalid_client` / `unauthorized_client` → "revisa que copiaste el Client ID/Secret del cliente Desktop correcto (paso 4)".
- `access_denied` → "tu email no está en la lista de test users, o la app no está publicada (paso 3)".
- Puerto 8085 ocupado → "cierra la otra instancia de gdrive" + botón reintentar.
- Timeout (>5 min sin redirect) → cancela limpiamente y ofrece reintentar.

## Fase 5 — GitHub Actions

Se **conservan** las tres workflows actuales del CLI (`release_{linux,macos,windows}.yaml`) y se añaden dos nuevas:

**`.github/workflows/ci.yaml`** (falta hoy por completo) — `on: [push, pull_request]`:
- job `rust`: matriz ubuntu/macos/windows → `dtolnay/rust-toolchain@stable`, `Swatinem/rust-cache@v2`, `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo build --workspace`.
- job `ui`: `actions/setup-node@v4` (node 20, cache npm) → `npm ci`, `npm run typecheck`, `npm run build` en `gdrive-ui/`.

**`.github/workflows/release_ui.yaml`** — `on: release: types: [created]` (mismo disparador que el resto) + `workflow_dispatch`:
```yaml
strategy:
  matrix:
    include:
      - { platform: ubuntu-22.04,  args: "" }                        # .deb + .AppImage + .rpm
      - { platform: macos-14,      args: "--target aarch64-apple-darwin" }
      - { platform: macos-13,      args: "--target x86_64-apple-darwin" }
      - { platform: windows-latest, args: "" }                       # .msi + .exe (NSIS)
```
Pasos: checkout → setup-node 20 → rust-toolchain stable (+ `targets` en macOS) → `Swatinem/rust-cache@v2` → **sólo en ubuntu**: `apt-get install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf build-essential curl wget file libssl-dev libayatana-appindicator3-dev` (4.1, requerido por Tauri v2) → `npm ci` en `gdrive-ui` → `tauri-apps/tauri-action@v0` con `projectPath: gdrive-ui`, `tagName`/`releaseId` tomados del release que dispara, `args: ${{ matrix.args }}`.
- `permissions: contents: write`; `GITHUB_TOKEN` estándar.
- Firma/notarización: se dejan **preparados pero opcionales** los secretos `APPLE_CERTIFICATE`, `APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID`, `WINDOWS_CERTIFICATE`, y `TAURI_SIGNING_PRIVATE_KEY` (updater). Si no están definidos el build sigue funcionando sin firmar, como hoy hace el CLI (documentado en el README).

De paso, en las workflows existentes: subir `actions/checkout@v3`→`v4` y sustituir el `actions/upload-release-asset@v1` deprecado por `softprops/action-gh-release@v2`, ya que el nuevo job usará ese release.

## Fase 6 — Documentación

- `README.md`: nueva sección "Desktop app" con capturas del wizard, tabla de descargas por plataforma y nota de binarios sin firmar (Gatekeeper/SmartScreen).
- `docs/create_google_api_credentials.md`: añadir arriba "Si usas la app de escritorio, el asistente te guía por estos pasos automáticamente".
- `gdrive-ui/README.md`: cómo desarrollar (`npm run tauri dev`), requisitos por SO.

---

## Ficheros críticos

| Fichero | Cambio |
|---|---|
| [Cargo.toml](Cargo.toml) | workspace + secciones `[lib]`/`[[bin]]` |
| `src/lib.rs` | **nuevo**, expone los módulos |
| [src/main.rs](src/main.rs) | pasa a bin fino sobre `gdrive::` |
| [src/hub.rs](src/hub.rs#L93-L112) | delegate parametrizable (`new_with_presenter`) |
| [src/account/add.rs](src/account/add.rs#L9-L61) | extraer `add_with_secret(secret, presenter)` |
| [src/app_config.rs](src/app_config.rs#L192-L196) | parser de `client_secret_*.json` |
| `gdrive-ui/src-tauri/**` | **nuevo**, comandos Tauri |
| `gdrive-ui/src/wizard/**` | **nuevo**, asistente de 8 pantallas |
| `.github/workflows/ci.yaml`, `release_ui.yaml` | **nuevos** |
| `.github/workflows/release_{linux,macos,windows}.yaml` | actualizar actions deprecadas |

## Verificación

1. **No regresión del CLI**: `cargo build --release && ./target/release/gdrive account list && ./target/release/gdrive files list --max 5` — salida idéntica a la de antes del refactor.
2. **Unit tests nuevos** (hoy no hay ninguno): parser de `client_secret_*.json` (variantes `installed`/`web`/JSON inválido) y validador del formato de Client ID.
3. **Wizard end-to-end en local**: `cd gdrive-ui && npm run tauri dev`, recorrer los 8 pasos con una cuenta Google real, importando el JSON descargado; confirmar que aparece `~/.config/gdrive3/<email>/{secret.json,tokens.json}` con `secret.json` en modo 0600, y que `./target/release/gdrive files list` **ya funciona con la cuenta creada desde la GUI** (prueba clave de interoperabilidad).
4. **Casos de error**: introducir un Client Secret erróneo → mensaje de remedio, no traza; ocupar el 8085 con `nc -l 8085` → aviso previo; cancelar a mitad → el puerto se libera y se puede reintentar.
5. **CI**: abrir un PR y comprobar que `ci.yaml` pasa en las tres plataformas.
6. **Release**: crear un pre-release de prueba y verificar que se adjuntan `.deb`/`.AppImage`, `.dmg` arm64 y x64, y `.msi`/`.exe`; instalar el `.deb` y el `.msi` y arrancar la app.
