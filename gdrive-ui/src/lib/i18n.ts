import { createContext, useContext } from "react";

export type Lang = "es" | "en";

/**
 * Every user-visible string, in both languages.
 *
 * The wizard is the reason this exists: it is the part of the app that has to
 * explain an unfamiliar Google Cloud flow, and it is much less useful in a
 * language the reader does not speak.
 */
const es = {
  appName: "gdrive",
  nav: {
    accounts: "Cuentas",
    files: "Archivos",
    drives: "Unidades",
    permissions: "Permisos",
  },
  common: {
    cancel: "Cancelar",
    back: "Atrás",
    next: "Siguiente",
    retry: "Reintentar",
    close: "Cerrar",
    copy: "Copiar",
    copied: "Copiado",
    loading: "Cargando…",
    refresh: "Actualizar",
    openInBrowser: "Abrir en el navegador",
    done: "Hecho",
    remove: "Eliminar",
    confirm: "Confirmar",
    name: "Nombre",
    id: "Id",
    type: "Tipo",
    size: "Tamaño",
    created: "Creado",
    actions: "Acciones",
    noResults: "No hay nada que mostrar",
    hint: "Qué hacer",
  },
  accounts: {
    title: "Cuentas de Google",
    subtitle:
      "Las cuentas se guardan en la misma carpeta que usa el CLI de gdrive, así que sirven para ambos.",
    add: "Añadir cuenta de Google",
    current: "Activa",
    use: "Usar",
    export: "Exportar",
    import: "Importar cuenta…",
    empty: "Todavía no has añadido ninguna cuenta.",
    emptyCta:
      "El asistente te guía paso a paso, incluida la creación de las credenciales en Google Cloud.",
    removeConfirm: (name: string) =>
      `¿Eliminar la cuenta ${name}? Se borrarán sus credenciales y tokens de este equipo.`,
  },
  files: {
    title: "Archivos",
    upload: "Subir…",
    uploadFolder: "Subir carpeta…",
    mkdir: "Nueva carpeta",
    mkdirPrompt: "Nombre de la carpeta",
    download: "Descargar",
    rename: "Renombrar",
    renamePrompt: "Nuevo nombre",
    del: "Eliminar",
    info: "Detalles",
    permissions: "Permisos",
    root: "Mi unidad",
    search: "Buscar por nombre…",
    deleteConfirm: (name: string) => `¿Eliminar "${name}"?`,
    max: "Máximo",
  },
  drives: {
    title: "Unidades compartidas",
    empty: "Esta cuenta no tiene unidades compartidas.",
  },
  permissions: {
    title: "Permisos",
    fileId: "Id del archivo",
    load: "Cargar permisos",
    share: "Compartir",
    role: "Rol",
    kind: "Tipo",
    email: "Email",
    domain: "Dominio",
    discoverable: "Visible en búsquedas",
    revoke: "Revocar",
    revokeAll: "Revocar todo excepto el propietario",
  },
  wizard: {
    title: "Añadir una cuenta de Google",
    stepOf: (a: number, b: number) => `Paso ${a} de ${b}`,
    markDone: "Ya lo he hecho",
    skipToCredentials: "Ya tengo credenciales",
    startOver: "Empezar de cero",
    projectIdLabel: "Project ID (opcional, personaliza los enlaces siguientes)",
    projectIdPlaceholder: "mi-proyecto-123456",
    steps: {
      welcome: {
        title: "Antes de empezar",
        lead: "gdrive accede a tu Drive con credenciales OAuth que creas tú, en tu propia cuenta de Google Cloud.",
        body: [
          "Esta aplicación no lleva credenciales incrustadas: nadie más que tú tiene acceso a tus archivos, y no dependes de una cuota compartida.",
          "El precio es que hay que crearlas una vez. Son unos 5 minutos, y este asistente abre cada pantalla exacta que necesitas.",
          "Al final volverás aquí con un Client ID y un Client Secret, y autorizarás el acceso desde tu navegador.",
        ],
      },
      project: {
        title: "1. Crea un proyecto en Google Cloud",
        lead: "Un proyecto es el contenedor de tus credenciales. Si ya tienes uno, puedes reutilizarlo.",
        body: [
          "Pulsa el botón para abrir el creador de proyectos, ponle un nombre (por ejemplo «gdrive») y créalo.",
          "Cuando esté listo, copia aquí abajo su Project ID: lo usaremos para llevarte directamente a la pantalla correcta en los pasos siguientes.",
        ],
      },
      enableApi: {
        title: "2. Habilita la API de Google Drive",
        lead: "Sin esto, las peticiones se rechazan aunque las credenciales sean correctas.",
        body: [
          "Se abrirá la ficha de «Google Drive API» en el marketplace.",
          "Comprueba arriba que esté seleccionado tu proyecto y pulsa «Enable» (o «Habilitar»). Si el botón ya dice «Manage», es que estaba habilitada.",
        ],
      },
      consent: {
        title: "3. Configura la pantalla de consentimiento",
        lead: "Es la pantalla que verás al autorizar. Google exige configurarla antes de dejarte crear credenciales.",
        body: [
          "Elige el tipo de usuario «External» (el tipo «Internal» sólo existe si tienes Google Workspace).",
          "Rellena «App name», «User support email» y «Developer contact information». Deja el resto vacío.",
          "En «Scopes», pulsa «Add or remove scopes» y añade estos dos permisos (puedes copiarlos con el botón de abajo):",
          "En «Test users», añade tu propia dirección de Gmail. Sin esto Google rechazará la autorización.",
        ],
        scopesLabel: "Permisos que hay que añadir",
        publishWarning:
          "Importante: al terminar, vuelve a «OAuth consent screen» y pulsa «Publish app» y luego «Confirm». Si dejas la app en modo «Testing», Google caducará tu acceso a los 7 días y tendrás que repetir todo esto.",
      },
      client: {
        title: "4. Crea el ID de cliente OAuth",
        lead: "Este es el paso que produce el Client ID y el Client Secret.",
        body: [
          "En «Credentials», pulsa «Create credentials» → «OAuth client ID».",
          "En «Application type» elige «Desktop app». Es imprescindible: otros tipos no permiten el redireccionamiento local que usa gdrive.",
          "Ponle un nombre y pulsa «Create».",
          "Google te mostrará el Client ID y el Client Secret, y te ofrecerá descargar un fichero JSON. Descárgalo: en el paso siguiente puedes soltarlo aquí y se rellena todo solo.",
        ],
      },
      credentials: {
        title: "5. Introduce tus credenciales",
        lead: "Suelta aquí el JSON que descargaste, o pega los valores a mano.",
        dropzone: "Arrastra aquí el fichero client_secret_….json",
        dropzoneOr: "o",
        chooseFile: "Elegir fichero…",
        manual: "Introducir a mano",
        clientId: "Client ID",
        clientSecret: "Client Secret",
        show: "Mostrar",
        hide: "Ocultar",
        loadedFrom: (name: string) => `Credenciales cargadas de ${name}`,
        badClientId:
          "Esto no parece un Client ID de Google. Debería terminar en .apps.googleusercontent.com — comprueba que no lo has confundido con el Client Secret.",
        emptySecret: "Falta el Client Secret.",
        portBusy: (port: number) =>
          `El puerto ${port} está ocupado. Google necesita redirigir ahí al terminar la autorización. Cierra la otra instancia de gdrive y vuelve a comprobarlo.`,
        portCheck: "Comprobar de nuevo",
        portOk: (port: number) => `El puerto ${port} está libre.`,
        authorize: "Autorizar en Google",
      },
      authorize: {
        title: "6. Autoriza el acceso",
        waiting: "Esperando a que completes la autorización en el navegador…",
        opening: "Abriendo tu navegador…",
        urlLabel: "Si no se ha abierto solo, entra en esta dirección:",
        unverified:
          "Si ves «Google no ha verificado esta aplicación», es normal: la app es tuya y nadie la ha revisado. Pulsa «Avanzado» y luego «Ir a … (no seguro)».",
        timeout:
          "Han pasado más de 5 minutos sin respuesta. Se ha cancelado la autorización para liberar el puerto.",
      },
      done: {
        title: "¡Listo!",
        loggedIn: (email: string) => `Has iniciado sesión como ${email}.`,
        savedIn: (path: string) => `Credenciales guardadas en ${path}`,
        keepSafe:
          "Guárdalas bien: quien tenga acceso a esos ficheros tendrá también acceso a tu Google Drive.",
        alsoCli:
          "El CLI de gdrive ya puede usar esta cuenta, sin configurar nada más.",
        goToFiles: "Ver mis archivos",
      },
    },
  },
};

