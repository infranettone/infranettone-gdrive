import { useEffect, useState, type ReactNode } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useI18n } from "../lib/i18n";
import type { UiError } from "../lib/api";

/** Renders a UiError with its remedy, which is the whole point of having one. */
export function ErrorBox({ error, onRetry }: { error: UiError; onRetry?: () => void }) {
  const { t } = useI18n();

  return (
    <div className="alert error" role="alert">
      <div>{error.message}</div>
      {error.hint && (
        <div className="hint">
          <strong>{t.common.hint}</strong>
          {error.hint}
        </div>
      )}
      {onRetry && (
        <button className="btn small" style={{ marginTop: 8 }} onClick={onRetry}>
          {t.common.retry}
        </button>
      )}
    </div>
  );
}

export function Warning({ children }: { children: ReactNode }) {
  return (
    <div className="alert warn" role="note">
      {children}
    </div>
  );
}

/** Opens an external url through the opener plugin (never in the webview). */
export function ExternalButton({
  url,
  children,
  className = "btn primary",
}: {
  url: string;
  children: ReactNode;
  className?: string;
}) {
  return (
    <button className={className} onClick={() => void openUrl(url)}>
      {children}
    </button>
  );
}

/** Copy-to-clipboard button that confirms, then reverts after a moment. */
export function CopyButton({ value, label }: { value: string; label?: string }) {
  const { t } = useI18n();
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (!copied) return;
    const id = setTimeout(() => setCopied(false), 1600);
    return () => clearTimeout(id);
  }, [copied]);

  return (
    <button
      className="btn small"
      onClick={async () => {
        await navigator.clipboard.writeText(value);
        setCopied(true);
      }}
    >
      {copied ? t.common.copied : (label ?? t.common.copy)}
    </button>
  );
}

export function Spinner() {
  return <span className="spinner" aria-hidden="true" />;
}

export function Loading() {
  const { t } = useI18n();
  return (
    <div className="waiting">
      <Spinner />
      <span>{t.common.loading}</span>
    </div>
  );
}
