import { useEffect, useRef, useState } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
  accountAddCancel,
  accountAddStart,
  onOauthEvents,
  toUiError,
  type AddedAccount,
  type UiError,
} from "../../lib/api";
import { CopyButton, ErrorBox, Spinner, Warning } from "../../components/common";
import { useI18n } from "../../lib/i18n";
import type { Credentials } from "./CredentialsStep";

/** Give up after this long so a forgotten flow doesn't hold the port forever. */
const TIMEOUT_MS = 5 * 60 * 1000;

/**
 * Step 6: run the OAuth flow. The Rust side blocks until Google redirects back
 * to localhost, so everything here is driven by the `oauth://*` events.
 */
export function AuthorizeStep({
  credentials,
  onDone,
  onBack,
}: {
  credentials: Credentials;
  onDone: (account: AddedAccount) => void;
  onBack: () => void;
}) {
  const { t } = useI18n();
  const s = t.wizard.steps.authorize;

  const [consentUrl, setConsentUrl] = useState<string | null>(null);
  const [error, setError] = useState<UiError | null>(null);
  const [attempt, setAttempt] = useState(0);

  // Keep the latest onDone without making it a dependency of the flow effect,
  // which must run exactly once per attempt.
  const onDoneRef = useRef(onDone);
  onDoneRef.current = onDone;

  useEffect(() => {
    let cancelled = false;
    let timeoutId: ReturnType<typeof setTimeout> | undefined;
    let unlisten: (() => void) | undefined;

    setConsentUrl(null);
    setError(null);

    void (async () => {
      unlisten = await onOauthEvents({
        onUrl: (url) => {
          if (cancelled) return;
          setConsentUrl(url);
          // Open it for them, but the url stays on screen as a fallback for
          // machines where no browser can be launched.
          void openUrl(url);
        },
        onDone: (account) => {
          if (!cancelled) onDoneRef.current(account);
        },
        onError: (err) => {
          if (!cancelled) setError(err);
        },
      });

      if (cancelled) return;

      try {
        await accountAddStart(credentials.clientId, credentials.clientSecret);
      } catch (err) {
        if (!cancelled) setError(toUiError(err));
        return;
      }

      timeoutId = setTimeout(() => {
        if (cancelled) return;
        void accountAddCancel();
        setError({ code: "timeout", message: s.timeout, hint: null });
      }, TIMEOUT_MS);
    })();

    return () => {
      cancelled = true;
      if (timeoutId) clearTimeout(timeoutId);
      unlisten?.();
      // Leaving this screen must free the redirect port.
      void accountAddCancel();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [attempt]);

  return (
    <div className="wizard-panel">
      <h1>{s.title}</h1>

      {error ? (
        <ErrorBox error={error} onRetry={() => setAttempt((n) => n + 1)} />
      ) : (
        <>
          <div className="waiting">
            <Spinner />
            <span>{consentUrl ? s.waiting : s.opening}</span>
          </div>

          {consentUrl && (
            <>
              <p className="body">{s.urlLabel}</p>
              <div className="url-box mono">{consentUrl}</div>
              <div className="row">
                <CopyButton value={consentUrl} />
                <button className="btn small" onClick={() => void openUrl(consentUrl)}>
                  {t.common.openInBrowser}
                </button>
              </div>
            </>
          )}

          <Warning>{s.unverified}</Warning>
        </>
      )}

      <div className="wizard-foot">
        <button
          className="btn"
          onClick={() => {
            void accountAddCancel();
            onBack();
          }}
        >
          {t.common.back}
        </button>
        <span className="spacer" />
      </div>
    </div>
  );
}
