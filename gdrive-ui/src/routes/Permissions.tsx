import { useState } from "react";
import {
  permissionsList,
  permissionsRevoke,
  permissionsShare,
  toUiError,
  type PermissionRow,
  type UiError,
} from "../lib/api";
import { ErrorBox } from "../components/common";
import { useI18n } from "../lib/i18n";

const ROLES = ["reader", "commenter", "writer", "fileOrganizer", "organizer", "owner"];
const TYPES = ["anyone", "user", "group", "domain"];

export function Permissions({ hasAccount }: { hasAccount: boolean }) {
  const { t } = useI18n();

  const [fileId, setFileId] = useState("");
  const [rows, setRows] = useState<PermissionRow[] | null>(null);
  const [error, setError] = useState<UiError | null>(null);
  const [busy, setBusy] = useState(false);

  const [role, setRole] = useState("reader");
  const [type, setType] = useState("anyone");
  const [email, setEmail] = useState("");
  const [domain, setDomain] = useState("");
  const [discoverable, setDiscoverable] = useState(false);

  async function run(action: () => Promise<unknown>, reload = true) {
    setBusy(true);
    setError(null);
    try {
      await action();
      if (reload) setRows(await permissionsList(fileId.trim()));
    } catch (err) {
      setError(toUiError(err));
    } finally {
      setBusy(false);
    }
  }

  if (!hasAccount) {
    return <div className="card empty">{t.accounts.empty}</div>;
  }

  const needsEmail = type === "user" || type === "group";
  const needsDomain = type === "domain";
  const canAct = fileId.trim() !== "";

  return (
    <>
      <div className="page-head">
        <h1>{t.permissions.title}</h1>
      </div>

      <div className="card" style={{ marginBottom: 16 }}>
        <div className="field">
          <label htmlFor="file-id">{t.permissions.fileId}</label>
          <input
            id="file-id"
            type="text"
            className="mono"
            spellCheck={false}
            value={fileId}
            onChange={(e) => setFileId(e.target.value)}
          />
        </div>
        <div className="row">
          <button
            className="btn"
            disabled={!canAct || busy}
            onClick={() => void run(async () => {}, true)}
          >
            {t.permissions.load}
          </button>
          <button
            className="btn"
            disabled={!canAct || busy}
            onClick={() => void run(() => permissionsRevoke(fileId.trim(), true))}
          >
            {t.permissions.revokeAll}
          </button>
          {busy && <span className="spinner" aria-hidden="true" />}
        </div>
      </div>

      {error && <ErrorBox error={error} />}

      <div className="card" style={{ marginBottom: 16 }}>
        <h2>{t.permissions.share}</h2>
        <div className="row" style={{ alignItems: "flex-end" }}>
          <div className="field" style={{ minWidth: 160, marginBottom: 0 }}>
            <label htmlFor="role">{t.permissions.role}</label>
            <select id="role" value={role} onChange={(e) => setRole(e.target.value)}>
              {ROLES.map((r) => (
                <option key={r} value={r}>
                  {r}
                </option>
              ))}
            </select>
          </div>
          <div className="field" style={{ minWidth: 160, marginBottom: 0 }}>
            <label htmlFor="type">{t.permissions.kind}</label>
            <select id="type" value={type} onChange={(e) => setType(e.target.value)}>
              {TYPES.map((k) => (
                <option key={k} value={k}>
                  {k}
                </option>
              ))}
            </select>
          </div>
          {needsEmail && (
            <div className="field" style={{ minWidth: 220, marginBottom: 0 }}>
              <label htmlFor="email">{t.permissions.email}</label>
              <input id="email" type="text" value={email} onChange={(e) => setEmail(e.target.value)} />
            </div>
          )}
          {needsDomain && (
            <div className="field" style={{ minWidth: 220, marginBottom: 0 }}>
              <label htmlFor="domain">{t.permissions.domain}</label>
              <input
                id="domain"
                type="text"
                value={domain}
                onChange={(e) => setDomain(e.target.value)}
              />
            </div>
          )}
          {(type === "anyone" || type === "domain") && (
            <label className="row" style={{ gap: 6 }}>
              <input
                type="checkbox"
                checked={discoverable}
                onChange={(e) => setDiscoverable(e.target.checked)}
                style={{ width: "auto" }}
              />
              {t.permissions.discoverable}
            </label>
          )}
          <button
            className="btn primary"
            disabled={
              !canAct || busy || (needsEmail && !email.trim()) || (needsDomain && !domain.trim())
            }
            onClick={() =>
              void run(() =>
                permissionsShare({
                  fileId: fileId.trim(),
                  role,
                  permissionType: type,
                  discoverable,
                  email: email.trim() || null,
                  domain: domain.trim() || null,
                }),
              )
            }
          >
            {t.permissions.share}
          </button>
        </div>
      </div>

      {rows && (
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>{t.permissions.role}</th>
                <th>{t.permissions.kind}</th>
                <th>{t.permissions.email}</th>
                <th>{t.permissions.domain}</th>
                <th>{t.common.actions}</th>
              </tr>
            </thead>
            <tbody>
              {rows.length === 0 && (
                <tr>
                  <td colSpan={5} className="empty">
                    {t.common.noResults}
                  </td>
                </tr>
              )}
              {rows.map((p) => (
                <tr key={p.id}>
                  <td>{p.role}</td>
                  <td>{p.type}</td>
                  <td>{p.emailAddress ?? "—"}</td>
                  <td>{p.domain ?? "—"}</td>
                  <td>
                    <button
                      className="btn small danger"
                      disabled={busy}
                      onClick={() => void run(() => permissionsRevoke(fileId.trim(), false, p.id))}
                    >
                      {t.permissions.revoke}
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </>
  );
}
