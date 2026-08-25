import { useCallback, useEffect, useState } from "react";
import { drivesList, toUiError, type DriveRow, type UiError } from "../lib/api";
import { ErrorBox, Loading } from "../components/common";
import { useI18n } from "../lib/i18n";

export function Drives({ hasAccount }: { hasAccount: boolean }) {
  const { t } = useI18n();
  const [rows, setRows] = useState<DriveRow[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<UiError | null>(null);

  const load = useCallback(async () => {
    if (!hasAccount) return;
    setLoading(true);
    setError(null);
    try {
      setRows(await drivesList());
    } catch (err) {
      setError(toUiError(err));
    } finally {
      setLoading(false);
    }
  }, [hasAccount]);

  useEffect(() => {
    void load();
  }, [load]);

  if (!hasAccount) {
    return <div className="card empty">{t.accounts.empty}</div>;
  }

  return (
    <>
      <div className="page-head">
        <h1>{t.drives.title}</h1>
        <button className="btn" onClick={() => void load()}>
          {t.common.refresh}
        </button>
      </div>

      {error && <ErrorBox error={error} onRetry={() => void load()} />}

      {loading ? (
        <Loading />
      ) : rows.length === 0 ? (
        <div className="card empty">{t.drives.empty}</div>
      ) : (
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>{t.common.name}</th>
                <th>{t.common.id}</th>
              </tr>
            </thead>
            <tbody>
              {rows.map((drive) => (
                <tr key={drive.id}>
                  <td className="name">{drive.name}</td>
                  <td className="mono">{drive.id}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </>
  );
}
