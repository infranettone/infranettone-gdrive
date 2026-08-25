import { useCallback, useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import {
  parseCredentialsFile,
  redirectPort,
  redirectPortFree,
  toUiError,
  validateClientId,
  type UiError,
} from "../../lib/api";
import { ErrorBox } from "../../components/common";
import { useI18n } from "../../lib/i18n";

export interface Credentials {
  clientId: string;
  clientSecret: string;
}

/**
 * Step 5: collect the Client ID and Client Secret, either from the json Google
 * downloads or typed by hand, and refuse to move on with values that clearly
 * cannot work.
 */
export function CredentialsStep({
  value,
  onChange,
  onSubmit,
}: {
  value: Credentials;
  onChange: (next: Credentials) => void;
  onSubmit: () => void;
}) {
  const { t } = useI18n();
  const s = t.wizard.steps.credentials;

  const [showSecret, setShowSecret] = useState(false);
  const [dragOver, setDragOver] = useState(false);
  const [loadedFrom, setLoadedFrom] = useState<string | null>(null);
  const [error, setError] = useState<UiError | null>(null);
  const [clientIdOk, setClientIdOk] = useState(true);
  const [port, setPort] = useState(8085);
  const [portFree, setPortFree] = useState<boolean | null>(null);

  const checkPort = useCallback(async () => {
    setPortFree(await redirectPortFree());
  }, []);

  useEffect(() => {
    void redirectPort().then(setPort);
    void checkPort();
  }, [checkPort]);

  // Validate lazily: an empty field is "not yet wrong", not an error.
  useEffect(() => {
    const id = value.clientId.trim();
    if (!id) {
      setClientIdOk(true);
      return;
    }
    let live = true;
    void validateClientId(id).then((ok) => live && setClientIdOk(ok));
    return () => {
      live = false;
    };
  }, [value.clientId]);

  const loadFromPath = useCallback(
    async (path: string) => {
      setError(null);
      try {
        const secret = await parseCredentialsFile(path);
        onChange({ clientId: secret.clientId, clientSecret: secret.clientSecret });
        setLoadedFrom(path.split(/[/\\]/).pop() ?? path);
      } catch (err) {
        setError(toUiError(err));
      }
    },
    [onChange],
  );

  // Tauri delivers OS drag & drop as a webview event, not as a DOM drop event.
  useEffect(() => {
    const unlisten = getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === "over") {
        setDragOver(true);
      } else if (event.payload.type === "drop") {
        setDragOver(false);
        const path = event.payload.paths[0];
        if (path) void loadFromPath(path);
      } else {
        setDragOver(false);
      }
    });

    return () => {
      void unlisten.then((un) => un());
    };
  }, [loadFromPath]);

  async function chooseFile() {
    const selected = await open({
      multiple: false,
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (typeof selected === "string") await loadFromPath(selected);
  }

  const secretMissing = value.clientSecret.trim().length === 0;
  const canSubmit = clientIdOk && value.clientId.trim() !== "" && !secretMissing && portFree === true;

  return (
    <div className="wizard-panel">
      <h1>{s.title}</h1>
      <p className="lead">{s.lead}</p>

      {error && <ErrorBox error={error} />}

      <div
        className={dragOver ? "dropzone over" : "dropzone"}
        onDragOver={(e) => e.preventDefault()}
      >
        <div>{s.dropzone}</div>
        <div style={{ margin: "8px 0" }}>{s.dropzoneOr}</div>
        <button className="btn" onClick={() => void chooseFile()}>
          {s.chooseFile}
        </button>
      </div>

      {loadedFrom && (
        <p className="body" style={{ color: "var(--ok)" }}>
          {s.loadedFrom(loadedFrom)}
        </p>
      )}

      <div className="field">
        <label htmlFor="client-id">{s.clientId}</label>
        <input
          id="client-id"
          type="text"
          className="mono"
          spellCheck={false}
          autoComplete="off"
          value={value.clientId}
          placeholder="123456789012-abc.apps.googleusercontent.com"
          onChange={(e) => onChange({ ...value, clientId: e.target.value })}
        />
        {!clientIdOk && (
          <span style={{ color: "var(--danger)", fontSize: 12.5 }}>{s.badClientId}</span>
        )}
      </div>

      <div className="field">
        <label htmlFor="client-secret">{s.clientSecret}</label>
        <div className="row" style={{ flexWrap: "nowrap" }}>
          <input
            id="client-secret"
            type={showSecret ? "text" : "password"}
            className="mono"
            spellCheck={false}
            autoComplete="off"
            value={value.clientSecret}
            placeholder="GOCSPX-…"
            onChange={(e) => onChange({ ...value, clientSecret: e.target.value })}
          />
          <button className="btn small" onClick={() => setShowSecret((v) => !v)}>
            {showSecret ? s.hide : s.show}
          </button>
        </div>
      </div>

      {portFree === false && (
        <div className="alert warn">
          {s.portBusy(port)}
          <div>
            <button className="btn small" style={{ marginTop: 8 }} onClick={() => void checkPort()}>
              {s.portCheck}
            </button>
          </div>
        </div>
      )}

      <div className="wizard-foot">
        <span className="spacer" />
        <button className="btn primary" disabled={!canSubmit} onClick={onSubmit}>
          {s.authorize}
        </button>
      </div>
    </div>
  );
}
