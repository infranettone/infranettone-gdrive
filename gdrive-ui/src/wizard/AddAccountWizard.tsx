import { useEffect, useState } from "react";
import { CopyButton, ExternalButton, Warning } from "../components/common";
import { REQUIRED_SCOPES, consoleUrls, useI18n } from "../lib/i18n";
import type { AddedAccount } from "../lib/api";
import { CredentialsStep, type Credentials } from "./steps/CredentialsStep";
import { AuthorizeStep } from "./steps/AuthorizeStep";

/** Screens, in order. `guide` screens are the Google Cloud walk-through. */
type StepId =
  | "welcome"
  | "project"
  | "enableApi"
  | "consent"
  | "client"
  | "credentials"
  | "authorize"
  | "done";

const STEPS: StepId[] = [
  "welcome",
  "project",
  "enableApi",
  "consent",
  "client",
  "credentials",
  "authorize",
  "done",
];

/**
 * Where the user got to, so closing the app mid-way doesn't mean redoing the
 * Google Cloud steps. Deliberately excludes the client secret: that never
 * touches localStorage, only the wizard's in-memory state and, once the flow
 * succeeds, the config dir with 0600 permissions.
 */
interface PersistedProgress {
  stepIndex: number;
  projectId: string;
  acknowledged: StepId[];
}

const STORAGE_KEY = "gdrive.wizard.progress";

function loadProgress(): PersistedProgress {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      const parsed = JSON.parse(raw) as Partial<PersistedProgress>;
      return {
        // Never resume straight into the live OAuth flow or the done screen.
        stepIndex: Math.min(Math.max(parsed.stepIndex ?? 0, 0), STEPS.indexOf("credentials")),
        projectId: parsed.projectId ?? "",
        acknowledged: parsed.acknowledged ?? [],
      };
    }
  } catch {
    // A private window or cleared storage just means starting fresh.
  }
  return { stepIndex: 0, projectId: "", acknowledged: [] };
}

