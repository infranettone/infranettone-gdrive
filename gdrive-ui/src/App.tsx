import { useCallback, useEffect, useState } from "react";
import { accountList, type AccountSummary } from "./lib/api";
import { LangContext, strings, type Lang } from "./lib/i18n";
import { Accounts } from "./routes/Accounts";
import { Files } from "./routes/Files";
import { Drives } from "./routes/Drives";
import { Permissions } from "./routes/Permissions";
import { AddAccountWizard } from "./wizard/AddAccountWizard";

type Route = "accounts" | "files" | "drives" | "permissions";

const LANG_KEY = "gdrive.lang";

function initialLang(): Lang {
  try {
    const stored = localStorage.getItem(LANG_KEY);
    if (stored === "es" || stored === "en") return stored;
  } catch {
    // ignore
  }
  return navigator.language.startsWith("es") ? "es" : "en";
}

export default function App() {
  const [lang, setLangState] = useState<Lang>(initialLang);
  const [route, setRoute] = useState<Route>("accounts");
  const [accounts, setAccounts] = useState<AccountSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [wizardOpen, setWizardOpen] = useState(false);

  const t = strings[lang];

  const setLang = useCallback((next: Lang) => {
    setLangState(next);
    try {
      localStorage.setItem(LANG_KEY, next);
    } catch {
      // ignore
    }
  }, []);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      setAccounts(await accountList());
    } catch {
      // Listing accounts fails only if the config dir is unreadable; the
      // Accounts screen surfaces the real errors from actions.
      setAccounts([]);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  // Nothing works without an account, so land on Accounts until there is one.
  useEffect(() => {
    if (!loading && accounts.length === 0) setRoute("accounts");
  }, [loading, accounts.length]);

  const current = accounts.find((a) => a.isCurrent);
  const hasAccount = current !== undefined;

  const routes: { id: Route; label: string }[] = [
    { id: "accounts", label: t.nav.accounts },
    { id: "files", label: t.nav.files },
    { id: "drives", label: t.nav.drives },
    { id: "permissions", label: t.nav.permissions },
  ];

  return (
    <LangContext.Provider value={{ lang, t, setLang }}>
      <div className="app">
        <nav className="sidebar">
          <div className="brand">
            <span aria-hidden="true">🗂️</span>
            {t.appName}
          </div>

          {routes.map((item) => (
            <button
              key={item.id}
              className="nav-item"
              aria-current={route === item.id}
              onClick={() => setRoute(item.id)}
            >
              {item.label}
            </button>
          ))}

          <div className="sidebar-foot">
            {current && <div className="account-chip">{current.name}</div>}
            <select
              aria-label="Language"
              value={lang}
              onChange={(e) => setLang(e.target.value as Lang)}
            >
              <option value="es">Español</option>
              <option value="en">English</option>
            </select>
          </div>
        </nav>

        <main className="main">
          {route === "accounts" && (
            <Accounts
              accounts={accounts}
              loading={loading}
              onRefresh={() => void refresh()}
              onAdd={() => setWizardOpen(true)}
            />
          )}
          {route === "files" && <Files hasAccount={hasAccount} />}
          {route === "drives" && <Drives hasAccount={hasAccount} />}
          {route === "permissions" && <Permissions hasAccount={hasAccount} />}
        </main>
      </div>

      {wizardOpen && (
        <AddAccountWizard
          onClose={() => {
            setWizardOpen(false);
            void refresh();
          }}
          onAdded={() => void refresh()}
        />
      )}
    </LangContext.Provider>
  );
}
