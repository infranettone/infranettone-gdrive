import { useCallback, useEffect, useState } from "react";
import { confirm } from "@tauri-apps/plugin-dialog";
import {
  cliInstall,
  cliStatus,
  cliUninstall,
  toUiError,
  type CliStatus,
  type UiError,
} from "../lib/api";
import { CopyButton, ErrorBox, Loading, Warning } from "../components/common";
import { useI18n } from "../lib/i18n";

const isWindows = navigator.userAgent.includes("Windows");

export function Cli() {
  const { t } = useI18n();
  const [status, setStatus] = useState<CliStatus | null>(null);
  const [error, setError] = useState<UiError | null>(null);
  const [busy, setBusy] = useState(false);

  const refresh = useCallback(async () => {
    setError(null);
    try {
      setStatus(await cliStatus());
    } catch (err) {
      setError(toUiError(err));
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  async function run(action: () => Promise<CliStatus>) {
    setBusy(true);
    setError(null);
    try {
      setStatus(await action());
    } catch (err) {
      setError(toUiError(err));
    } finally {
      setBusy(false);
    }
  }

  async function uninstall() {
    if (await confirm(t.cli.uninstallConfirm)) await run(cliUninstall);
  }

  return (
    <>
      <div className="page-head">
        <div>
          <h1>{t.cli.title}</h1>
          <p className="subtitle">{t.cli.subtitle}</p>
        </div>
      </div>

      {error && <ErrorBox error={error} onRetry={() => void refresh()} />}

      {!status ? (
        <Loading />
      ) : (
        <div className="card">
          <p>
            <span className={status.installed ? "badge current" : "badge"}>
              {status.installed ? t.cli.installed : t.cli.notInstalled}
            </span>
            {status.version && <span className="mono"> {status.version}</span>}
          </p>

          <p className="mono" style={{ color: "var(--text-dim)" }}>
            {status.resolved
              ? t.cli.resolvedAt(status.resolved)
              : t.cli.willInstallTo(status.targetPath)}
          </p>

          {!status.bundled && <Warning>{t.cli.notBundled}</Warning>}

          {/* An installed copy nobody can reach is the one genuinely
              confusing outcome, so say exactly what to add to the PATH. */}
          {status.userCopy && !status.onPath && !status.resolved && (
            <Warning>
              <div>{t.cli.notOnPath(status.targetDir)}</div>
              <pre className="mono">
                {isWindows
                  ? t.cli.notOnPathWindows(status.targetDir)
                  : t.cli.notOnPathUnix(status.targetDir)}
              </pre>
              <CopyButton value={status.targetDir} />
            </Warning>
          )}

          {status.installed && status.resolved && (
            <>
              <p>{t.cli.tryIt}</p>
              <pre className="mono">gdrive account list</pre>
            </>
          )}

          <div className="row" style={{ marginTop: 12 }}>
            <button
              className="btn primary"
              disabled={busy || !status.bundled}
              onClick={() => void run(cliInstall)}
            >
              {status.installed ? t.cli.reinstall : t.cli.install}
            </button>

            {/* Only ever offer to remove our own copy: a `gdrive` that came
                from a package manager is not ours to delete. */}
            {status.userCopy && (
              <button className="btn danger" disabled={busy} onClick={() => void uninstall()}>
                {t.cli.uninstall}
              </button>
            )}

            <button className="btn" disabled={busy} onClick={() => void refresh()}>
              {t.common.refresh}
            </button>
          </div>
        </div>
      )}
    </>
  );
}
