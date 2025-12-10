import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { enable, isEnabled, disable } from "@tauri-apps/plugin-autostart";
import { open } from "@tauri-apps/plugin-shell";
import "./App.css";

type SyncStatus = {
  last_url?: string | null;
  last_saved_path?: string | null;
  last_result?: string | null;
  last_error?: string | null;
  last_run?: string | null;
};

type AppSettings = {
  auto_enabled: boolean;
  apply_all: boolean;
  resolution: string;
};

function App() {
  const [isSyncing, setIsSyncing] = useState(false);
  const [autoSync, setAutoSync] = useState(true);
  const [applyAllDisplays, setApplyAllDisplays] = useState(true);
  const [resolution, setResolution] = useState("UHD");
  const [startAtLogin, setStartAtLogin] = useState(false);
  const [status, setStatus] = useState<SyncStatus | null>(null);
  const [error, setError] = useState<string | null>(null);

  const lastRun = useMemo(() => {
    if (!status?.last_run) return "Not yet";
    const parsed = new Date(status.last_run);
    return isNaN(parsed.getTime()) ? "Unknown" : parsed.toLocaleString();
  }, [status?.last_run]);

  const runSync = async () => {
    setIsSyncing(true);
    setError(null);
    try {
      const result = await invoke<SyncStatus>("sync_wallpaper", {
        applyAll: applyAllDisplays,
      });
      setStatus(result);
    } catch (err) {
      setError(String(err));
    } finally {
      setIsSyncing(false);
    }
  };

  const updateAutoSync = async (enabled: boolean, applyAll: boolean) => {
    try {
      await invoke("set_auto_sync", { enabled, applyAll });
    } catch (err) {
      setError(String(err));
    }
  };

  const updateResolution = async (newResolution: string) => {
    try {
      await invoke("set_resolution", { resolution: newResolution });
      setResolution(newResolution);
    } catch (err) {
      setError(String(err));
    }
  };

  const toggleStartAtLogin = async (enabled: boolean) => {
    try {
      if (enabled) {
        await enable();
      } else {
        await disable();
      }
      setStartAtLogin(enabled);
    } catch (err) {
      setError(String(err));
    }
  };

  const handleClearCache = async () => {
    try {
      const result = await invoke<string>("clear_cache");
      alert(result);
    } catch (err) {
      setError(String(err));
    }
  };

  useEffect(() => {
    (async () => {
      try {
        const result = await invoke<SyncStatus>("get_status");
        setStatus(result);

        const settings = await invoke<AppSettings>("get_settings");
        setAutoSync(settings.auto_enabled);
        setApplyAllDisplays(settings.apply_all);
        setResolution(settings.resolution);

        const autostartEnabled = await isEnabled();
        setStartAtLogin(autostartEnabled);
      } catch (err) {
        console.error(err);
      }
    })();
  }, []);

  useEffect(() => {
    updateAutoSync(autoSync, applyAllDisplays);
  }, [autoSync, applyAllDisplays]);

  return (
    <div className="page">
      <header className="hero">
        <div>
          <p className="eyebrow">Bingscape</p>
          <h1>Fresh wallpaper, every hour.</h1>
          <p className="lede">
            Pull the latest Bing daily image, apply it to your desktop, and keep it
            running in the background—no dock icon, no fuss.
          </p>
          <div className="hero-actions">
            <button className="primary" onClick={runSync} disabled={isSyncing}>
              {isSyncing ? "Syncing..." : "Sync now"}
            </button>
            <span className="hint">Manual sync anytime.</span>
          </div>
        </div>
        <div className="pill">
          <span className={`status-dot ${isSyncing ? "warn" : "ok"}`} />
          {isSyncing ? "Updating wallpaper" : "Ready"}
        </div>
      </header>

      <section className="grid">
        <div className="card">
          <div className="card-head">
            <div>
              <p className="eyebrow">Automation</p>
              <h2>Hourly polling</h2>
            </div>
            <label className="switch">
              <input
                type="checkbox"
                checked={autoSync}
                onChange={(e) => setAutoSync(e.target.checked)}
              />
              <span className="slider" />
            </label>
          </div>
          <p className="body">Stay in sync automatically every hour.</p>
          <div className="option">
            <div>
              <p className="option-title">Apply to all displays</p>
              <p className="option-sub">If off, only the main display updates.</p>
            </div>
            <label className="switch">
              <input
                type="checkbox"
                checked={applyAllDisplays}
                onChange={(e) => setApplyAllDisplays(e.target.checked)}
              />
              <span className="slider" />
            </label>
          </div>
          <div className="option">
            <div>
              <p className="option-title">Image resolution</p>
              <p className="option-sub">Choose between UHD or HD quality.</p>
            </div>
            <select
              value={resolution}
              onChange={(e) => updateResolution(e.target.value)}
              className="resolution-select"
            >
              <option value="UHD">UHD (3840x2160)</option>
              <option value="1920x1080">HD (1920x1080)</option>
            </select>
          </div>
          <div className="option">
            <div>
              <p className="option-title">Start at login</p>
              <p className="option-sub">Launch automatically when you log in.</p>
            </div>
            <label className="switch">
              <input
                type="checkbox"
                checked={startAtLogin}
                onChange={(e) => toggleStartAtLogin(e.target.checked)}
              />
              <span className="slider" />
            </label>
          </div>
          <div className="option">
            <div>
              <p className="option-title">Clear cache</p>
              <p className="option-sub">Remove all cached wallpaper images.</p>
            </div>
            <button className="ghost small" onClick={handleClearCache}>
              Clear
            </button>
          </div>
        </div>

        <div className="card">
          <div className="card-head">
            <div>
              <p className="eyebrow">Status</p>
              <h2>Latest sync</h2>
            </div>
            <button className="ghost" onClick={runSync} disabled={isSyncing}>
              {isSyncing ? "Working" : "Sync now"}
            </button>
          </div>
          <ul className="meta">
            <li>
              <span>Last run</span>
              <strong>{lastRun}</strong>
            </li>
            <li>
              <span>Target displays</span>
              <strong>{applyAllDisplays ? "All" : "Main only"}</strong>
            </li>
            <li>
              <span>Image</span>
              {status?.last_url ? (
                <strong
                  className="link"
                  onClick={() => status.last_url && open(status.last_url)}
                >
                  Open image
                </strong>
              ) : (
                <strong>Not fetched</strong>
              )}
            </li>
            <li>
              <span>File</span>
              {status?.last_saved_path ? (
                <strong
                  className="link"
                  onClick={async () => {
                    try {
                      if (status.last_saved_path) {
                        const { Command } = await import("@tauri-apps/plugin-shell");
                        await Command.create("open", ["-R", status.last_saved_path]).execute();
                      }
                    } catch (err) {
                      console.error("Failed to open in Finder:", err);
                      setError(String(err));
                    }
                  }}
                >
                  Open in Finder
                </strong>
              ) : (
                <strong>No file yet</strong>
              )}
            </li>
            <li>
              <span>Result</span>
              <strong className={status?.last_error ? "error" : "success"}>
                {status?.last_error ?? status?.last_result ?? "Waiting..."}
              </strong>
            </li>
          </ul>
          {error ? <p className="error-text">{error}</p> : null}
        </div>
      </section>
    </div>
  );
}

export default App;
