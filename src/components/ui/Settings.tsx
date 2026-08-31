import { useCallback, useEffect, useState } from "react";
import { getVersion } from "@tauri-apps/api/app";
import { useAppStore, useSettings as useSettingsStore } from "../../store/app";
import { saveSettings, setAlwaysOnTop } from "../../api/tauri";
import { LANGUAGE_LIST } from "../../types";
import type { LanguageId, Settings } from "../../types";

// ============================================================
// Tatpar — Settings Panel
// Phase 3 Step 2: Appearance · Execution · Startup · About
// ============================================================

export function SettingsPanel() {
  const [appVersion, setAppVersion] = useState<string>("—");

  useEffect(() => {
    getVersion()
      .then(setAppVersion)
      .catch(() => setAppVersion("unknown"));
  }, []);
  const settings = useSettingsStore();
  const setSettings = useAppStore((s) => s.setSettings);

  // Persist to disk immediately after every change
  const persist = useCallback(
    async (updated: Settings) => {
      try {
        await saveSettings(updated);
      } catch (err) {
        console.warn("[Tatpar] save_settings failed:", err);
      }
    },
    []
  );

  const update = useCallback(
    (partial: Partial<Settings>) => {
      const updated = { ...settings, ...partial };
      setSettings(partial);
      persist(updated);
    },
    [settings, setSettings, persist]
  );

  const handleAlwaysOnTop = async (val: boolean) => {
    update({ window: { ...settings.window, alwaysOnTop: val } });
    try {
      await setAlwaysOnTop(val);
    } catch (err) {
      console.warn("[Tatpar] set_always_on_top failed:", err);
    }
  };

  const handleLanguageTimeout = (langId: LanguageId, secs: number) => {
    update({
      languageSettings: {
        ...settings.languageSettings,
        [langId]: {
          ...settings.languageSettings[langId],
          timeoutSecs: secs,
        },
      },
    });
  };

  return (
    <div className="settings-panel" role="region" aria-label="Settings">
      <div className="settings-inner">

        {/* ── Appearance ── */}
        <section className="settings-section">
          <h2 className="settings-section-title">Appearance</h2>

          <div className="settings-row">
            <label className="settings-label" htmlFor="theme-select">Theme</label>
            <div className="settings-theme-toggle">
              <button
                id="theme-btn-dark"
                className={`settings-theme-btn${settings.theme === "dark" ? " settings-theme-btn--active" : ""}`}
                onClick={() => update({ theme: "dark" })}
                aria-pressed={settings.theme === "dark"}
              >
                Dark
              </button>
              <button
                id="theme-btn-light"
                className={`settings-theme-btn${settings.theme === "light" ? " settings-theme-btn--active" : ""}`}
                onClick={() => update({ theme: "light" })}
                aria-pressed={settings.theme === "light"}
              >
                Light
              </button>
            </div>
          </div>

          <div className="settings-row">
            <label className="settings-label" htmlFor="font-size-slider">
              Editor Font Size
            </label>
            <div className="settings-slider-group">
              <input
                id="font-size-slider"
                type="range"
                className="settings-slider"
                min={11}
                max={20}
                step={1}
                value={settings.editorFontSize}
                onChange={(e) => update({ editorFontSize: Number(e.target.value) })}
                aria-valuemin={11}
                aria-valuemax={20}
                aria-valuenow={settings.editorFontSize}
              />
              <span className="settings-value">{settings.editorFontSize}px</span>
            </div>
          </div>
        </section>

        {/* ── Window ── */}
        <section className="settings-section">
          <h2 className="settings-section-title">Window</h2>

          <div className="settings-row">
            <div className="settings-label-group">
              <label className="settings-label" htmlFor="always-on-top-toggle">Always on Top</label>
              <span className="settings-hint">Keep Tatpar above all other windows</span>
            </div>
            <Toggle
              id="always-on-top-toggle"
              checked={settings.window.alwaysOnTop}
              onChange={handleAlwaysOnTop}
            />
          </div>
        </section>

        {/* ── Execution ── */}
        <section className="settings-section">
          <h2 className="settings-section-title">Execution Timeouts</h2>
          <p className="settings-section-desc">
            Maximum time (seconds) each language is allowed to run before being cancelled.
          </p>

          {LANGUAGE_LIST.map((lang) => {
            const timeout = settings.languageSettings[lang.id]?.timeoutSecs ?? 10;
            return (
              <div className="settings-row" key={lang.id}>
                <label
                  className="settings-label"
                  htmlFor={`timeout-${lang.id}`}
                >
                  {lang.name}
                </label>
                <div className="settings-slider-group">
                  <input
                    id={`timeout-${lang.id}`}
                    type="range"
                    className="settings-slider"
                    min={5}
                    max={60}
                    step={5}
                    value={timeout}
                    onChange={(e) =>
                      handleLanguageTimeout(lang.id as LanguageId, Number(e.target.value))
                    }
                    aria-valuemin={5}
                    aria-valuemax={60}
                    aria-valuenow={timeout}
                  />
                  <span className="settings-value">{timeout}s</span>
                </div>
              </div>
            );
          })}
        </section>

        {/* ── Startup ── */}
        <section className="settings-section">
          <h2 className="settings-section-title">Startup</h2>

          <div className="settings-row">
            <div className="settings-label-group">
              <label className="settings-label" htmlFor="startup-toggle">Launch at Windows Startup</label>
              <span className="settings-hint">Automatically start Tatpar when you log in</span>
            </div>
            <Toggle
              id="startup-toggle"
              checked={settings.launchOnStartup}
              onChange={(val) => update({ launchOnStartup: val })}
            />
          </div>
        </section>

        {/* ── About ── */}
        <section className="settings-section settings-section--about">
          <h2 className="settings-section-title">About</h2>

          <div className="settings-about-grid">
            <span className="settings-about-key">Version</span>
            <span className="settings-about-val">v{appVersion}</span>

            <span className="settings-about-key">Global Hotkey</span>
            <div className="settings-about-val">
              <kbd className="settings-kbd">{settings.hotkey.replace(/\+/g, " + ").toUpperCase()}</kbd>
            </div>

            <span className="settings-about-key">Framework</span>
            <span className="settings-about-val">Tauri v2 + React 19</span>

            <span className="settings-about-key">License</span>
            <span className="settings-about-val">MIT</span>
          </div>
        </section>

      </div>
    </div>
  );
}

// ─── Toggle Switch ────────────────────────────────────────────

function Toggle({
  id,
  checked,
  onChange,
}: {
  id: string;
  checked: boolean;
  onChange: (val: boolean) => void;
}) {
  return (
    <button
      id={id}
      role="switch"
      aria-checked={checked}
      className={`settings-toggle${checked ? " settings-toggle--on" : ""}`}
      onClick={() => onChange(!checked)}
    >
      <span className="settings-toggle-thumb" />
    </button>
  );
}