export function AddAccountWizard({
  onClose,
  onAdded,
}: {
  onClose: () => void;
  onAdded: (account: AddedAccount) => void;
}) {
  const { t } = useI18n();
  const w = t.wizard;

  const initial = useState(loadProgress)[0];
  const [stepIndex, setStepIndex] = useState(initial.stepIndex);
  const [projectId, setProjectId] = useState(initial.projectId);
  const [acknowledged, setAcknowledged] = useState<StepId[]>(initial.acknowledged);
  const [credentials, setCredentials] = useState<Credentials>({ clientId: "", clientSecret: "" });
  const [added, setAdded] = useState<AddedAccount | null>(null);

  const step = STEPS[stepIndex];
  const urls = consoleUrls(projectId);

  useEffect(() => {
    try {
      localStorage.setItem(
        STORAGE_KEY,
        JSON.stringify({ stepIndex, projectId, acknowledged } satisfies PersistedProgress),
      );
    } catch {
      // Persisting progress is a convenience; never break the wizard over it.
    }
  }, [stepIndex, projectId, acknowledged]);

  function goTo(id: StepId) {
    setStepIndex(STEPS.indexOf(id));
  }

  function acknowledge(id: StepId) {
    setAcknowledged((prev) => (prev.includes(id) ? prev : [...prev, id]));
    setStepIndex((i) => Math.min(i + 1, STEPS.length - 1));
  }

  function finish() {
    try {
      localStorage.removeItem(STORAGE_KEY);
    } catch {
      // ignore
    }
    onClose();
  }

  return (
    <div className="wizard" role="dialog" aria-modal="true" aria-label={w.title}>
      <aside className="wizard-steps">
        <strong>{w.title}</strong>
        <div style={{ color: "var(--text-dim)", fontSize: 12.5 }}>
          {w.stepOf(stepIndex + 1, STEPS.length)}
        </div>

        <ol>
          {STEPS.map((id, index) => {
            const isDone = acknowledged.includes(id) || index < stepIndex;
            const isActive = index === stepIndex;
            // Don't let the rail jump into or past the live OAuth flow.
            const reachable = index <= stepIndex && step !== "authorize" && step !== "done";

            return (
              <li key={id} aria-current={isActive}>
                <button disabled={!reachable} onClick={() => setStepIndex(index)}>
                  <span
                    className={`step-dot${isDone ? " done" : isActive ? " active" : ""}`}
                    aria-hidden="true"
                  >
                    {isDone ? "✓" : index + 1}
                  </span>
                  <span>{w.steps[id].title}</span>
                </button>
              </li>
            );
          })}
        </ol>

        <div style={{ marginTop: 18 }}>
          <button className="btn small" onClick={finish}>
            {t.common.cancel}
          </button>
        </div>
      </aside>

      <main className="wizard-body">
        {step === "welcome" && (
          <GuidePanel
            title={w.steps.welcome.title}
            lead={w.steps.welcome.lead}
            body={w.steps.welcome.body}
            footer={
              <>
                <button className="btn link" onClick={() => goTo("credentials")}>
                  {w.skipToCredentials}
                </button>
                <span className="spacer" />
                <button className="btn primary" onClick={() => acknowledge("welcome")}>
                  {t.common.next}
                </button>
              </>
            }
          />
        )}

        {step === "project" && (
          <GuidePanel
            title={w.steps.project.title}
            lead={w.steps.project.lead}
            body={w.steps.project.body}
            action={<ExternalButton url={urls.createProject}>{t.common.openInBrowser}</ExternalButton>}
            footer={<StepFooter onBack={() => setStepIndex(0)} onNext={() => acknowledge("project")} />}
          >
            <div className="field">
              <label htmlFor="project-id">{w.projectIdLabel}</label>
              <input
                id="project-id"
                type="text"
                className="mono"
                spellCheck={false}
                placeholder={w.projectIdPlaceholder}
                value={projectId}
                onChange={(e) => setProjectId(e.target.value)}
              />
            </div>
          </GuidePanel>
        )}

        {step === "enableApi" && (
          <GuidePanel
            title={w.steps.enableApi.title}
            lead={w.steps.enableApi.lead}
            body={w.steps.enableApi.body}
            action={
              <ExternalButton url={urls.enableDriveApi}>{t.common.openInBrowser}</ExternalButton>
            }
            footer={
              <StepFooter
                onBack={() => goTo("project")}
                onNext={() => acknowledge("enableApi")}
                nextLabel={w.markDone}
              />
            }
          />
        )}

        {step === "consent" && (
          <GuidePanel
            title={w.steps.consent.title}
            lead={w.steps.consent.lead}
            body={w.steps.consent.body}
            action={
              <ExternalButton url={urls.consentScreen}>{t.common.openInBrowser}</ExternalButton>
            }
            footer={
              <StepFooter
                onBack={() => goTo("enableApi")}
                onNext={() => acknowledge("consent")}
                nextLabel={w.markDone}
              />
            }
          >
            <div className="scopes">
              <strong>{w.steps.consent.scopesLabel}</strong>
              <ul className="mono">
                {REQUIRED_SCOPES.map((scope) => (
                  <li key={scope}>{scope}</li>
                ))}
              </ul>
              <CopyButton value={REQUIRED_SCOPES.join("\n")} />
            </div>
            <Warning>{w.steps.consent.publishWarning}</Warning>
          </GuidePanel>
        )}

        {step === "client" && (
          <GuidePanel
            title={w.steps.client.title}
            lead={w.steps.client.lead}
            body={w.steps.client.body}
            action={
              <ExternalButton url={urls.createClient}>{t.common.openInBrowser}</ExternalButton>
            }
            footer={
              <StepFooter
                onBack={() => goTo("consent")}
                onNext={() => acknowledge("client")}
                nextLabel={w.markDone}
              />
            }
          />
        )}

        {step === "credentials" && (
          <CredentialsStep
            value={credentials}
            onChange={setCredentials}
            onSubmit={() => goTo("authorize")}
          />
        )}

        {step === "authorize" && (
          <AuthorizeStep
            credentials={credentials}
            onBack={() => goTo("credentials")}
            onDone={(account) => {
              setAdded(account);
              // Drop the secret from memory as soon as it is persisted.
              setCredentials({ clientId: "", clientSecret: "" });
              onAdded(account);
              goTo("done");
            }}
          />
        )}

        {step === "done" && added && (
          <div className="wizard-panel">
            <h1>{w.steps.done.title}</h1>
            <p className="lead">{w.steps.done.loggedIn(added.email)}</p>
            <p className="body mono">{w.steps.done.savedIn(added.basePath)}</p>
            <Warning>{w.steps.done.keepSafe}</Warning>
            <p className="body">{w.steps.done.alsoCli}</p>
            <div className="wizard-foot">
              <span className="spacer" />
              <button className="btn primary" onClick={finish}>
                {w.steps.done.goToFiles}
              </button>
            </div>
          </div>
        )}
      </main>
    </div>
  );
}

function GuidePanel({
  title,
  lead,
  body,
  action,
  children,
  footer,
}: {
  title: string;
  lead: string;
  body: readonly string[];
  action?: React.ReactNode;
  children?: React.ReactNode;
  footer: React.ReactNode;
}) {
  return (
    <div className="wizard-panel">
      <h1>{title}</h1>
      <p className="lead">{lead}</p>
      <ol className="instructions">
        {body.map((line) => (
          <li key={line}>{line}</li>
        ))}
      </ol>
      {children}
      {action && <div className="row">{action}</div>}
      <div className="wizard-foot">{footer}</div>
    </div>
  );
}

function StepFooter({
  onBack,
  onNext,
  nextLabel,
}: {
  onBack: () => void;
  onNext: () => void;
  nextLabel?: string;
}) {
  const { t } = useI18n();

  return (
    <>
      <button className="btn" onClick={onBack}>
        {t.common.back}
      </button>
      <span className="spacer" />
      <button className="btn primary" onClick={onNext}>
        {nextLabel ?? t.common.next}
      </button>
    </>
  );
}