/** `Strings` is derived from the Spanish dictionary, which is the reference. */
export type Strings = typeof es;

const en: Strings = {
  appName: "gdrive",
  nav: {
    accounts: "Accounts",
    files: "Files",
    drives: "Drives",
    permissions: "Permissions",
  },
  common: {
    cancel: "Cancel",
    back: "Back",
    next: "Next",
    retry: "Retry",
    close: "Close",
    copy: "Copy",
    copied: "Copied",
    loading: "Loading…",
    refresh: "Refresh",
    openInBrowser: "Open in browser",
    done: "Done",
    remove: "Remove",
    confirm: "Confirm",
    name: "Name",
    id: "Id",
    type: "Type",
    size: "Size",
    created: "Created",
    actions: "Actions",
    noResults: "Nothing to show",
    hint: "What to do",
  },
  accounts: {
    title: "Google accounts",
    subtitle:
      "Accounts are stored in the same folder the gdrive CLI uses, so they work for both.",
    add: "Add Google account",
    current: "Active",
    use: "Use",
    export: "Export",
    import: "Import account…",
    empty: "You haven't added any account yet.",
    emptyCta:
      "The wizard walks you through it, including creating the credentials in Google Cloud.",
    removeConfirm: (name: string) =>
      `Remove account ${name}? Its credentials and tokens will be deleted from this machine.`,
  },
  files: {
    title: "Files",
    upload: "Upload…",
    uploadFolder: "Upload folder…",
    mkdir: "New folder",
    mkdirPrompt: "Folder name",
    download: "Download",
    rename: "Rename",
    renamePrompt: "New name",
    del: "Delete",
    info: "Details",
    permissions: "Permissions",
    root: "My Drive",
    search: "Search by name…",
    deleteConfirm: (name: string) => `Delete "${name}"?`,
    max: "Max",
  },
  drives: {
    title: "Shared drives",
    empty: "This account has no shared drives.",
  },
  permissions: {
    title: "Permissions",
    fileId: "File id",
    load: "Load permissions",
    share: "Share",
    role: "Role",
    kind: "Type",
    email: "Email",
    domain: "Domain",
    discoverable: "Discoverable in search",
    revoke: "Revoke",
    revokeAll: "Revoke all except owner",
  },
  wizard: {
    title: "Add a Google account",
    stepOf: (a: number, b: number) => `Step ${a} of ${b}`,
    markDone: "I've done this",
    skipToCredentials: "I already have credentials",
    startOver: "Start over",
    projectIdLabel: "Project ID (optional, tailors the links below)",
    projectIdPlaceholder: "my-project-123456",
    steps: {
      welcome: {
        title: "Before you start",
        lead: "gdrive reaches your Drive with OAuth credentials that you create, in your own Google Cloud account.",
        body: [
          "This app ships with no embedded credentials: nobody but you can reach your files, and you don't share a quota with anyone.",
          "The price is creating them once. It takes about 5 minutes, and this wizard opens each exact screen you need.",
          "You'll come back here with a Client ID and a Client Secret, and authorize access from your browser.",
        ],
      },
      project: {
        title: "1. Create a Google Cloud project",
        lead: "A project holds your credentials. If you already have one, reuse it.",
        body: [
          'Open the project creator, give it a name ("gdrive" works) and create it.',
          "Once ready, paste its Project ID below: we'll use it to take you straight to the right screen in the next steps.",
        ],
      },
      enableApi: {
        title: "2. Enable the Google Drive API",
        lead: "Without this, requests are rejected even with valid credentials.",
        body: [
          'The "Google Drive API" marketplace page will open.',
          'Check your project is selected at the top and press "Enable". If the button already says "Manage", it was enabled already.',
        ],
      },
      consent: {
        title: "3. Configure the consent screen",
        lead: "This is the screen you'll see when authorizing. Google requires it before letting you create credentials.",
        body: [
          'Pick user type "External" ("Internal" only exists with Google Workspace).',
          'Fill in "App name", "User support email" and "Developer contact information". Leave the rest empty.',
          'Under "Scopes", press "Add or remove scopes" and add these two (copy them with the button below):',
          'Under "Test users", add your own Gmail address. Without it Google will refuse the authorization.',
        ],
        scopesLabel: "Scopes to add",
        publishWarning:
          'Important: when you\'re done, go back to "OAuth consent screen" and press "Publish app", then "Confirm". Leaving the app in "Testing" makes Google expire your access after 7 days, and you\'d have to redo all of this.',
      },
      client: {
        title: "4. Create the OAuth client ID",
        lead: "This is the step that produces the Client ID and Client Secret.",
        body: [
          'Under "Credentials", press "Create credentials" → "OAuth client ID".',
          'For "Application type" pick "Desktop app". This matters: other types don\'t allow the local redirect gdrive uses.',
          'Give it a name and press "Create".',
          "Google shows the Client ID and Client Secret, and offers a JSON file to download. Download it: in the next step you can drop it here and everything fills in for you.",
        ],
      },
      credentials: {
        title: "5. Enter your credentials",
        lead: "Drop the JSON you downloaded here, or paste the values by hand.",
        dropzone: "Drag the client_secret_….json file here",
        dropzoneOr: "or",
        chooseFile: "Choose file…",
        manual: "Enter by hand",
        clientId: "Client ID",
        clientSecret: "Client Secret",
        show: "Show",
        hide: "Hide",
        loadedFrom: (name: string) => `Credentials loaded from ${name}`,
        badClientId:
          "That doesn't look like a Google Client ID. It should end in .apps.googleusercontent.com — check you didn't swap it with the Client Secret.",
        emptySecret: "The Client Secret is missing.",
        portBusy: (port: number) =>
          `Port ${port} is busy. Google needs to redirect there when authorization finishes. Close the other gdrive instance and check again.`,
        portCheck: "Check again",
        portOk: (port: number) => `Port ${port} is free.`,
        authorize: "Authorize with Google",
      },
      authorize: {
        title: "6. Authorize access",
        waiting: "Waiting for you to finish the authorization in the browser…",
        opening: "Opening your browser…",
        urlLabel: "If it didn't open by itself, go to this address:",
        unverified:
          'If you see "Google hasn\'t verified this app", that\'s expected: the app is yours and nobody reviewed it. Press "Advanced", then "Go to … (unsafe)".',
        timeout:
          "More than 5 minutes passed with no response. The authorization was cancelled to free the port.",
      },
      done: {
        title: "All set!",
        loggedIn: (email: string) => `You're signed in as ${email}.`,
        savedIn: (path: string) => `Credentials saved in ${path}`,
        keepSafe:
          "Keep them safe: anyone with access to those files also has access to your Google Drive.",
        alsoCli:
          "The gdrive CLI can already use this account, with no extra setup.",
        goToFiles: "See my files",
      },
    },
  },
};

export const strings: Record<Lang, Strings> = { es, en };

export const LangContext = createContext<{
  lang: Lang;
  t: Strings;
  setLang: (l: Lang) => void;
}>({
  lang: "es",
  t: strings.es,
  setLang: () => {},
});

export const useI18n = () => useContext(LangContext);

/** The OAuth scopes the user must add on the consent screen (step 3). */
export const REQUIRED_SCOPES = [
  "https://www.googleapis.com/auth/drive",
  "https://www.googleapis.com/auth/drive.metadata.readonly",
];

/** Deep links into Google Cloud Console, narrowed to the project when known. */
export function consoleUrls(projectId: string) {
  const suffix = projectId.trim()
    ? `?project=${encodeURIComponent(projectId.trim())}`
    : "";

  return {
    createProject: "https://console.cloud.google.com/projectcreate",
    enableDriveApi: `https://console.cloud.google.com/apis/library/drive.googleapis.com${suffix}`,
    consentScreen: `https://console.cloud.google.com/apis/credentials/consent${suffix}`,
    createClient: `https://console.cloud.google.com/apis/credentials/oauthclient${suffix}`,
    credentials: `https://console.cloud.google.com/apis/credentials${suffix}`,
  };
}
