import { useCallback, useEffect, useState } from "react";
import { confirm, open, save } from "@tauri-apps/plugin-dialog";
import {
  filesDelete,
  filesDownload,
  filesInfo,
  filesList,
  filesMkdir,
  filesRename,
  filesUpload,
  toUiError,
  type FieldRow,
  type FileRow,
  type UiError,
} from "../lib/api";
import { ErrorBox, Loading } from "../components/common";
import { useI18n } from "../lib/i18n";

interface Crumb {
  id: string | null;
  name: string;
}

const TYPE_ICON: Record<FileRow["fileType"], string> = {
  folder: "📁",
  regular: "📄",
  shortcut: "🔗",
  document: "📝",
};

export function Files({ hasAccount }: { hasAccount: boolean }) {
  const { t } = useI18n();

  const [crumbs, setCrumbs] = useState<Crumb[]>([{ id: null, name: t.files.root }]);
  const [rows, setRows] = useState<FileRow[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<UiError | null>(null);
  const [busy, setBusy] = useState(false);
  const [max, setMax] = useState(100);
  const [search, setSearch] = useState("");
  const [details, setDetails] = useState<{ name: string; fields: FieldRow[] } | null>(null);

  const current = crumbs[crumbs.length - 1];

  const load = useCallback(async () => {
    if (!hasAccount) return;
    setLoading(true);
    setError(null);
    try {
      setRows(await filesList({ folderId: current.id, max }));
    } catch (err) {
      setError(toUiError(err));
    } finally {
      setLoading(false);
    }
  }, [current.id, max, hasAccount]);

  useEffect(() => {
    void load();
  }, [load]);

  async function run(action: () => Promise<unknown>) {
    setBusy(true);
    setError(null);
    try {
      await action();
      await load();
    } catch (err) {
      setError(toUiError(err));
    } finally {
      setBusy(false);
    }
  }

  const parents = current.id ? [current.id] : undefined;

  async function upload(directory: boolean) {
    const selected = await open({ multiple: false, directory });
    if (typeof selected === "string") {
      await run(() => filesUpload({ filePath: selected, parents, recursive: directory }));
    }
  }

  async function download(file: FileRow) {
    if (file.fileType === "folder") {
      const dir = await open({ multiple: false, directory: true });
      if (typeof dir !== "string") return;
      await run(() =>
        filesDownload({
          fileId: file.id,
          destination: dir,
          overwrite: true,
          recursive: true,
          followShortcuts: true,
        }),
      );
      return;
    }

    const target = await save({ defaultPath: file.name });
    if (typeof target !== "string") return;
    await run(() =>
      filesDownload({
        fileId: file.id,
        destination: target,
        overwrite: true,
        recursive: false,
        followShortcuts: true,
      }),
    );
  }

  if (!hasAccount) {
    return (
      <div className="card empty">
        <p>{t.accounts.empty}</p>
        <p>{t.accounts.emptyCta}</p>
      </div>
    );
  }

  const visible = search.trim()
    ? rows.filter((row) => row.name.toLowerCase().includes(search.trim().toLowerCase()))
    : rows;

  return (
    <>
      <div className="page-head">
        <div>
          <h1>{t.files.title}</h1>
          <nav className="row subtitle" aria-label="breadcrumb">
            {crumbs.map((crumb, index) => (
              <span key={`${crumb.id}-${index}`}>
                {index > 0 && <span aria-hidden="true"> / </span>}
                <button
                  className="btn link"
                  onClick={() => setCrumbs((c) => c.slice(0, index + 1))}
                  disabled={index === crumbs.length - 1}
                >
                  {crumb.name}
                </button>
              </span>
            ))}
          </nav>
        </div>
        <div className="row">
          <button className="btn" disabled={busy} onClick={() => void upload(false)}>
            {t.files.upload}
          </button>
          <button className="btn" disabled={busy} onClick={() => void upload(true)}>
            {t.files.uploadFolder}
          </button>
          <button
            className="btn"
            disabled={busy}
            onClick={() =>
              void (async () => {
                const name = window.prompt(t.files.mkdirPrompt);
                if (name?.trim()) await run(() => filesMkdir(name.trim(), parents));
              })()
            }
          >
            {t.files.mkdir}
          </button>
          <button className="btn" disabled={busy} onClick={() => void load()}>
            {t.common.refresh}
          </button>
        </div>
      </div>

      <div className="row" style={{ marginBottom: 14 }}>
        <input
          type="text"
          placeholder={t.files.search}
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          style={{ maxWidth: 280 }}
        />
        <label htmlFor="max-files" style={{ color: "var(--text-dim)" }}>
          {t.files.max}
        </label>
        <input
          id="max-files"
          type="number"
          min={1}
          max={1000}
          value={max}
          onChange={(e) => setMax(Math.max(1, Number(e.target.value) || 1))}
          style={{ width: 90 }}
        />
        {busy && <span className="spinner" aria-hidden="true" />}
      </div>

      {error && <ErrorBox error={error} onRetry={() => void load()} />}

      {details && (
        <div className="card" style={{ marginBottom: 14 }}>
          <div className="page-head" style={{ marginBottom: 8 }}>
            <h2>{details.name}</h2>
            <button className="btn small" onClick={() => setDetails(null)}>
              {t.common.close}
            </button>
          </div>
          <dl style={{ margin: 0 }}>
            {details.fields.map((field) => (
              <div key={field.name} className="row" style={{ gap: 6 }}>
                <dt style={{ color: "var(--text-dim)", minWidth: 130 }}>{field.name}</dt>
                <dd className="mono" style={{ margin: 0, overflowWrap: "anywhere" }}>
                  {field.value}
                </dd>
              </div>
            ))}
          </dl>
        </div>
      )}

      {loading ? (
        <Loading />
      ) : visible.length === 0 ? (
        <div className="card empty">{t.common.noResults}</div>
      ) : (
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>{t.common.name}</th>
                <th>{t.common.size}</th>
                <th>{t.common.created}</th>
                <th>{t.common.actions}</th>
              </tr>
            </thead>
            <tbody>
              {visible.map((file) => (
                <tr key={file.id}>
                  <td className="name">
                    <span aria-hidden="true">{TYPE_ICON[file.fileType]} </span>
                    {file.fileType === "folder" ? (
                      <button
                        className="btn link"
                        onClick={() => setCrumbs((c) => [...c, { id: file.id, name: file.name }])}
                      >
                        {file.name}
                      </button>
                    ) : (
                      file.name
                    )}
                    <div className="mono" style={{ color: "var(--text-dim)" }}>
                      {file.id}
                    </div>
                  </td>
                  <td>{file.sizeHuman ?? "—"}</td>
                  <td>{file.createdTime ?? "—"}</td>
                  <td>
                    <div className="row">
                      <button className="btn small" disabled={busy} onClick={() => void download(file)}>
                        {t.files.download}
                      </button>
                      <button
                        className="btn small"
                        disabled={busy}
                        onClick={() =>
                          void (async () => {
                            const name = window.prompt(t.files.renamePrompt, file.name);
                            if (name?.trim() && name !== file.name) {
                              await run(() => filesRename(file.id, name.trim()));
                            }
                          })()
                        }
                      >
                        {t.files.rename}
                      </button>
                      <button
                        className="btn small"
                        disabled={busy}
                        onClick={() =>
                          void (async () => {
                            setError(null);
                            try {
                              setDetails({ name: file.name, fields: await filesInfo(file.id) });
                            } catch (err) {
                              setError(toUiError(err));
                            }
                          })()
                        }
                      >
                        {t.files.info}
                      </button>
                      <button
                        className="btn small danger"
                        disabled={busy}
                        onClick={() =>
                          void (async () => {
                            if (await confirm(t.files.deleteConfirm(file.name))) {
                              await run(() => filesDelete(file.id, file.fileType === "folder"));
                            }
                          })()
                        }
                      >
                        {t.files.del}
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
