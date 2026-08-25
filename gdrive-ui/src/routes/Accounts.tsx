import { useState } from "react";
import { confirm, open } from "@tauri-apps/plugin-dialog";
import {
  accountImport,
  accountExport,
  accountRemove,
  accountSwitch,
  toUiError,
  type AccountSummary,
  type UiError,
} from "../lib/api";
import { ErrorBox, Loading } from "../components/common";
import { useI18n } from "../lib/i18n";

export function Accounts({
  accounts,
  loading,
  onRefresh,
  onAdd,
}: {
  accounts: AccountSummary[];
  loading: boolean;
  onRefresh: () => void;
  onAdd: () => void;
}) {
  const { t } = useI18n();
  const [error, setError] = useState<UiError | null>(null);

  async function run(action: () => Promise<unknown>) {
    setError(null);
    try {
      await action();
      onRefresh();
    } catch (err) {
      setError(toUiError(err));
    }
  }

  async function importAccount() {
    const selected = await open({
      multiple: false,
      filters: [{ name: "gdrive account archive", extensions: ["tar"] }],
    });
    if (typeof selected === "string") await run(() => accountImport(selected));
  }

  return (
    <>
      <div className="page-head">
        <div>
          <h1>{t.accounts.title}</h1>
          <p className="subtitle">{t.accounts.subtitle}</p>
        </div>
        <div className="row">
          <button className="btn" onClick={() => void importAccount()}>
            {t.accounts.import}
          </button>
          <button className="btn primary" onClick={onAdd}>
            {t.accounts.add}
          </button>
        </div>
      </div>

      {error && <ErrorBox error={error} />}

      {loading ? (
        <Loading />
      ) : accounts.length === 0 ? (
        <div className="card empty">
          <p>{t.accounts.empty}</p>
          <p>{t.accounts.emptyCta}</p>
          <button className="btn primary" onClick={onAdd}>
            {t.accounts.add}
          </button>
        </div>
      ) : (
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>{t.common.name}</th>
                <th>{t.common.actions}</th>
              </tr>
            </thead>
            <tbody>
              {accounts.map((account) => (
                <tr key={account.name}>
                  <td className="name">
                    {account.name}{" "}
                    {account.isCurrent && <span className="badge current">{t.accounts.current}</span>}
                    <div className="mono" style={{ color: "var(--text-dim)" }}>
                      {account.path}
                    </div>
                  </td>
                  <td>
                    <div className="row">
                      {!account.isCurrent && (
                        <button
                          className="btn small"
                          onClick={() => void run(() => accountSwitch(account.name))}
                        >
                          {t.accounts.use}
                        </button>
                      )}
                      <button
                        className="btn small"
                        onClick={() => void run(() => accountExport(account.name))}
                      >
                        {t.accounts.export}
                      </button>
                      <button
                        className="btn small danger"
                        onClick={() =>
                          void (async () => {
                            if (await confirm(t.accounts.removeConfirm(account.name))) {
                              await run(() => accountRemove(account.name));
                            }
                          })()
                        }
                      >
                        {t.common.remove}
                      </button>
                    </div>
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
